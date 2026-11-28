#![no_std]
#![no_main]

use csc_r1p_bringup::{
    EncoderTrace, Pedal, Pot,
    pll::{BenchBand, POWER_DOWN},
};
use embedded_hal::{i2c::I2c, spi::SpiBus};
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config, I2c as HardwareI2c},
    spi::{
        Mode,
        master::{Config as SpiConfig, Spi},
    },
    time::Rate,
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
    let mut latch = Output::new(p.GPIO8, Level::Low, OutputConfig::default());
    let lock = Input::new(p.GPIO4, InputConfig::default().with_pull(Pull::Down));
    let mut spi = Spi::new(
        p.SPI2,
        SpiConfig::default()
            .with_mode(Mode::_0)
            .with_frequency(Rate::from_mhz(1)),
    )
    .unwrap()
    .with_sck(p.GPIO7)
    .with_mosi(p.GPIO9);
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
                            println!(
                                "SAFE | CONTROLS | LED ON | LED OFF | POTS | ENCODER TEST | OLED TEST | ADC CHECK | PLL TEST 2M | PLL TEST 70CM | HELP"
                            );
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
                        b"ENCODER TEST" => {
                            println!("Turn TUNE in one direction for five seconds.");
                            match trace_encoder(&mut pedal, &delay) {
                                Ok(trace) => println!(
                                    "ENCODER forward={} reverse={} missed={}",
                                    trace.forward_edges, trace.reverse_edges, trace.missed_samples
                                ),
                                Err(error) => println!("ENCODER I2C error: {error:?}"),
                            }
                        }
                        b"OLED TEST" => {
                            let show = pedal.oled_test();
                            if show.is_ok() {
                                delay.delay_millis(1500);
                            }
                            let off = pedal.oled_off();
                            println!("OLED show={show:?} off={off:?}");
                        }
                        b"ADC CHECK" => check_adc(&mut pedal, &delay),
                        b"PLL TEST 2M" | b"PLL TEST 70CM" => {
                            let band = if &line[..len] == b"PLL TEST 2M" {
                                BenchBand::TwoMeters
                            } else {
                                BenchBand::SeventyCentimeters
                            };
                            run_pll_test(&mut pedal, &mut spi, &mut latch, &lock, &delay, band);
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

fn check_adc<Bus: I2c>(pedal: &mut Pedal<Bus>, delay: &Delay) {
    if let Err(error) = pedal.enter_safe() {
        println!("ADC: safe setup failed: {error:?}");
        return;
    }
    // Allow a previous shutdown to settle before releasing SHDNZ again.
    delay.delay_millis(100);
    let status = pedal.adc_release_shutdown(true).and_then(|()| {
        delay.delay_millis(2);
        pedal.adc_status()
    });
    let shutdown = pedal.adc_release_shutdown(false);
    println!("ADC status={status:02x?} shutdown={shutdown:?}");
}

fn run_pll_test<Bus: I2c, Serial: SpiBus>(
    pedal: &mut Pedal<Bus>,
    spi: &mut Serial,
    latch: &mut Output<'_>,
    lock: &Input<'_>,
    delay: &Delay,
    band: BenchBand,
) {
    if let Err(error) = pedal.enter_safe() {
        println!("PLL: safe setup failed: {error:?}");
        return;
    }
    if lock.is_high() {
        println!("PLL: MUXOUT is high with CE low; check the lock wiring");
        return;
    }
    match pedal.pll_enable(true) {
        Ok(()) => {
            // Allow the part to come out of CE power-down before loading R5.
            delay.delay_millis(1);
            match band
                .registers()
                .into_iter()
                .try_for_each(|word| write_pll_register(spi, latch, delay, word))
            {
                Ok(()) => {
                    let mut high_samples = 0;
                    for _ in 0..100 {
                        delay.delay_millis(1);
                        if lock.is_high() {
                            high_samples += 1;
                            if high_samples == 3 {
                                break;
                            }
                        } else {
                            high_samples = 0;
                        }
                    }
                    if high_samples == 3 {
                        println!("PLL {} Hz: locked, measure LO MON now", band.frequency_hz());
                        delay.delay_millis(5_000);
                    } else {
                        println!("PLL {} Hz: lock timeout", band.frequency_hz());
                    }
                }
                Err(error) => println!("PLL: SPI programming failed: {error:?}"),
            }
        }
        Err(error) => println!("PLL: CE enable failed: {error:?}"),
    }

    let power_down = write_pll_register(spi, latch, delay, POWER_DOWN);
    let ce_low = pedal.pll_enable(false);
    println!("PLL shutdown: register={power_down:?} CE={ce_low:?}");
}

fn write_pll_register<Serial: SpiBus>(
    spi: &mut Serial,
    latch: &mut Output<'_>,
    delay: &Delay,
    word: u32,
) -> Result<(), Serial::Error> {
    spi.write(&word.to_be_bytes())?;
    spi.flush()?;
    delay.delay_micros(1);
    latch.set_high();
    delay.delay_micros(1);
    latch.set_low();
    delay.delay_micros(1);
    Ok(())
}

fn trace_encoder<Bus: I2c>(
    pedal: &mut Pedal<Bus>,
    delay: &Delay,
) -> Result<EncoderTrace, Bus::Error> {
    let input = pedal.controls()?;
    let mut trace = EncoderTrace::new(input.encoder_a, input.encoder_b);
    let start = esp_hal::time::Instant::now();
    while start.elapsed().as_millis() < 5_000 {
        delay.delay_millis(2);
        let input = pedal.controls()?;
        trace.sample(input.encoder_a, input.encoder_b);
    }
    Ok(trace)
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
