//! Desk-verified narrow FM demodulation for the pedal's 192 kHz I/Q stream.

pub const SAMPLE_RATE: u32 = 192_000;
const WORD_SCALE: f32 = 1.0 / 2_147_483_648.0;
const QUIET_POWER: f32 = 1.0e-8;

#[derive(Default)]
struct Filter {
    z1: f32,
    z2: f32,
}

impl Filter {
    fn low_pass(&mut self, sample: f32, [b0, b1, a1, a2]: [f32; 4]) -> f32 {
        let filtered = sample * b0 + self.z1;
        self.z1 = sample * b1 + self.z2 - a1 * filtered;
        self.z2 = sample * b0 - a2 * filtered;
        filtered
    }
}

// Second-order Butterworth sections at 9 kHz (complex channel) and 3 kHz
// (audio). Written as [b0, b1, a1, a2] with b2 = b0.
const CHANNEL: [f32; 4] = [0.017_863_193, 0.035_726_387, -1.587_937_1, 0.659_389_85];
const VOICE: [f32; 4] = [0.002_251_583, 0.004_503_165, -1.861_361_1, 0.870_367_47];

#[derive(Default)]
pub struct NarrowFm {
    channel_i: Filter,
    channel_q: Filter,
    previous_iq: Option<(f32, f32)>,
    dc: f32,
    voice: Filter,
}

impl NarrowFm {
    /// Convert one signed 32-bit I/Q frame to a signed mono DAC word.
    pub fn process(&mut self, i: i32, q: i32) -> i32 {
        let i = self.channel_i.low_pass(i as f32 * WORD_SCALE, CHANNEL);
        let q = self.channel_q.low_pass(q as f32 * WORD_SCALE, CHANNEL);
        let phase = if i * i + q * q >= QUIET_POWER {
            self.previous_iq
                .replace((i, q))
                .map_or(0.0, |(prev_i, prev_q)| {
                    let cross = prev_i * q - prev_q * i;
                    let dot = prev_i * i + prev_q * q;
                    // Inside the selected channel the adjacent-sample phase is
                    // small. Reject out-of-band/ambiguous steps rather than wrap them.
                    if dot > 0.0 && cross.abs() <= dot {
                        let tangent = cross / dot;
                        tangent / (1.0 + 0.28 * tangent * tangent)
                    } else {
                        0.0
                    }
                })
        } else {
            self.previous_iq = None;
            0.0
        };

        // Remove carrier offset below about 150 Hz before filtering speech.
        self.dc += 0.004_897 * (phase - self.dc);
        let speech = self.voice.low_pass(phase - self.dc, VOICE);
        // Normalize around ±5 kHz deviation with 12 dB DAC headroom.
        let level = (speech * 6.111_55).clamp(-1.0, 1.0);
        (level * 536_870_912.0) as i32
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::f32::consts::TAU;

    fn demodulated_tone(audio_hz: f32) -> (f32, f32) {
        let mut demod = NarrowFm::default();
        let rate = SAMPLE_RATE as f32;
        let mut phase = 0.0;
        let mut sum = 0.0;
        let mut energy = 0.0;
        let mut correlation = 0.0;
        let count = 19_200;
        for n in 0..count * 2 {
            let t = n as f32 / rate;
            let tone = (TAU * audio_hz * t).cos();
            phase += TAU * (900.0 + 2_500.0 * tone) / rate;
            let scale = 1_000_000_000.0;
            let value = demod.process((scale * phase.cos()) as i32, (scale * phase.sin()) as i32)
                as f32
                / 536_870_912.0;
            if n >= count {
                sum += value;
                energy += value * value;
                correlation += value * tone;
            }
        }
        let count = count as f32;
        assert!(
            (sum / count).abs() < 0.001,
            "carrier offset leaked into audio"
        );
        ((energy / count).sqrt(), correlation / count)
    }

    #[test]
    fn recovers_voice_and_attenuates_out_of_band_audio() {
        let (speech, correlation) = demodulated_tone(1_000.0);
        let (high, _) = demodulated_tone(12_000.0);
        assert!(speech > 0.32 && speech < 0.37);
        assert!(correlation > 0.17, "unexpected sign or phase");
        assert!(high < 0.04 * speech);
    }

    #[test]
    fn silence_and_static_iq_do_not_make_audio() {
        let mut fm = NarrowFm::default();
        for _ in 0..1_000 {
            assert_eq!(fm.process(0, 0), 0);
            assert_eq!(fm.process(i32::MAX / 2, 0), 0);
        }
    }
}
