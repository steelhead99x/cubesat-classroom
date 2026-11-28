//! Panel tuning logic shared by the bench test and the eventual listening mode.

use crate::{
    Controls, encoder_state, encoder_transition,
    pll::{BenchBand, SEVENTY_CM_KHZ, TWO_METERS_KHZ},
};

const DEBOUNCE_SAMPLES: u8 = 5;
const EDGES_PER_DETENT: i8 = 4;

struct Debounced {
    stable: bool,
    changed_samples: u8,
}

impl Debounced {
    fn new(stable: bool) -> Self {
        Self {
            stable,
            changed_samples: 0,
        }
    }

    /// Return the new state once a transition has been stable long enough.
    fn sample(&mut self, raw: bool) -> Option<bool> {
        if raw == self.stable {
            self.changed_samples = 0;
            return None;
        }
        self.changed_samples += 1;
        if self.changed_samples < DEBOUNCE_SAMPLES {
            return None;
        }
        self.stable = raw;
        self.changed_samples = 0;
        Some(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TuningStatus {
    pub frequency_khz: u32,
    pub step_khz: u32,
    pub seventy_cm: bool,
    /// A latched panel request. PLL lock and output state are verified separately.
    pub engaged_requested: bool,
}

pub struct PanelTuning {
    status: TuningStatus,
    band: Debounced,
    encoder_switch: Debounced,
    foot_switch: Debounced,
    encoder_state: u8,
    encoder_edges: i8,
}

impl PanelTuning {
    pub fn new(input: Controls) -> Self {
        Self {
            status: TuningStatus {
                frequency_khz: preset_khz(input.band_70cm),
                step_khz: 5,
                seventy_cm: input.band_70cm,
                engaged_requested: false,
            },
            band: Debounced::new(input.band_70cm),
            encoder_switch: Debounced::new(input.encoder_pressed),
            foot_switch: Debounced::new(input.foot_pressed),
            encoder_state: encoder_state(input.encoder_a, input.encoder_b),
            encoder_edges: 0,
        }
    }

    pub fn status(&self) -> TuningStatus {
        self.status
    }

    /// Apply one poll of the panel. The bench default is one poll every 2 ms.
    pub fn sample(&mut self, input: Controls) {
        if self.encoder_switch.sample(input.encoder_pressed) == Some(true) {
            self.status.step_khz = if self.status.step_khz == 5 { 1 } else { 5 };
        }
        if self.foot_switch.sample(input.foot_pressed) == Some(true) {
            self.status.engaged_requested = !self.status.engaged_requested;
        }

        let previous = self.encoder_state;
        self.encoder_state = encoder_state(input.encoder_a, input.encoder_b);
        if let Some(seventy_cm) = self.band.sample(input.band_70cm) {
            self.status.seventy_cm = seventy_cm;
            self.status.frequency_khz = preset_khz(seventy_cm);
            self.encoder_edges = 0;
            return;
        }
        match encoder_transition(previous, self.encoder_state) {
            1 => self.encoder_edges += 1,
            2 => self.encoder_edges = 0,
            3 => self.encoder_edges -= 1,
            _ => {}
        }
        if self.encoder_edges.abs() == EDGES_PER_DETENT {
            let step = self.status.step_khz as i32 * i32::from(self.encoder_edges.signum());
            let band = if self.status.seventy_cm {
                SEVENTY_CM_KHZ
            } else {
                TWO_METERS_KHZ
            };
            self.status.frequency_khz = self
                .status
                .frequency_khz
                .saturating_add_signed(step)
                .clamp(*band.start(), *band.end());
            self.encoder_edges = 0;
        }
    }
}

fn preset_khz(seventy_cm: bool) -> u32 {
    let band = if seventy_cm {
        BenchBand::SeventyCentimeters
    } else {
        BenchBand::TwoMeters
    };
    band.channel().frequency_hz() / 1_000
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel(a: bool, b: bool, tune: bool, foot: bool, high: bool) -> Controls {
        Controls {
            encoder_a: a,
            encoder_b: b,
            encoder_pressed: tune,
            foot_pressed: foot,
            band_70cm: high,
        }
    }

    #[test]
    fn bounce_and_simultaneous_encoder_changes_do_not_tune() {
        let mut tuner = PanelTuning::new(panel(false, false, false, false, false));
        for (a, b) in [(false, true), (false, false), (true, true), (false, false)] {
            tuner.sample(panel(a, b, false, false, false));
        }
        assert_eq!(tuner.status().frequency_khz, 145_800);
        for (a, b) in [(false, true), (true, true), (true, false), (false, false)] {
            tuner.sample(panel(a, b, false, false, false));
        }
        assert_eq!(tuner.status().frequency_khz, 145_805);
        for (a, b) in [(true, false), (true, true), (false, true), (false, false)] {
            tuner.sample(panel(a, b, false, false, false));
        }
        assert_eq!(tuner.status().frequency_khz, 145_800);
    }

    #[test]
    fn switches_debounce_once_and_tune_wrap_is_blocked() {
        let mut tuner = PanelTuning::new(panel(false, false, false, false, false));
        for _ in 0..(DEBOUNCE_SAMPLES - 1) {
            tuner.sample(panel(false, false, true, true, true));
        }
        assert_eq!(tuner.status().step_khz, 5);
        assert!(!tuner.status().engaged_requested);
        assert!(!tuner.status().seventy_cm);
        for _ in 0..20 {
            tuner.sample(panel(false, false, true, true, true));
        }
        assert_eq!(
            tuner.status(),
            TuningStatus {
                frequency_khz: 437_000,
                step_khz: 1,
                seventy_cm: true,
                engaged_requested: true
            }
        );
        tuner.status.frequency_khz = 438_000;
        for (a, b) in [(false, true), (true, true), (true, false), (false, false)] {
            tuner.sample(panel(a, b, true, true, true));
        }
        assert_eq!(tuner.status().frequency_khz, 438_000);
    }
}
