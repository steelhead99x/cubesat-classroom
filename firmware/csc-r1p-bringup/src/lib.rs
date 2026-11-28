#![no_std]

pub mod pll;

use embedded_hal::i2c::I2c;

const RF: u8 = 0x20;
const CONTROL: u8 = 0x21;
const OLED: u8 = 0x3c;
const POTS: u8 = 0x48;

const INPUT: u8 = 0;
const OUTPUT: u8 = 1;
const DIRECTION: u8 = 3;
const STATUS_LED: u8 = 1 << 5;
const PLL_CE: u8 = 1 << 6;

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
