#![no_std]

use embedded_hal::i2c::I2c;

const RF: u8 = 0x20;
const CONTROL: u8 = 0x21;
const POTS: u8 = 0x48;

const INPUT: u8 = 0;
const OUTPUT: u8 = 1;
const DIRECTION: u8 = 3;
const STATUS_LED: u8 = 1 << 5;

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
