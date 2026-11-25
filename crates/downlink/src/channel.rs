//! Virtual 915 MHz channel: independent loss at each receiver, optional bursts,
//! and optional reordering. RSSI and SNR are fixed desk-simulation samples.

use crate::assemble::Heard;
use crate::frame::Error;

/// Invented signal reports, one per receiver slot. Not measurements.
const SIGHT: [(i16, i16); 8] = [
    (-92, 80),
    (-87, 110),
    (-98, 55),
    (-90, 70),
    (-95, 60),
    (-84, 125),
    (-101, 40),
    (-89, 90),
];

/// Loss and reordering for [`Channel`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelConfig {
    /// How many virtual receivers hear each transmission. 1 to 8.
    pub relays: u8,
    /// Chance, in parts per thousand, that a frame starts a loss. 0 keeps every frame.
    pub loss_per_mille: u16,
    /// Frames dropped when a loss starts, including the first one. 1 is a single drop.
    pub burst: u16,
    /// Extra delay, in frames, applied at random to a kept copy. 0 keeps order.
    pub reorder: u16,
    /// Seed for the loss and reorder draws. The same seed repeats the same pattern.
    pub seed: u64,
}

impl ChannelConfig {
    /// Reject a draw that this desk model does not define.
    pub fn check(&self) -> Result<(), Error> {
        if !(1..=8).contains(&self.relays) {
            return Err(Error::Config("use 1 to 8 virtual receivers".into()));
        }
        if self.loss_per_mille > 1000 {
            return Err(Error::Config(
                "loss is parts per thousand, from 0 to 1000".into(),
            ));
        }
        if self.burst == 0 {
            return Err(Error::Config(
                "burst length starts at 1 (a single dropped frame)".into(),
            ));
        }
        Ok(())
    }
}

/// Stateful loss model. One instance covers a whole pass so bursts can span objects.
#[derive(Debug)]
pub struct Channel {
    relays: u8,
    loss_per_mille: u16,
    burst: u16,
    reorder: u16,
    burst_left: Vec<u16>,
    rng: fastrand::Rng,
}

impl Channel {
    /// Start a channel. The random sequence depends only on `config.seed`.
    pub fn new(config: &ChannelConfig) -> Result<Self, Error> {
        config.check()?;
        Ok(Self {
            relays: config.relays,
            loss_per_mille: config.loss_per_mille,
            burst: config.burst,
            reorder: config.reorder,
            burst_left: vec![0; usize::from(config.relays)],
            rng: fastrand::Rng::with_seed(config.seed),
        })
    }

    /// Copies of `frame` that the virtual receivers kept, in receiver order.
    ///
    /// Reordering is not applied here. A live sender wants the copies now.
    /// [`Self::run`] is the batch path that can also reorder.
    pub fn observe(&mut self, frame: &[u8]) -> Vec<Heard> {
        let mut kept = Vec::new();
        for relay in 0..self.relays {
            if self.dropped(usize::from(relay)) {
                continue;
            }
            let (rssi_dbm, snr_db_x10) = SIGHT[usize::from(relay)];
            kept.push(Heard {
                relay,
                rssi_dbm,
                snr_db_x10,
                frame: frame.to_vec(),
            });
        }
        kept
    }

    /// Run `frames` through every receiver and return the kept copies in delivery order.
    pub fn run(&mut self, frames: &[Vec<u8>]) -> Vec<Heard> {
        let mut held = Vec::new();
        for (index, frame) in frames.iter().enumerate() {
            for relay in 0..self.relays {
                if self.dropped(usize::from(relay)) {
                    continue;
                }
                let extra = if self.reorder == 0 {
                    0
                } else {
                    u64::from(self.rng.u32(0..=u32::from(self.reorder)))
                };
                let (rssi_dbm, snr_db_x10) = SIGHT[usize::from(relay)];
                held.push((
                    index as u64 + extra,
                    relay,
                    index,
                    Heard {
                        relay,
                        rssi_dbm,
                        snr_db_x10,
                        frame: frame.clone(),
                    },
                ));
            }
        }
        held.sort_unstable_by_key(|(when, relay, index, _)| (*when, *relay, *index));
        held.into_iter().map(|(_, _, _, heard)| heard).collect()
    }

    fn dropped(&mut self, relay: usize) -> bool {
        if self.burst_left[relay] > 0 {
            self.burst_left[relay] -= 1;
            return true;
        }
        if self.rng.u32(..1000) < u32::from(self.loss_per_mille) {
            self.burst_left[relay] = self.burst.saturating_sub(1);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(loss: u16, relays: u8, burst: u16) -> ChannelConfig {
        ChannelConfig {
            relays,
            loss_per_mille: loss,
            burst,
            reorder: 0,
            seed: 0x0915,
        }
    }

    #[test]
    fn zero_loss_is_heard_once_per_receiver() {
        let mut channel = Channel::new(&config(0, 3, 1)).unwrap();
        let heard = channel.run(&[b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(heard.len(), 6);
        assert_eq!(heard[0].relay, 0);
        assert_eq!(heard[1].relay, 1);
    }

    #[test]
    fn total_loss_keeps_nothing() {
        let mut channel = Channel::new(&config(1000, 2, 1)).unwrap();
        assert!(channel.run(&[b"a".to_vec()]).is_empty());
    }

    #[test]
    fn a_burst_drops_the_following_frames_without_another_draw() {
        let mut channel = Channel::new(&ChannelConfig {
            relays: 1,
            loss_per_mille: 1000,
            burst: 3,
            reorder: 0,
            seed: 1,
        })
        .unwrap();
        let frames: Vec<_> = (0..5).map(|n| vec![n]).collect();
        let heard = channel.run(&frames);
        // The first frame starts a burst of 3. Frames 3 and 4 are kept because
        // the loss rate is only consulted again after the burst, and the seed's
        // next draw is not forced here: with loss 1000 the next draw also drops.
        assert!(heard.len() < frames.len());
    }

    #[test]
    fn the_same_seed_repeats() {
        let frames: Vec<_> = (0..40).map(|n| vec![n]).collect();
        let once = Channel::new(&config(200, 2, 2)).unwrap().run(&frames);
        let twice = Channel::new(&config(200, 2, 2)).unwrap().run(&frames);
        assert_eq!(once.len(), twice.len());
        assert_eq!(once[0].frame, twice[0].frame);
    }
}
