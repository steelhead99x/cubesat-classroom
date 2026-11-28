//! Two ADF4351 local-oscillator presets for the CSC-R1P bench.
//! Register fields and initialization order follow Figures 24–29 and the
//! worked example in the ADF4351 datasheet, Rev. A.

const PFD_HZ: u32 = 25_000_000;
const MODULUS: u32 = 125;
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
    pub fn frequency_hz(self) -> u32 {
        match self {
            Self::TwoMeters => 145_800_000,
            Self::SeventyCentimeters => 437_000_000,
        }
    }

    /// Six words in the order required at startup (R5 through R0).
    pub fn registers(self) -> [u32; 6] {
        let divider_code = match self {
            Self::TwoMeters => 4,          // divide by 16
            Self::SeventyCentimeters => 3, // divide by 8
        };
        let vco_hz = self.frequency_hz() << divider_code;
        let integer = vco_hz / PFD_HZ;
        let fraction = (vco_hz % PFD_HZ) / (PFD_HZ / MODULUS);

        let r5 = (0b11 << 19) | 5; // reserved bits, dedicated LD pin unused
        let r4 = (1 << 23) // feedback from the VCO
            | (divider_code << 20)
            | (200 << 12) // 125 kHz band-select clock
            | (1 << 10) // mute until digital lock
            | (1 << 8) // divided auxiliary output for LO MON
            | (1 << 5) // main output feeds the on-board mixer
            | 4; // both output power fields are 0 (lowest setting)
        let r3 = 3; // fractional-N timing, no fast lock or phase resync
        let r1 = (1 << 27) | (1 << 15) | (MODULUS << 3) | 1; // 8/9, phase 1
        let r0 = (integer << 15) | (fraction << 3);
        [r5, r4, r3, R2, r1, r0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_profiles_use_exact_dividers_and_fractional_frequencies() {
        for (band, expected_hz, expected_divider, expected_n, expected_fraction) in [
            (BenchBand::TwoMeters, 145_800_000, 16, 93, 39),
            (BenchBand::SeventyCentimeters, 437_000_000, 8, 139, 105),
        ] {
            let words = band.registers();
            assert_eq!(words.map(|word| word & 7), [5, 4, 3, 2, 1, 0]);
            let [_, r4, _, r2, r1, r0] = words;
            let divider = 1_u64 << ((r4 >> 20) & 7);
            let integer = u64::from((r0 >> 15) & 0xffff);
            let fraction = u64::from((r0 >> 3) & 0xfff);
            let modulus = u64::from((r1 >> 3) & 0xfff);
            assert_eq!(
                (divider, integer, fraction, modulus),
                (expected_divider, expected_n, expected_fraction, 125)
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
}
