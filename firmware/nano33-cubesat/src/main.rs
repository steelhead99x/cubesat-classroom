//! Bench CubeSat firmware for the Arduino Nano 33 BLE Sense Rev2, in Rust.
//!
//! The mission itself (modes, commands, store and forward, telemetry lines) lives in
//! the `cubesat-mission` crate and is tested on a computer. This file connects it to
//! the board: USB serial for the ground link, the RGB status light, the D2
//! remove-before-flight jumper, and the built-in sensors.
//!
//! Wiring (a mentor checks it before USB is connected):
//!   Remove-before-flight jumper: one wire from D2 to GND. Nothing else.
//!
//! The radio is never switched on. Bluetooth stays off.
//!
//! A hardware watchdog restarts the board if the main loop or the sensor task stops
//! for 5 s (a hung sensor, a panic, a bug). The next boot reports why it started,
//! so a restart is never silent.

#![no_std]
#![no_main]

mod sensors;

use core::cell::Cell;

use cubesat_mission::{Config, Mission, Present, Readings, ResetCause, SerialOut, Timing};
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::saadc::{self, ChannelConfig, Saadc, VddInput};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::wdt::{self, HaltConfig, SleepConfig, Watchdog, WatchdogHandle};
use embassy_nrf::{bind_interrupts, peripherals, usb};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Ticker, Timer, with_timeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use panic_halt as _;
use static_cell::StaticCell;

// ---- Mission settings: crews may change these ------------------------------

const CONFIG: Config = Config {
    sat_name: "CSC-RS",
    firmware: "nano33-cubesat 0.1.0 (Rust)",
    // Timing::Bench waits 30 s after deployment. Timing::Cds waits the 45 minutes
    // the CubeSat Design Specification requires before any transmission.
    timing: Timing::Bench,
    // Lux per light count times 100 000. Leave at 0 until a crew calibrates with CAL.
    lux_per_count_e5: 0,
    imu_driver: false,
};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

type UsbDriver = Driver<'static, HardwareVbusDetect>;

/// The latest readings and rail voltage, refreshed by the sensor task.
static LATEST: Mutex<CriticalSectionRawMutex, Cell<Readings>> = Mutex::new(Cell::new(Readings {
    volt_mv: 3300,
    temp_mc: None,
    humidity_mpct: None,
    pressure_pa: None,
    color: None,
    accel_mg: None,
    gyro_mdps: None,
    mag_nt: None,
}));
static PRESENT: Signal<CriticalSectionRawMutex, Present> = Signal::new();

#[embassy_executor::task]
async fn usb_task(mut device: embassy_usb::UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn sensor_task(
    mut bus: Twim<'static>,
    mut adc: Saadc<'static, 1>,
    mut dog: WatchdogHandle,
) -> ! {
    adc.calibrate().await;
    let present = sensors::probe(&mut bus).await;
    PRESENT.signal(present);
    loop {
        // If a sensor holds the bus forever, this stops and the watchdog restarts us.
        let r = sensors::read(&mut bus, &mut adc, present).await;
        LATEST.lock(|c| c.set(r));
        dog.pet();
        Timer::after_millis(150).await;
    }
}

/// Why the processor started, then clear the register (its bits accumulate and
/// are cleared by writing ones). Zero means a power-on reset. The Arduino bootloader
/// runs first and may clear it; then a reset shows as power_on.
fn reset_cause() -> ResetCause {
    let reg = nrf_pac::POWER.resetreas();
    let why = reg.read();
    reg.write_value(nrf_pac::power::regs::Resetreas(u32::MAX));
    if why.0 == 0 {
        ResetCause::PowerOn
    } else if why.dog() {
        ResetCause::Watchdog
    } else if why.lockup() {
        ResetCause::Lockup
    } else if why.sreq() {
        ResetCause::Software
    } else if why.resetpin() {
        ResetCause::ResetPin
    } else {
        ResetCause::Unknown
    }
}

fn now_ms() -> u32 {
    // Wraps after about 49 days; the mission's timers use wrapping arithmetic.
    #[allow(clippy::cast_possible_truncation)]
    let ms = Instant::now().as_millis() as u32;
    ms
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal; // USB needs the crystal
    let p = embassy_nrf::init(config);
    let reset = reset_cause();

    // Watchdog: two handles, one for the main loop and one for the sensor task.
    // Both must check in every 5 s. It pauses while a debugger halts the CPU.
    let mut dog_config = wdt::Config::default();
    dog_config.timeout_ticks = 5 * 32_768;
    dog_config.action_during_sleep = SleepConfig::Run;
    dog_config.action_during_debug_halt = HaltConfig::Pause;
    let (_watchdog, [mut main_dog, sensor_dog]) = match Watchdog::try_new(p.WDT, dog_config) {
        Ok(dog) => dog,
        // Already running with other settings (it survives a soft reset): stop here
        // and let it restart the board cleanly.
        Err(_) => loop {
            cortex_m::asm::wfe();
        },
    };

    // Status light: active low. Remove-before-flight jumper on D2 (P1.11): low = fitted.
    let mut red = Output::new(p.P0_24, Level::High, OutputDrive::Standard);
    let mut green = Output::new(p.P0_16, Level::High, OutputDrive::Standard);
    let mut blue = Output::new(p.P0_06, Level::High, OutputDrive::Standard);
    let rbf = Input::new(p.P1_11, Pull::Up);

    // Sensors: power the sensor rail (P0.22) and the I2C pull-ups (P1.00), then the bus.
    let _sensor_power = Output::new(p.P0_22, Level::High, OutputDrive::HighDrive);
    let _pullups = Output::new(p.P1_00, Level::High, OutputDrive::Standard);
    Timer::after_millis(20).await;
    // The I2C hardware can only send bytes from RAM. Register writes such as
    // `&[0x80, 0x03]` are constants the compiler keeps in flash, so the driver copies
    // them into this buffer first. Without it every one of those writes fails.
    static TWIM_TX: StaticCell<[u8; 16]> = StaticCell::new();
    let bus = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P0_14,
        p.P0_15,
        twim::Config::default(),
        TWIM_TX.init([0; 16]),
    );
    let adc = Saadc::new(
        p.SAADC,
        Irqs,
        saadc::Config::default(),
        [ChannelConfig::single_ended(VddInput)],
    );
    spawner.spawn(sensor_task(bus, adc, sensor_dog).unwrap());

    // USB serial: the ground link.
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));
    let mut usb_config = embassy_usb::Config::new(0x1209, 0x0001);
    usb_config.manufacturer = Some("CubeSat Classroom");
    usb_config.product = Some("nano33-cubesat");
    usb_config.serial_number = Some("CSC-RS");
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();
    let mut builder = embassy_usb::Builder::new(
        driver,
        usb_config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );
    let mut serial = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    spawner.spawn(usb_task(builder.build()).unwrap());

    // Give the computer up to 3 s to open the port so the boot lines are seen.
    let _ = with_timeout(Duration::from_secs(3), serial.wait_connection()).await;
    let present = with_timeout(Duration::from_millis(500), PRESENT.wait())
        .await
        .unwrap_or_default();

    // Text the mission writes, held until it goes out over USB (see SerialOut).
    static OUTBOX: StaticCell<SerialOut<4096>> = StaticCell::new();
    let out = OUTBOX.init(SerialOut::new());
    Timer::after_millis(60).await;
    let mut mission = Mission::boot(CONFIG, present, reset, now_ms(), rbf.is_low(), out);

    let mut sense = || LATEST.lock(Cell::get);
    let mut rail = || LATEST.lock(Cell::get).volt_mv;
    let mut ticker = Ticker::every(Duration::from_millis(10));
    let mut rx = [0u8; 64];
    loop {
        main_dog.pet();
        if serial.dtr() {
            match select(serial.read_packet(&mut rx), ticker.next()).await {
                Either::First(Ok(n)) => {
                    for &b in &rx[..n] {
                        mission.receive(b, now_ms(), &mut sense, &mut rail, out);
                    }
                }
                Either::First(Err(_)) => ticker.next().await,
                Either::Second(()) => mission.poll(now_ms(), rbf.is_low(), &mut sense, out),
            }
        } else {
            ticker.next().await;
            mission.poll(now_ms(), rbf.is_low(), &mut sense, out);
        }

        let light = mission.light(now_ms());
        red.set_level(if light.r { Level::Low } else { Level::High });
        green.set_level(if light.g { Level::Low } else { Level::High });
        blue.set_level(if light.b { Level::Low } else { Level::High });

        // Send what the mission wrote. With no terminal open, drop it, like a satellite
        // talking to an empty sky; stored readings in it stay stored for the next pass.
        // If only part goes out (a timeout or a USB error), keep the rest: the next send
        // resumes from that exact byte on the same connection, so a line is never cut and
        // started again. Sending never retires a stored reading: only the receiver's
        // `RECACK <seq>` does, because bytes copied to USB may still never reach anyone.
        if !serial.dtr() {
            out.clear();
            mission.undelivered();
        } else if !out.is_empty() {
            let mut sent = 0;
            let total = out.pending().len();
            while sent < total {
                let end = (sent + 64).min(total);
                let ok = matches!(
                    with_timeout(
                        Duration::from_millis(50),
                        serial.write_packet(&out.pending()[sent..end])
                    )
                    .await,
                    Ok(Ok(()))
                );
                if !ok {
                    break;
                }
                sent = end;
            }
            // A full 64-byte packet tells the computer more is coming, so it may hold
            // the line back. An empty packet ends the transfer.
            if sent == total && sent % 64 == 0 {
                let _ = with_timeout(Duration::from_millis(50), serial.write_packet(&[])).await;
            }
            out.sent(sent);
        }
    }
}
