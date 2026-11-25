//! The Nano 33 BLE Sense Rev2's built-in sensors on its internal I2C bus, and the
//! processor's own measurement of its 3.3 V rail. Register-level and small on purpose,
//! so a class can read each step against the datasheet.

use cubesat_mission::{Present, Readings};
use embassy_nrf::saadc::Saadc;
use embassy_nrf::twim::Twim;
use embassy_time::{Duration, Timer, with_timeout};

const HS3003: u8 = 0x44; // temperature and humidity (Renesas HS3003)
const LPS22HB: u8 = 0x5C; // pressure (ST LPS22HB)
const APDS9960: u8 = 0x39; // light and colour (Broadcom APDS-9960)
const BUS_TIMEOUT: Duration = Duration::from_millis(20);

/// One I2C transaction with a timeout, so a stuck sensor can't stall the mission.
async fn write(bus: &mut Twim<'static>, addr: u8, bytes: &[u8]) -> bool {
    matches!(
        with_timeout(BUS_TIMEOUT, bus.write(addr, bytes)).await,
        Ok(Ok(()))
    )
}

async fn read_reg(bus: &mut Twim<'static>, addr: u8, reg: u8, buf: &mut [u8]) -> bool {
    matches!(
        with_timeout(BUS_TIMEOUT, bus.write_read(addr, &[reg], buf)).await,
        Ok(Ok(()))
    )
}

/// Find the sensors and set them up. Run once after the sensor rail powers up.
pub async fn probe(bus: &mut Twim<'static>) -> Present {
    let mut id = [0u8; 1];
    // HS3003 has no ID register: a measurement request that is acknowledged means it's there.
    let temp_humidity = write(bus, HS3003, &[]).await;
    // LPS22HB: WHO_AM_I (0x0F) reads 0xB1.
    let pressure = read_reg(bus, LPS22HB, 0x0F, &mut id).await && id[0] == 0xB1;
    // APDS-9960: ID (0x92) reads 0xAB.
    let color = read_reg(bus, APDS9960, 0x92, &mut id).await && id[0] == 0xAB;
    if color {
        // ATIME 0xB6: 74 cycles of 2.78 ms, about 200 ms per reading. CONTROL: 4x gain.
        // ENABLE: power on (PON) and the colour engine (AEN).
        let _ = write(bus, APDS9960, &[0x81, 0xB6]).await;
        let _ = write(bus, APDS9960, &[0x8F, 0x01]).await;
        let _ = write(bus, APDS9960, &[0x80, 0x03]).await;
    }
    Present {
        temp_humidity,
        pressure,
        color,
        imu: false,
    }
}

/// Take one set of readings. Missing or failed sensors come back as `None`.
pub async fn read(
    bus: &mut Twim<'static>,
    adc: &mut Saadc<'static, 1>,
    present: Present,
) -> Readings {
    let mut r = Readings {
        volt_mv: rail_mv(adc).await,
        ..Readings::default()
    };

    if present.temp_humidity && write(bus, HS3003, &[]).await {
        // Conversion takes about 33 ms at 14 bits for both values.
        Timer::after_millis(40).await;
        let mut b = [0u8; 4];
        if matches!(
            with_timeout(BUS_TIMEOUT, bus.read(HS3003, &mut b)).await,
            Ok(Ok(()))
        ) && b[0] >> 6 == 0
        {
            let hum = i32::from(u16::from_be_bytes([b[0], b[1]]) & 0x3FFF);
            let temp = i32::from(u16::from_be_bytes([b[2], b[3]]) >> 2);
            // Datasheet: RH = raw / (2^14 - 1) x 100; T = raw / (2^14 - 1) x 165 - 40.
            r.humidity_mpct = Some(hum * 100_000 / 16_383);
            r.temp_mc = Some(temp * 165_000 / 16_383 - 40_000);
        }
    }

    if present.pressure && write(bus, LPS22HB, &[0x11, 0x11]).await {
        // CTRL_REG2: one-shot measurement, register auto-increment left on.
        Timer::after_millis(30).await;
        let mut b = [0u8; 3];
        if read_reg(bus, LPS22HB, 0x28, &mut b).await {
            // 24-bit pressure, 4096 counts per hPa, so Pa = raw x 100 / 4096.
            let raw =
                i32::from_le_bytes([b[0], b[1], b[2], if b[2] & 0x80 != 0 { 0xFF } else { 0 }]);
            let pa = raw * 25 / 1024;
            if (1_000..120_000).contains(&pa) {
                r.pressure_pa = Some(pa);
            }
        }
    }

    if present.color {
        let mut status = [0u8; 1];
        let mut b = [0u8; 8];
        // STATUS bit 0 (AVALID): a full integration is ready. Then clear, red, green, blue.
        if read_reg(bus, APDS9960, 0x93, &mut status).await
            && status[0] & 0x01 != 0
            && read_reg(bus, APDS9960, 0x94, &mut b).await
        {
            let w = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
            r.color = Some([w(0), w(2), w(4), w(6)]);
        }
    }
    r
}

/// The regulated 3.3 V rail, measured by the processor's ADC against its 0.6 V
/// reference with a gain of 1/6: full scale is 3.6 V over 4096 counts.
/// This is the regulator's output, not a battery.
pub async fn rail_mv(adc: &mut Saadc<'static, 1>) -> i32 {
    let mut buf = [0i16; 1];
    adc.sample(&mut buf).await;
    i32::from(buf[0].max(0)) * 3600 / 4096
}
