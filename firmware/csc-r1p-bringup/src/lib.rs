#![no_std]

pub mod fm;
pub mod pll;

use embedded_hal::i2c::I2c;

const RF: u8 = 0x20;
const CONTROL: u8 = 0x21;
const OLED: u8 = 0x3c;
const POTS: u8 = 0x48;
const AUDIO_ADC: u8 = 0x4c;

const INPUT: u8 = 0;
const OUTPUT: u8 = 1;
const DIRECTION: u8 = 3;
const STATUS_LED: u8 = 1 << 5;
const BAND_70CM: u8 = 1 << 0;
const DEMOD_EN: u8 = 1 << 2;
const PLL_CE: u8 = 1 << 6;
const ADC_SHDNZ: u8 = 1 << 5;

pub struct Pedal<Bus>(pub Bus);

#[derive(Clone, Copy)]
pub enum Pot {
    Gain,
    Level,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Controls {
    pub encoder_a: bool,
    pub encoder_b: bool,
    pub encoder_pressed: bool,
    pub foot_pressed: bool,
    pub band_70cm: bool,
}

/// Counts quadrature edges without assuming a particular pulses-per-detent encoder.
pub struct EncoderTrace {
    previous: u8,
    pub forward_edges: u32,
    pub reverse_edges: u32,
    pub missed_samples: u32,
}

impl EncoderTrace {
    pub fn new(a: bool, b: bool) -> Self {
        Self {
            previous: (u8::from(a) << 1) | u8::from(b),
            forward_edges: 0,
            reverse_edges: 0,
            missed_samples: 0,
        }
    }

    pub fn sample(&mut self, a: bool, b: bool) {
        // Phase order 00 → 01 → 11 → 10. The bench test establishes
        // whether this is clockwise for the fitted encoder.
        const PHASE_INDEX: [u8; 4] = [0, 1, 3, 2];
        let current = (u8::from(a) << 1) | u8::from(b);
        let delta = (PHASE_INDEX[current as usize] + 4 - PHASE_INDEX[self.previous as usize]) % 4;
        match delta {
            1 => self.forward_edges += 1,
            2 => self.missed_samples += 1,
            3 => self.reverse_edges += 1,
            _ => {}
        }
        self.previous = current;
    }
}

impl<Bus: I2c> Pedal<Bus> {
    /// Set the latch before taking control of the pins. Keep the bias tee,
    /// demodulator, ADC, PLL and audio disabled.
    pub fn enter_safe(&mut self) -> Result<(), Bus::Error> {
        self.0.write(RF, &[OUTPUT, 0])?;
        self.0.write(RF, &[DIRECTION, 0])
    }

    /// Change CE only in bench mode, after enter_safe has configured the port.
    pub fn pll_enable(&mut self, enabled: bool) -> Result<(), Bus::Error> {
        self.0
            .write(RF, &[OUTPUT, if enabled { PLL_CE } else { 0 }])
    }

    /// Release only the ADC shutdown pin after enter_safe has configured the port.
    pub fn adc_release_shutdown(&mut self, released: bool) -> Result<(), Bus::Error> {
        self.0
            .write(RF, &[OUTPUT, if released { ADC_SHDNZ } else { 0 }])
    }

    /// Enable the selected passive receive path after the bench PLL test
    /// has acquired lock. Keep antenna bias, trims and DAC unmute low.
    pub fn enable_muted_receiver(&mut self, seventy_cm: bool) -> Result<(), Bus::Error> {
        let band = if seventy_cm { BAND_70CM } else { 0 };
        self.0
            .write(RF, &[OUTPUT, PLL_CE | DEMOD_EN | ADC_SHDNZ | band])
    }

    /// Read TLV320ADC3140 DEV_STS0/1 on the default page after hardware reset.
    pub fn adc_status(&mut self) -> Result<[u8; 2], Bus::Error> {
        let mut status = [0; 2];
        self.0.write_read(AUDIO_ADC, &[0x76], &mut status[..1])?;
        self.0.write_read(AUDIO_ADC, &[0x77], &mut status[1..])?;
        Ok(status)
    }

    pub fn adc_wake_for_clocks(&mut self) -> Result<(), Bus::Error> {
        self.0.write(AUDIO_ADC, &[0x02, 0x81]) // exit sleep with internal AREG
    }

    /// Configure the on-board ADC as controller. Leave DAC mute and demodulator off.
    pub fn adc_start_clocks(&mut self) -> Result<(), Bus::Error> {
        for command in [
            [0x21, 0xa0], // GPIO1 is the 24.576 MHz MCLK input
            [0x13, 0x87], // controller mode, automatic clocks with PLL, 48 kHz family
            [0x14, 0x64], // 192 kHz FSYNC; BCLK = 64 × FSYNC
            [0x07, 0xbe], // 32-bit slots, TI SBAA382 §3.2 standard I²S edge alignment
            [0x0b, 0x00], // I in left slot zero
            [0x0c, 0x20], // Q in right slot zero
            [0x73, 0xc0], // input channels 1 and 2
            [0x74, 0xc0], // enable their ASI output
            [0x75, 0x60], // power ADC and PLL; MICBIAS off
        ] {
            self.0.write(AUDIO_ADC, &command)?;
        }
        Ok(())
    }

    pub fn adc_clock_status(&mut self) -> Result<u8, Bus::Error> {
        let mut status = [0];
        self.0.write_read(AUDIO_ADC, &[0x15], &mut status)?;
        Ok(status[0])
    }

    pub fn init_controls(&mut self) -> Result<(), Bus::Error> {
        self.0.write(CONTROL, &[OUTPUT, STATUS_LED])?;
        self.0.write(CONTROL, &[DIRECTION, !STATUS_LED])
    }

    pub fn led(&mut self, on: bool) -> Result<(), Bus::Error> {
        self.0
            .write(CONTROL, &[OUTPUT, if on { 0 } else { STATUS_LED }])
    }

    pub fn controls(&mut self) -> Result<Controls, Bus::Error> {
        let mut input = [0];
        self.0.write_read(CONTROL, &[INPUT], &mut input)?;
        let low = |bit: u8| input[0] & (1u8 << bit) == 0;
        Ok(Controls {
            encoder_a: low(0),
            encoder_b: low(1),
            encoder_pressed: low(2),
            foot_pressed: low(3),
            band_70cm: low(4),
        })
    }

    /// Draw a border on the specified 128×32 SSD1306 module.
    pub fn oled_test(&mut self) -> Result<(), Bus::Error> {
        self.0.write(
            OLED,
            &[
                0x00, // I²C command stream
                0xae, // display off while loading RAM
                0x2e, // stop scrolling
                0xa8, 31, // 32 rows
                0xd3, 0, 0x40, // no display offset, start at line 0
                0xa1, 0xc8, // segment and COM mapping
                0xda, 0x02, // sequential COM pins
                0x8d, 0x14, // internal charge pump on
                0xa4, 0xa6, // display RAM, normal polarity
                0x20, 0, // horizontal addressing
                0x21, 0, 127, // columns 0–127
                0x22, 0, 3, // four pages of eight rows
            ],
        )?;
        for edge in [0x01, 0x00, 0x00, 0x80] {
            let mut page = [edge; 129];
            page[0] = 0x40; // I²C display data stream
            page[1] = 0xff;
            page[128] = 0xff;
            self.0.write(OLED, &page)?;
        }
        self.0.write(OLED, &[0x00, 0xaf])
    }

    pub fn oled_off(&mut self) -> Result<(), Bus::Error> {
        self.0.write(OLED, &[0x00, 0xae, 0x8d, 0x10])
    }

    pub fn start_pot(&mut self, pot: Pot) -> Result<(), Bus::Error> {
        // AIN0 or AIN1 against ground, ±4.096 V, single shot at 1600 SPS.
        let config = match pot {
            Pot::Gain => 0xc3,
            Pot::Level => 0xd3,
        };
        self.0.write(POTS, &[1, config, 0x83])
    }

    pub fn poll_pot(&mut self) -> Result<Option<i16>, Bus::Error> {
        let mut register = [0; 2];
        self.0.write_read(POTS, &[1], &mut register)?;
        if register[0] & 0x80 == 0 {
            return Ok(None);
        }
        self.0.write_read(POTS, &[0], &mut register)?;
        Ok(Some(i16::from_be_bytes(register) >> 4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::i2c::{ErrorKind, ErrorType, Operation};

    enum Step {
        Write(u8, &'static [u8], bool),
        Read(u8, u8, &'static [u8]),
    }

    struct Mock<'a>(&'a [Step]);

    impl ErrorType for Mock<'_> {
        type Error = ErrorKind;
    }

    impl I2c for Mock<'_> {
        fn transaction(
            &mut self,
            addr: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            let (step, rest) = self.0.split_first().unwrap();
            self.0 = rest;
            match (step, operations) {
                (Step::Write(expected, data, fail), [Operation::Write(actual)]) => {
                    assert_eq!((addr, *actual), (*expected, *data));
                    if *fail {
                        return Err(ErrorKind::Other);
                    }
                }
                (
                    Step::Read(expected, reg, data),
                    [Operation::Write(register), Operation::Read(out)],
                ) => {
                    assert_eq!((addr, *register), (*expected, [*reg].as_slice()));
                    out.copy_from_slice(data);
                }
                _ => panic!("unexpected I2C transaction"),
            }
            Ok(())
        }
    }

    #[test]
    fn preloads_safe_latches_before_enabling_outputs() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x20, &[1, 0], false),
            Step::Write(0x20, &[3, 0], false),
            Step::Write(0x21, &[1, 0x20], false),
            Step::Write(0x21, &[3, 0xdf], false),
            Step::Write(0x21, &[1, 0], false),
            Step::Write(0x21, &[1, 0x20], false),
            Step::Read(0x21, 0, &[0xea]),
        ]));
        pedal.enter_safe().unwrap();
        pedal.init_controls().unwrap();
        pedal.led(true).unwrap();
        pedal.led(false).unwrap();
        assert_eq!(
            pedal.controls().unwrap(),
            Controls {
                encoder_a: true,
                encoder_b: false,
                encoder_pressed: true,
                foot_pressed: false,
                band_70cm: true,
            }
        );
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn traces_encoder_in_both_directions_and_resyncs_after_a_missed_edge() {
        let mut trace = EncoderTrace::new(false, false);
        for (a, b) in [(false, true), (true, true), (true, false), (false, false)] {
            trace.sample(a, b);
        }
        trace.sample(true, false);
        trace.sample(true, false); // a held position adds no edge
        trace.sample(false, true); // two bits changed between samples
        trace.sample(false, false); // reverse from the resynchronized phase
        assert_eq!(
            (
                trace.forward_edges,
                trace.reverse_edges,
                trace.missed_samples
            ),
            (4, 2, 1)
        );
    }

    #[test]
    fn checks_adc_control_without_enabling_any_other_rf_stage() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x20, &[1, 0x20], false),
            Step::Read(0x4c, 0x76, &[0x00]),
            Step::Read(0x4c, 0x77, &[0x00]),
            Step::Write(0x20, &[1, 0], false),
        ]));
        pedal.adc_release_shutdown(true).unwrap();
        assert_eq!(pedal.adc_status().unwrap(), [0, 0]);
        pedal.adc_release_shutdown(false).unwrap();
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn configures_adc_clock_controller_before_powering_the_channels() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x4c, &[0x02, 0x81], false),
            Step::Write(0x4c, &[0x21, 0xa0], false),
            Step::Write(0x4c, &[0x13, 0x87], false),
            Step::Write(0x4c, &[0x14, 0x64], false),
            Step::Write(0x4c, &[0x07, 0xbe], false),
            Step::Write(0x4c, &[0x0b, 0x00], false),
            Step::Write(0x4c, &[0x0c, 0x20], false),
            Step::Write(0x4c, &[0x73, 0xc0], false),
            Step::Write(0x4c, &[0x74, 0xc0], false),
            Step::Write(0x4c, &[0x75, 0x60], false),
            Step::Read(0x4c, 0x15, &[0x64]),
        ]));
        pedal.adc_wake_for_clocks().unwrap();
        pedal.adc_start_clocks().unwrap();
        assert_eq!(pedal.adc_clock_status().unwrap(), 0x64);
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn incomplete_adc_clock_setup_never_reaches_power_enable() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x4c, &[0x21, 0xa0], false),
            Step::Write(0x4c, &[0x13, 0x87], true),
        ]));
        assert_eq!(pedal.adc_start_clocks(), Err(ErrorKind::Other));
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn muted_receive_selects_the_band_and_disables_bias_and_audio() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x20, &[1, 0x64], false),
            Step::Write(0x20, &[1, 0x65], false),
            Step::Write(0x20, &[1, 0], false),
        ]));
        pedal.enable_muted_receiver(false).unwrap();
        pedal.enable_muted_receiver(true).unwrap();
        pedal.pll_enable(false).unwrap();
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn enabling_the_pll_keeps_every_other_rf_control_low() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x20, &[1, 0x40], false),
            Step::Write(0x20, &[1, 0], false),
        ]));
        pedal.pll_enable(true).unwrap();
        pedal.pll_enable(false).unwrap();
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn a_latch_error_stops_direction_changes() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x20, &[1, 0], true),
            Step::Write(0x21, &[1, 0x20], true),
        ]));
        assert_eq!(pedal.enter_safe(), Err(ErrorKind::Other));
        assert_eq!(pedal.init_controls(), Err(ErrorKind::Other));
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn oled_loads_a_128_by_32_border_before_enabling_display() {
        const fn page(edge: u8) -> [u8; 129] {
            let mut bytes = [edge; 129];
            bytes[0] = 0x40;
            bytes[1] = 0xff;
            bytes[128] = 0xff;
            bytes
        }
        const TOP: [u8; 129] = page(0x01);
        const MIDDLE: [u8; 129] = page(0x00);
        const BOTTOM: [u8; 129] = page(0x80);
        let mut pedal = Pedal(Mock(&[
            Step::Write(
                0x3c,
                &[
                    0, 0xae, 0x2e, 0xa8, 31, 0xd3, 0, 0x40, 0xa1, 0xc8, 0xda, 2, 0x8d, 0x14, 0xa4,
                    0xa6, 0x20, 0, 0x21, 0, 127, 0x22, 0, 3,
                ],
                false,
            ),
            Step::Write(0x3c, &TOP, false),
            Step::Write(0x3c, &MIDDLE, false),
            Step::Write(0x3c, &MIDDLE, false),
            Step::Write(0x3c, &BOTTOM, false),
            Step::Write(0x3c, &[0, 0xaf], false),
            Step::Write(0x3c, &[0, 0xae, 0x8d, 0x10], false),
        ]));
        pedal.oled_test().unwrap();
        pedal.oled_off().unwrap();
        assert!(pedal.0.0.is_empty());
    }

    #[test]
    fn waits_for_conversion_and_decodes_signed_twelve_bit_counts() {
        let mut pedal = Pedal(Mock(&[
            Step::Write(0x48, &[1, 0xc3, 0x83], false),
            Step::Read(0x48, 1, &[0x43, 0x83]),
            Step::Read(0x48, 1, &[0xc3, 0x83]),
            Step::Read(0x48, 0, &[0x3f, 0xf0]),
            Step::Write(0x48, &[1, 0xd3, 0x83], false),
            Step::Read(0x48, 1, &[0xd3, 0x83]),
            Step::Read(0x48, 0, &[0xff, 0xe0]),
        ]));
        pedal.start_pot(Pot::Gain).unwrap();
        assert_eq!(pedal.poll_pot().unwrap(), None);
        assert_eq!(pedal.poll_pot().unwrap(), Some(1023));
        pedal.start_pot(Pot::Level).unwrap();
        assert_eq!(pedal.poll_pot().unwrap(), Some(-2));
        assert!(pedal.0.0.is_empty());
    }
}
