//! ADF4351 local-oscillator settings for the CSC-R1P receiver.
//! Register fields and initialization order follow Figures 24–29 and the
//! worked example in the ADF4351 datasheet, Rev. A.

pub const TWO_METERS_KHZ: core::ops::RangeInclusive<u32> = 144_000..=146_000;
pub const SEVENTY_CM_KHZ: core::ops::RangeInclusive<u32> = 435_000..=438_000;

const PFD_HZ: u32 = 25_000_000;
const MODULUS: u32 = 3125; // 8 kHz VCO grid, an exact 1 kHz output grid on both bands
const R2: u32 = (6 << 26) // digital lock detect on MUXOUT
    | (1 << 14) // reference divider R = 1
    | (1 << 13) // double-buffer the RF divider
    | (7 << 9) // 2.5 mA charge pump, as modeled for this board
    | (1 << 6) // positive phase-detector polarity
    | 2;

pub const POWER_DOWN: u32 = R2 | (1 << 5);

#[derive(Clone, Copy)]
pub enum BenchBand {
    TwoMeters,
    SeventyCentimeters,
}

impl BenchBand {
    pub fn channel(self) -> ReceiveChannel {
        let khz = match self {
            Self::TwoMeters => 145_800,
            Self::SeventyCentimeters => 437_000,
        };
        ReceiveChannel::from_khz(khz).expect("bench preset is in range")
    }
}

#[derive(Clone, Copy)]
pub struct ReceiveChannel {
    hz: u32,
    divider_code: u32,
}

impl ReceiveChannel {
    /// Select one of the supported receiver channels on the whole-kHz grid.
    pub fn from_khz(khz: u32) -> Option<Self> {
        let divider_code = if TWO_METERS_KHZ.contains(&khz) {
            4 // divide by 16
        } else if SEVENTY_CM_KHZ.contains(&khz) {
            3 // divide by 8
        } else {
            return None;
        };
        Some(Self {
            hz: khz * 1_000,
            divider_code,
        })
    }

    pub fn frequency_hz(self) -> u32 {
        self.hz
    }

    /// Six words in the order required at startup (R5 through R0).
    pub fn registers(self) -> [u32; 6] {
        let vco_hz = self.hz << self.divider_code;
        let integer = vco_hz / PFD_HZ;
        let fraction = (vco_hz % PFD_HZ) / (PFD_HZ / MODULUS);

        let r5 = (0b11 << 19) | 5; // reserved bits, dedicated LD pin unused
        let r4 = (1 << 23) // feedback from the VCO
            | (self.divider_code << 20)
            | (200 << 12) // 125 kHz band-select clock
            | (1 << 10) // mute until digital lock
            | (1 << 8) // divided auxiliary output for LO MON
            | (1 << 5) // main output feeds the on-board mixer
            | 4; // both output power fields are 0 (lowest setting)
        // Match digital lock-detect timing and the antibacklash pulse to the divider mode.
        // Fast lock, charge cancellation and phase resync remain disabled.
        let (r3, r2) = if fraction == 0 {
            (3 | (1 << 22), R2 | (1 << 8) | (1 << 7))
        } else {
            (3, R2)
        };
        let r1 = (1 << 27) | (1 << 15) | (MODULUS << 3) | 1; // 8/9, phase 1
        let r0 = (integer << 15) | (fraction << 3);
        [r5, r4, r3, r2, r1, r0]
    }
}

/// Require three consecutive high samples within the caller's bounded window.
pub fn has_stable_lock(samples: impl IntoIterator<Item = bool>) -> bool {
    let mut high = 0;
    for sample in samples {
        high = if sample { high + 1 } else { 0 };
        if high == 3 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_profiles_use_exact_dividers_and_fractional_frequencies() {
        for (band, expected_hz, expected_divider, expected_n, expected_fraction) in [
            (BenchBand::TwoMeters, 145_800_000, 16, 93, 975),
            (BenchBand::SeventyCentimeters, 437_000_000, 8, 139, 2625),
        ] {
            let words = band.channel().registers();
            assert_eq!(words.map(|word| word & 7), [5, 4, 3, 2, 1, 0]);
            let [_, r4, _, r2, r1, r0] = words;
            let divider = 1_u64 << ((r4 >> 20) & 7);
            let integer = u64::from((r0 >> 15) & 0xffff);
            let fraction = u64::from((r0 >> 3) & 0xfff);
            let modulus = u64::from((r1 >> 3) & 0xfff);
            assert_eq!(
                (divider, integer, fraction, modulus),
                (expected_divider, expected_n, expected_fraction, 3125)
            );
            assert_eq!(
                (integer * modulus + fraction) * 25_000_000,
                expected_hz * modulus * divider
            );
            assert_eq!(r2 >> 26 & 7, 6);
            assert_eq!(r2 >> 9 & 15, 7);
            assert_eq!(r4 >> 12 & 255, 200);
            assert_eq!(r4 >> 10 & 1, 1);
            assert_eq!(r4 >> 9 & 1, 0);
            assert_eq!((r4 >> 8 & 1, r4 >> 5 & 1), (1, 1));
            assert_eq!(POWER_DOWN ^ r2, 1 << 5);
        }
    }

    #[test]
    fn covers_both_windows_and_rejects_outside_requests() {
        for khz in [144_000, 145_797, 146_000, 435_000, 437_003, 438_000] {
            let channel = ReceiveChannel::from_khz(khz).unwrap();
            let [_, r4, _, _, r1, r0] = channel.registers();
            let divider = 1_u64 << ((r4 >> 20) & 7);
            let integer = u64::from((r0 >> 15) & 0xffff);
            let fraction = u64::from((r0 >> 3) & 0xfff);
            let modulus = u64::from((r1 >> 3) & 0xfff);
            assert_eq!(
                (integer * modulus + fraction) * 25_000_000,
                u64::from(khz) * 1000 * modulus * divider
            );
        }
        for rejected in [0, 143_999, 146_001, 434_999, 438_001, u32::MAX] {
            assert!(ReceiveChannel::from_khz(rejected).is_none());
        }
    }

    #[test]
    fn integer_boundary_uses_integer_lock_timing_and_wraps_the_fraction() {
        let words = |khz| ReceiveChannel::from_khz(khz).unwrap().registers();
        let [_, _, before_r3, before_r2, _, before_r0] = words(437_499);
        let [_, _, exact_r3, exact_r2, _, exact_r0] = words(437_500);
        assert_eq!(before_r0 >> 15, 139);
        assert_eq!((before_r0 >> 3) & 0xfff, 3124);
        assert_eq!(exact_r0, 140 << 15);
        assert_eq!(before_r3 & (1 << 22), 0);
        assert_eq!(exact_r3 & (1 << 22), 1 << 22);
        assert_eq!(before_r2 & (3 << 7), 0);
        assert_eq!(exact_r2 & (3 << 7), 3 << 7);
    }

    #[test]
    fn lock_requires_three_consecutive_high_samples() {
        assert!(!has_stable_lock([true, true, false, true, true]));
        assert!(has_stable_lock([false, true, true, true]));
        assert!(!has_stable_lock([false; 100]));
    }
}
