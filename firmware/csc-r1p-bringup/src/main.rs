#![no_std]
#![no_main]

use csc_r1p_bringup::{Pedal, Pot};
use embedded_hal::i2c::I2c;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::master::{Config, I2c as HardwareI2c},
    usb::usb_serial_jtag::UsbSerialJtag,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let p = esp_hal::init(esp_hal::Config::default());
    // Use esp-println for output: its USB writer times out if no host is connected.
    let mut usb = UsbSerialJtag::new(p.USB_DEVICE);
    let bus = HardwareI2c::new(p.I2C0, Config::default())
        .unwrap()
        .with_sda(p.GPIO5)
        .with_scl(p.GPIO6);
    let mut pedal = Pedal(bus);
    let delay = Delay::new();
    let mut controls_ready = setup(&mut pedal);

    let mut line = [0; 32];
    let mut len = 0;
    let mut discard = false;
    loop {
        let Ok(byte) = usb.read_byte() else {
            delay.delay_millis(1);
            continue;
        };
        match byte {
            b'\r' | b'\n' => {
                if discard {
                    println!("ERR command too long");
                } else if len != 0 {
                    match &line[..len] {
                        b"HELP" => {
                            println!("SAFE | CONTROLS | LED ON | LED OFF | POTS | HELP");
                        }
                        b"SAFE" => controls_ready = setup(&mut pedal),
                        b"CONTROLS" => {
                            println!("{:?}", pedal.controls());
                        }
                        b"LED ON" | b"LED OFF" if controls_ready => {
                            println!("LED {:?}", pedal.led(&line[..len] == b"LED ON"));
                        }
                        b"LED ON" | b"LED OFF" => {
                            println!("ERR control board setup failed. Retry SAFE.");
                        }
                        b"POTS" => {
                            let gain = read_pot(&mut pedal, Pot::Gain, &delay);
                            let level = read_pot(&mut pedal, Pot::Level, &delay);
                            println!("POTS gain={gain:?} level={level:?} (raw counts)");
                        }
                        _ => {
                            println!("ERR unknown command. Type HELP.");
                        }
                    }
                }
                len = 0;
                discard = false;
            }
            _ if discard => {}
            _ if len == line.len() => discard = true,
            _ => {
                line[len] = byte;
                len += 1;
            }
        }
    }
}

fn setup<Bus: I2c>(pedal: &mut Pedal<Bus>) -> bool {
    let main = pedal.enter_safe();
    let controls = pedal.init_controls();
    println!("CSC-R1P bring-up: RF safe={main:?} controls={controls:?}");
    controls.is_ok()
}

fn read_pot<Bus: I2c>(
    pedal: &mut Pedal<Bus>,
    pot: Pot,
    delay: &Delay,
) -> Result<Option<i16>, Bus::Error> {
    pedal.start_pot(pot)?;
    for _ in 0..5 {
        delay.delay_millis(1);
        if let Some(sample) = pedal.poll_pot()? {
            return Ok(Some(sample));
        }
    }
    Ok(None)
}
