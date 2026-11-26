//! Reassemble air frames heard by one or more virtual ground receivers.
//!
//! A shard is identified by session, kind, sequence, and index. A second copy,
//! from the same receiver or another one, is counted and dropped. Payload is
//! returned as soon as enough shards exist, so a later copy can arrive out of
//! order. The incomplete-object map is capped: media is discarded before
//! telemetry when the cap is hit.

use std::collections::{HashSet, VecDeque};

use crate::fec;
use crate::frame::{AirFrame, AirKind, FrameFault, decode};

/// How many unfinished objects to hold. Older media goes first when this fills.
const MAX_INCOMPLETE: usize = 128;
/// Ignore a shard this many sequence numbers behind the newest of its kind.
const WINDOW: u32 = 128;
/// Refuse a shard this far ahead of the newest of its kind. The CRC is not authentication,
/// and one frame claiming sequence 4294967295 would otherwise make every real frame after
/// it "late" until the sender started a new session. About a day of frames at 12 per second
/// still fits, so a long gap between passes is fine.
const MAX_FORWARD_JUMP: u32 = 1 << 20;
/// After this many frames in a row are refused as too old or too far ahead, the newest
/// sequence is taken to be wrong (one spoofed or garbled frame set it) and the window moves
/// to the incoming frames. Real late copies come in ones and twos, not in a run like this.
const REANCHOR_AFTER: u32 = 16;
/// Finished object keys kept so a late copy is a duplicate, not a new object.
const RECENT: usize = 4096;

/// One reception of one air frame, with the receiver's own signal report.
///
/// RSSI and SNR are measurements the receiver adds. They are not inside the
/// air frame. In this crate the numbers are invented for the desk simulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heard {
    /// Receiver index, starting at 0.
    pub relay: u8,
    /// Last reported RSSI for this copy, in dBm.
    pub rssi_dbm: i16,
    /// Last reported SNR in tenths of a decibel. 80 is 8.0 dB.
    pub snr_db_x10: i16,
    /// Air frame bytes, including the CRC.
    pub frame: Vec<u8>,
}

/// An object that survived the virtual channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recovered {
    /// Air kind.
    pub kind: AirKind,
    /// Sender session.
    pub session: u16,
    /// Object sequence.
    pub sequence: u32,
    /// Payload with zero padding removed.
    pub payload: Vec<u8>,
    /// Service label from the air header.
    pub service: crate::frame::Service,
    /// Station ID from the air header, when the frame carried one.
    pub station_id: Option<String>,
    /// True when at least one payload shard was rebuilt from parity.
    pub repaired: bool,
}

/// Identity of one object on the air.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupKey {
    /// Air kind.
    pub kind: AirKind,
    /// Sender session.
    pub session: u16,
    /// Object sequence.
    pub sequence: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RelayStats {
    rssi_dbm: i16,
    snr_db_x10: i16,
    frames: u64,
    crc_failures: u64,
    duplicates: u64,
}

/// Running totals from the assembler. Suitable for a ground health snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metrics {
    /// Objects returned from [`Assembler::push`].
    pub recovered: u64,
    /// Incomplete objects dropped because the sender moved on or [`Assembler::give_up`] was called.
    pub lost: u64,
    /// Incomplete objects dropped because the map was full.
    pub abandoned: u64,
    /// Recovered objects that used parity.
    pub fec_repairs: u64,
    /// Frames whose CRC failed.
    pub crc_failures: u64,
    /// Frames whose CRC passed and whose header was not usable.
    pub malformed: u64,
    /// Extra copies of a shard or of an object already recovered.
    pub duplicates: u64,
    /// Shards that arrived too far behind the live sequence.
    pub late: u64,
    /// Frames handed to [`Assembler::push`], good or bad.
    pub frames: u64,
    /// Per-receiver totals. Index is the receiver number.
    pub relays: Vec<RelayView>,
}

/// One receiver's totals, with RSSI and SNR converted for display.
#[derive(Clone, Debug, PartialEq)]
pub struct RelayView {
    /// `relay-N`.
    pub id: String,
    /// Latest RSSI sample, in dBm.
    pub rssi_dbm: f64,
    /// Latest SNR sample, in dB.
    pub snr_db: f64,
    /// Frames from this receiver, including CRC failures.
    pub frames: u64,
    /// CRC failures from this receiver.
    pub crc_failures: u64,
    /// Duplicate shards from this receiver.
    pub duplicates: u64,
}

#[derive(Clone, Debug)]
struct Group {
    data_shards: usize,
    parity_shards: usize,
    shard_len: usize,
    object_len: usize,
    service: crate::frame::Service,
    station_id: Option<crate::frame::StationId>,
    data: Vec<Option<Vec<u8>>>,
    parity: Vec<Option<Vec<u8>>>,
    data_got: usize,
    parity_got: usize,
}

impl Group {
    fn new(frame: &AirFrame) -> Self {
        let data_shards = usize::from(frame.data_shards);
        let parity_shards = usize::from(frame.parity_shards);
        Self {
            data_shards,
            parity_shards,
            shard_len: frame.shard.len(),
            object_len: usize::try_from(frame.object_len).unwrap_or(usize::MAX),
            service: frame.service,
            station_id: frame.station_id.clone(),
            data: vec![None; data_shards],
            parity: vec![None; parity_shards],
            data_got: 0,
            parity_got: 0,
        }
    }

    fn matches(&self, frame: &AirFrame) -> bool {
        self.data_shards == usize::from(frame.data_shards)
            && self.parity_shards == usize::from(frame.parity_shards)
            && self.shard_len == frame.shard.len()
            && self.object_len == usize::try_from(frame.object_len).unwrap_or(usize::MAX)
            && self.service == frame.service
            && self.station_id == frame.station_id
    }

    /// `Ok(false)` when this shard was already stored.
    fn insert(&mut self, frame: &AirFrame) -> bool {
        let index = usize::from(frame.index);
        let slot = if index < self.data_shards {
            &mut self.data[index]
        } else {
            &mut self.parity[index - self.data_shards]
        };
        if slot.is_some() {
            return false;
        }
        *slot = Some(frame.shard.clone());
        if index < self.data_shards {
            self.data_got += 1;
        } else {
            self.parity_got += 1;
        }
        true
    }

    fn try_recover(&self) -> Result<Option<(Vec<u8>, bool)>, ()> {
        if self.data_got == self.data_shards {
            return Ok(Some((self.concat(&self.filled_data()), false)));
        }
        if self.parity_shards == 0 || self.data_got + self.parity_got < self.data_shards {
            return Ok(None);
        }
        match fec::restore(&self.data, &self.parity) {
            Ok(shards) => Ok(Some((self.concat(&shards), true))),
            Err(_) => Err(()),
        }
    }

    fn filled_data(&self) -> Vec<Vec<u8>> {
        self.data
            .iter()
            .map(|shard| shard.clone().expect("complete"))
            .collect()
    }

    fn concat(&self, shards: &[Vec<u8>]) -> Vec<u8> {
        let mut raw = Vec::with_capacity(self.object_len);
        for shard in shards {
            raw.extend_from_slice(shard);
        }
        raw.truncate(self.object_len);
        raw
    }
}

/// Bounded reassembler for one or many receivers.
#[derive(Debug)]
pub struct Assembler {
    groups: std::collections::BTreeMap<GroupKey, Group>,
    recent: HashSet<GroupKey>,
    recent_order: VecDeque<GroupKey>,
    newest: std::collections::BTreeMap<(u16, AirKind), u32>,
    /// Frames refused in a row for being outside the window, per (session, kind).
    refused_run: std::collections::BTreeMap<(u16, AirKind), u32>,
    relay_stats: Vec<RelayStats>,
    metrics: Metrics,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    /// An empty assembler.
    pub fn new() -> Self {
        Self {
            groups: std::collections::BTreeMap::new(),
            recent: HashSet::new(),
            recent_order: VecDeque::new(),
            newest: std::collections::BTreeMap::new(),
            refused_run: std::collections::BTreeMap::new(),
            relay_stats: Vec::new(),
            metrics: Metrics::default(),
        }
    }

    /// Totals so far.
    pub fn metrics(&self) -> Metrics {
        let mut metrics = self.metrics.clone();
        metrics.relays = self
            .relay_stats
            .iter()
            .enumerate()
            .map(|(index, relay)| RelayView {
                id: format!("relay-{index}"),
                rssi_dbm: f64::from(relay.rssi_dbm),
                snr_db: f64::from(relay.snr_db_x10) / 10.0,
                frames: relay.frames,
                crc_failures: relay.crc_failures,
                duplicates: relay.duplicates,
            })
            .collect();
        metrics
    }

    /// Accept one reception. Returns the object when this shard completes it.
    pub fn push(&mut self, heard: Heard) -> Option<Recovered> {
        self.metrics.frames += 1;
        let relay = usize::from(heard.relay);
        if self.relay_stats.len() <= relay {
            self.relay_stats.resize(relay + 1, RelayStats::default());
        }
        self.relay_stats[relay].frames += 1;
        self.relay_stats[relay].rssi_dbm = heard.rssi_dbm;
        self.relay_stats[relay].snr_db_x10 = heard.snr_db_x10;

        let frame = match decode(&heard.frame) {
            Ok(frame) => frame,
            Err(FrameFault::Crc) => {
                self.metrics.crc_failures += 1;
                self.relay_stats[relay].crc_failures += 1;
                return None;
            }
            Err(FrameFault::Malformed) => {
                self.metrics.malformed += 1;
                return None;
            }
        };
        let key = GroupKey {
            kind: frame.kind,
            session: frame.session,
            sequence: frame.sequence,
        };
        if self.recent.contains(&key) {
            self.count_duplicate(relay);
            return None;
        }
        let stream = (frame.session, frame.kind);
        if let Some(&newest) = self.newest.get(&stream) {
            let late = frame.sequence.saturating_add(WINDOW) < newest;
            let jump = frame.sequence > newest.saturating_add(MAX_FORWARD_JUMP);
            if late || jump {
                let run = self.refused_run.entry(stream).or_insert(0);
                *run += 1;
                if *run < REANCHOR_AFTER {
                    if late {
                        self.metrics.late += 1;
                    } else {
                        // Counted as malformed: the header passed the CRC but is not usable.
                        self.metrics.malformed += 1;
                    }
                    return None;
                }
                // A long run of refusals: the newest sequence was the bad one. Start over
                // from this frame. This also recovers when the very first frame of a
                // session was the spoofed one, which no bound can catch on arrival.
                self.newest.insert(stream, frame.sequence);
            }
        }
        self.refused_run.remove(&stream);
        self.newest
            .entry(stream)
            .and_modify(|newest| *newest = (*newest).max(frame.sequence))
            .or_insert(frame.sequence);

        if !self.groups.contains_key(&key) && self.groups.len() >= MAX_INCOMPLETE {
            self.evict_one();
        }
        if !self.groups.contains_key(&key) && self.groups.len() >= MAX_INCOMPLETE {
            self.metrics.abandoned += 1;
            return None;
        }
        let group = self.groups.entry(key).or_insert_with(|| Group::new(&frame));
        if !group.matches(&frame) {
            self.metrics.malformed += 1;
            return None;
        }
        if !group.insert(&frame) {
            self.count_duplicate(relay);
            return None;
        }
        match group.try_recover() {
            Ok(None) => None,
            Ok(Some((payload, repaired))) => {
                let service = group.service;
                let station_id = group.station_id.as_ref().map(|id| id.as_str().to_string());
                self.groups.remove(&key);
                self.remember(key);
                self.metrics.recovered += 1;
                if repaired {
                    self.metrics.fec_repairs += 1;
                }
                Some(Recovered {
                    kind: key.kind,
                    session: key.session,
                    sequence: key.sequence,
                    payload,
                    service,
                    station_id,
                    repaired,
                })
            }
            Err(()) => {
                self.groups.remove(&key);
                self.metrics.lost += 1;
                None
            }
        }
    }

    /// Drop one unfinished object because its remaining shards will not arrive.
    pub fn give_up(&mut self, key: GroupKey) {
        if self.groups.remove(&key).is_some() {
            self.metrics.lost += 1;
        }
    }

    /// Count every unfinished object as lost and clear them.
    pub fn finish(&mut self) {
        self.metrics.lost += self.groups.len() as u64;
        self.groups.clear();
    }

    fn count_duplicate(&mut self, relay: usize) {
        self.metrics.duplicates += 1;
        if let Some(stats) = self.relay_stats.get_mut(relay) {
            stats.duplicates += 1;
        }
    }

    fn remember(&mut self, key: GroupKey) {
        if self.recent.insert(key) {
            self.recent_order.push_back(key);
        }
        while self.recent_order.len() > RECENT {
            if let Some(old) = self.recent_order.pop_front() {
                self.recent.remove(&old);
            }
        }
    }

    fn evict_one(&mut self) {
        let key = self
            .groups
            .keys()
            .find(|key| key.kind.is_media())
            .or_else(|| self.groups.keys().next())
            .copied();
        if let Some(key) = key {
            self.groups.remove(&key);
            self.metrics.abandoned += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{AirConfig, AirKind, encode_object};

    fn heard(relay: u8, frame: Vec<u8>) -> Heard {
        Heard {
            relay,
            rssi_dbm: -90 - i16::from(relay),
            snr_db_x10: 80,
            frame,
        }
    }

    #[test]
    fn two_receivers_count_one_object_and_one_duplicate() {
        let frame = encode_object(
            &AirConfig {
                parity: 0,
                ..AirConfig::default()
            },
            AirKind::Telemetry,
            4,
            b"snapshot",
        )
        .unwrap()
        .remove(0);
        let mut assembler = Assembler::new();
        let first = assembler.push(heard(0, frame.clone())).unwrap();
        assert_eq!(first.payload, b"snapshot");
        assert!(assembler.push(heard(1, frame)).is_none());
        let metrics = assembler.metrics();
        assert_eq!(metrics.recovered, 1);
        assert_eq!(metrics.duplicates, 1);
        assert_eq!(metrics.relays[1].duplicates, 1);
        assert_eq!(metrics.relays[0].rssi_dbm, -90.0);
    }

    #[test]
    fn a_far_future_sequence_cannot_make_real_frames_late() {
        let one = |sequence: u32| {
            encode_object(
                &AirConfig {
                    parity: 0,
                    ..AirConfig::default()
                },
                AirKind::Telemetry,
                sequence,
                b"snapshot",
            )
            .unwrap()
            .remove(0)
        };
        let mut assembler = Assembler::new();
        assert!(assembler.push(heard(0, one(10))).is_some());
        // A CRC-valid frame claiming the last sequence number is refused...
        assert!(assembler.push(heard(0, one(u32::MAX))).is_none());
        assert_eq!(assembler.metrics().malformed, 1);
        // ...so the next real frame is still accepted, not counted late.
        assert!(assembler.push(heard(0, one(11))).is_some());
        assert_eq!(assembler.metrics().late, 0);
        // A long but plausible gap between passes is fine.
        assert!(assembler.push(heard(0, one(11 + 50_000))).is_some());
    }

    #[test]
    fn a_spoofed_first_frame_is_outlived() {
        let one = |sequence: u32| {
            encode_object(
                &AirConfig {
                    parity: 0,
                    ..AirConfig::default()
                },
                AirKind::Telemetry,
                sequence,
                b"snapshot",
            )
            .unwrap()
            .remove(0)
        };
        let mut assembler = Assembler::new();
        // The first frame heard claims a huge sequence, so there is no newest to compare with.
        assert!(assembler.push(heard(0, one(4_000_000_000))).is_some());
        // Real frames are refused as late at first...
        for sequence in 1..REANCHOR_AFTER {
            assert!(assembler.push(heard(0, one(sequence))).is_none());
        }
        assert_eq!(assembler.metrics().late, u64::from(REANCHOR_AFTER - 1));
        // ...then the run of refusals moves the window, and real frames flow again.
        assert!(assembler.push(heard(0, one(REANCHOR_AFTER))).is_some());
        assert!(assembler.push(heard(0, one(REANCHOR_AFTER + 1))).is_some());
        // Normal rules apply again: a second copy is a duplicate.
        assert!(assembler.push(heard(0, one(REANCHOR_AFTER))).is_none());
        assert!(assembler.metrics().duplicates >= 1);
    }

    #[test]
    fn a_corrupt_copy_does_not_hide_a_good_one() {
        let frame = encode_object(
            &AirConfig {
                parity: 0,
                ..AirConfig::default()
            },
            AirKind::Telemetry,
            1,
            b"snapshot",
        )
        .unwrap()
        .remove(0);
        let mut corrupt = frame.clone();
        corrupt[8] ^= 0xff;
        let mut assembler = Assembler::new();
        assert!(assembler.push(heard(0, corrupt)).is_none());
        assert_eq!(assembler.metrics().crc_failures, 1);
        assert_eq!(
            assembler.push(heard(0, frame)).unwrap().payload,
            b"snapshot"
        );
    }

    #[test]
    fn a_full_map_drops_media_before_telemetry() {
        let media = AirConfig {
            parity: 1,
            max_shard: 8,
            ..AirConfig::default()
        };
        let mut assembler = Assembler::new();
        let payload = vec![1_u8; 16];
        for sequence in 0..MAX_INCOMPLETE {
            let frames = encode_object(&media, AirKind::Image, sequence as u32, &payload).unwrap();
            // Keep only the first shard, so the image stays incomplete.
            assembler.push(heard(0, frames.into_iter().next().unwrap()));
        }
        assert_eq!(assembler.groups.len(), MAX_INCOMPLETE);
        let telemetry = encode_object(
            &AirConfig {
                parity: 0,
                ..AirConfig::default()
            },
            AirKind::Telemetry,
            1,
            b"keep-me",
        )
        .unwrap()
        .remove(0);
        assert_eq!(
            assembler.push(heard(0, telemetry)).unwrap().payload,
            b"keep-me"
        );
        assert!(assembler.metrics().abandoned >= 1);
        assert!(
            assembler
                .groups
                .keys()
                .all(|key| key.kind == AirKind::Telemetry || key.sequence != 0)
        );
    }
}
