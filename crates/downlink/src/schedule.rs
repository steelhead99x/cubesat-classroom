//! Which track is published when the fan-out has a byte budget.
//!
//! Telemetry JSON is the core track: it is sent even when it is larger than the
//! budget, and an unsent older snapshot is replaced by the newer one. Media is
//! dropped first. The numbers on [`FeedTrack::priority`] are the moq-net model
//! (higher is more urgent). IETF subscriber priority is the opposite, so
//! [`FeedTrack::ietf_priority`] is `255 - priority`.

use std::collections::VecDeque;
use std::time::Duration;

/// Broadcast name for the live classroom feed.
pub const LIVE_BROADCAST: &str = "cubesat/classroom/live";

/// A track a classroom client can subscribe to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedTrack {
    /// JSON telemetry. Required. Full snapshot every object.
    Telemetry,
    /// CBOR of the same snapshot. Optional alternate for a native client.
    TelemetryCbor,
    /// Ground receiver totals. Optional. Not from the air frame.
    Health,
    /// Optional JPEG still.
    Image,
    /// Optional mono PCM. Skip it if the client has no speaker.
    Audio,
    /// Optional low-rate video. Each object is one independent JPEG.
    Video,
}

impl FeedTrack {
    /// Track name inside [`LIVE_BROADCAST`].
    pub const fn name(self) -> &'static str {
        match self {
            Self::Telemetry => "telemetry",
            Self::TelemetryCbor => "telemetry.cbor",
            Self::Health => "health",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
        }
    }

    /// moq-net publisher and subscriber priority. Higher is served first.
    pub const fn priority(self) -> u8 {
        match self {
            Self::Telemetry => 240,
            Self::TelemetryCbor => 230,
            Self::Health => 200,
            Self::Image => 100,
            Self::Audio => 80,
            Self::Video => 40,
        }
    }

    /// IETF subscriber priority for the same urgency. Lower is served first.
    pub const fn ietf_priority(self) -> u8 {
        u8::MAX - self.priority()
    }

    /// How long the publisher should keep groups other than the newest, in milliseconds.
    ///
    /// The newest group is kept either way. Media uses a shorter window so a
    /// slow viewer cannot build a large backlog.
    pub const fn publisher_max_age_ms(self) -> u64 {
        match self {
            Self::Telemetry | Self::TelemetryCbor | Self::Health => 5_000,
            Self::Image | Self::Audio | Self::Video => 1_000,
        }
    }

    /// False only for JSON telemetry, which the budget is not allowed to skip.
    pub const fn droppable(self) -> bool {
        !matches!(self, Self::Telemetry)
    }

    /// Every track, core first.
    pub const fn all() -> [Self; 6] {
        [
            Self::Telemetry,
            Self::TelemetryCbor,
            Self::Health,
            Self::Image,
            Self::Audio,
            Self::Video,
        ]
    }
}

/// One object waiting for a publish slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// Destination track.
    pub track: FeedTrack,
    /// Snapshot or media sequence.
    pub sequence: u64,
    /// Track payload. JSON, CBOR, JPEG, or a `CSA1` audio frame.
    pub payload: Vec<u8>,
}

/// How many bytes [`Scheduler::drain`] may hand out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    bytes_per_tick: Option<usize>,
}

impl Budget {
    /// No cap. Media is still bounded by the per-track queues.
    pub const fn unlimited() -> Self {
        Self {
            bytes_per_tick: None,
        }
    }

    /// Convert a bit rate and a tick length into a byte cap.
    ///
    /// Zero bits per second still sends JSON telemetry and sends no media.
    /// The core snapshot is not silenced by a tiny budget.
    pub fn from_bps(bps: u64, interval: Duration) -> Self {
        if bps == 0 {
            return Self {
                bytes_per_tick: Some(0),
            };
        }
        let ms = u64::try_from(interval.as_millis())
            .unwrap_or(u64::MAX)
            .max(1);
        let bytes = bps.saturating_mul(ms) / 8_000;
        Self {
            bytes_per_tick: Some(usize::try_from(bytes).unwrap_or(usize::MAX)),
        }
    }
}

/// Latest-wins telemetry and droppable media under a byte budget.
#[derive(Debug)]
pub struct Scheduler {
    budget: Option<usize>,
    telemetry: Option<Piece>,
    telemetry_cbor: Option<Piece>,
    health: Option<Piece>,
    image: Option<Piece>,
    audio: VecDeque<Piece>,
    video: VecDeque<Piece>,
    audio_limit: usize,
    video_limit: usize,
    dropped_media: u64,
    coalesced: u64,
}

impl Scheduler {
    /// A scheduler with room for a few unsent audio and video objects.
    pub fn new(budget: Budget) -> Self {
        Self {
            budget: budget.bytes_per_tick,
            telemetry: None,
            telemetry_cbor: None,
            health: None,
            image: None,
            audio: VecDeque::new(),
            video: VecDeque::new(),
            audio_limit: 4,
            video_limit: 2,
            dropped_media: 0,
            coalesced: 0,
        }
    }

    /// Media objects dropped, including ones replaced before they were sent.
    pub const fn dropped_media(&self) -> u64 {
        self.dropped_media
    }

    /// Telemetry or health snapshots replaced by a newer one before sending.
    pub const fn coalesced(&self) -> u64 {
        self.coalesced
    }

    /// Queue `piece`. An older unsent snapshot on the same track is replaced.
    pub fn push(&mut self, piece: Piece) {
        if piece.payload.is_empty() {
            if piece.track.droppable() {
                self.dropped_media += 1;
            }
            return;
        }
        match piece.track {
            FeedTrack::Telemetry => replace(&mut self.coalesced, &mut self.telemetry, piece),
            FeedTrack::TelemetryCbor => {
                replace(&mut self.coalesced, &mut self.telemetry_cbor, piece);
            }
            FeedTrack::Health => replace(&mut self.coalesced, &mut self.health, piece),
            FeedTrack::Image => replace_media(&mut self.dropped_media, &mut self.image, piece),
            FeedTrack::Audio => {
                push_media(
                    &mut self.dropped_media,
                    &mut self.audio,
                    self.audio_limit,
                    piece,
                );
            }
            FeedTrack::Video => {
                push_media(
                    &mut self.dropped_media,
                    &mut self.video,
                    self.video_limit,
                    piece,
                );
            }
        }
    }

    /// Objects to publish this tick, telemetry first.
    pub fn drain(&mut self) -> Vec<Piece> {
        let mut left = self.budget.unwrap_or(usize::MAX);
        let mut out = Vec::new();
        take(&mut self.telemetry, &mut left, &mut out, true);
        take(&mut self.telemetry_cbor, &mut left, &mut out, false);
        take(&mut self.health, &mut left, &mut out, false);
        take(&mut self.image, &mut left, &mut out, false);
        take_queue(&mut self.audio, &mut left, &mut out);
        take_queue(&mut self.video, &mut left, &mut out);
        out
    }
}

fn replace(counter: &mut u64, slot: &mut Option<Piece>, piece: Piece) {
    if slot.is_some() {
        *counter += 1;
    }
    *slot = Some(piece);
}

fn replace_media(dropped: &mut u64, slot: &mut Option<Piece>, piece: Piece) {
    if slot.is_some() {
        *dropped += 1;
    }
    *slot = Some(piece);
}

fn push_media(dropped: &mut u64, queue: &mut VecDeque<Piece>, limit: usize, piece: Piece) {
    while queue.len() >= limit {
        queue.pop_front();
        *dropped += 1;
    }
    queue.push_back(piece);
}

fn take(slot: &mut Option<Piece>, left: &mut usize, out: &mut Vec<Piece>, force: bool) {
    let Some(piece) = slot.as_ref() else {
        return;
    };
    if !force && piece.payload.len() > *left {
        return;
    }
    let piece = slot.take().expect("piece is present");
    *left = left.saturating_sub(piece.payload.len());
    out.push(piece);
}

fn take_queue(queue: &mut VecDeque<Piece>, left: &mut usize, out: &mut Vec<Piece>) {
    while queue
        .front()
        .is_some_and(|piece| piece.payload.len() <= *left)
    {
        let piece = queue.pop_front().expect("piece is present");
        *left = left.saturating_sub(piece.payload.len());
        out.push(piece);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(track: FeedTrack, sequence: u64, len: usize) -> Piece {
        Piece {
            track,
            sequence,
            payload: vec![1; len],
        }
    }

    #[test]
    fn telemetry_is_more_urgent_on_both_priority_scales() {
        assert!(FeedTrack::Telemetry.priority() > FeedTrack::Video.priority());
        assert!(FeedTrack::Telemetry.ietf_priority() < FeedTrack::Video.ietf_priority());
        assert_eq!(FeedTrack::Telemetry.ietf_priority(), 15);
        assert!(!FeedTrack::Telemetry.droppable());
        assert!(FeedTrack::Image.droppable());
    }

    #[test]
    fn a_tight_budget_sends_telemetry_and_holds_media() {
        let mut scheduler = Scheduler::new(Budget::from_bps(1_200, Duration::from_secs(1)));
        // 1200 bps for one second is 150 bytes.
        assert_eq!(scheduler.budget, Some(150));
        scheduler.push(piece(FeedTrack::Video, 1, 400));
        scheduler.push(piece(FeedTrack::Image, 1, 300));
        scheduler.push(piece(FeedTrack::Telemetry, 1, 120));
        scheduler.push(piece(FeedTrack::TelemetryCbor, 1, 80));
        let sent = scheduler.drain();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].track, FeedTrack::Telemetry);
        assert_eq!(sent[0].sequence, 1);
        // The image did not fit. A newer one replaces it and counts a drop.
        scheduler.push(piece(FeedTrack::Image, 2, 300));
        assert_eq!(scheduler.dropped_media(), 1);
        let sent = scheduler.drain();
        assert!(
            sent.iter()
                .all(|piece| piece.track == FeedTrack::TelemetryCbor)
        );
    }

    #[test]
    fn telemetry_still_sends_when_the_budget_is_zero() {
        let mut scheduler = Scheduler::new(Budget::from_bps(0, Duration::from_secs(1)));
        scheduler.push(piece(FeedTrack::Telemetry, 4, 200));
        scheduler.push(piece(FeedTrack::Video, 4, 20));
        let sent = scheduler.drain();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].sequence, 4);
        let sent = scheduler.drain();
        assert!(sent.is_empty(), "video stays queued and still does not fit");
    }

    #[test]
    fn an_unsent_snapshot_is_replaced_by_the_newer_one() {
        let mut scheduler = Scheduler::new(Budget::unlimited());
        scheduler.push(piece(FeedTrack::Telemetry, 1, 10));
        scheduler.push(piece(FeedTrack::Telemetry, 2, 10));
        assert_eq!(scheduler.coalesced(), 1);
        let sent = scheduler.drain();
        assert_eq!(sent[0].sequence, 2);
    }
}
