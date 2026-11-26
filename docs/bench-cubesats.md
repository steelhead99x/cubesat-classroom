# Bench CubeSats: one Arduino build, one Seeed XIAO build

These are two complete classroom CubeSats for the bench, each built from a single vendor's parts. Each one has the same subsystems a real CubeSat has: structure, power, a flight computer, payload sensors, attitude sensors, data storage, a deployment inhibit and a link to the ground. The USB cable stands in for the radio and the Arduino IDE Serial Monitor is the ground station.

| | Arduino CubeSat | Seeed XIAO CubeSat |
| --- | --- | --- |
| Vendor | Arduino | Seeed Studio |
| Flight computer | Nano 33 BLE Sense Rev2 | XIAO ESP32-S3 Sense (or ESP32-C3, RP2040, SAMD21) on the XIAO Expansion Board |
| Payload | Built-in temperature, humidity, pressure, light and color | Grove BME280, Grove Digital Light Sensor, camera on the S3 Sense |
| Attitude sensors | Built-in accelerometer, gyroscope, magnetometer | Grove 6-axis accelerometer and gyroscope |
| Status | RGB LED | OLED screen, user LED, buzzer |
| Data storage | 64 readings in memory | 64 readings in memory, plus a microSD log and photos |
| Clock | Time since power-up | Real-time clock with a coin cell |
| Deploy inhibit | Jumper wire (remove-before-flight pin) | Button (deployer switch) |
| Parts to wire | One jumper wire | Plug-in Grove cables |
| Approximate cost | about $40–45 | about $65–90 |
| Firmware | [firmware/arduino-cubesat](../firmware/arduino-cubesat/arduino-cubesat.ino) | [firmware/xiao-cubesat](../firmware/xiao-cubesat/xiao-cubesat.ino) |

**Status: written 25 November 2026. Both sketches compile in CI for every board listed. Neither has been run on hardware yet.** Label every result "hardware bench", and check each sensor against a reference before you trust it. Nothing here transmits on radio: both sketches leave Wi-Fi and Bluetooth off.

Start with the Arduino build if your class wants something working in ten minutes with almost no wiring. Pick the XIAO build if you want a screen, a data recorder, a clock and a camera, and are happy to plug in a few Grove cables.

## What the class does with either CubeSat

Both sketches fly the same short mission, so one lesson plan works for both.

1. **Integration.** The CubeSat starts *inhibited*, like a satellite packed in its deployer: powered on the bench but not allowed to transmit. Commands get no answer.
2. **Deployment.** Pull the jumper (Arduino) or press the button (XIAO). A 30-second timer starts. Real deployers also require a delay before deployables or transmitters switch on. The provider sets the real delay, so treat 30 seconds as a demo value.
3. **First contact.** The CubeSat sends a beacon, then one telemetry line a second. The crew sends `PING`, `SENSE` and `STATUS` and reads the replies.
4. **Lose the signal.** Send `LOS`. The CubeSat keeps measuring but stores its readings instead of sending them. Commands get no answer, because nothing reaches a satellite that is out of range. Send `AOS` for the next pass and the stored readings arrive as `REC` lines. `ORBIT 60 60` repeats this on its own: 60 seconds in contact, 60 seconds out.
5. **Safe mode.** Send `SAFE`. The CubeSat slows to one reading every 10 seconds and refuses some commands until `NOMINAL`.
6. **Bring back data.** Calibrate the Arduino's light sensor (`CAL`), or take and downlink a photo with the XIAO ESP32-S3 Sense (`PHOTO`, then `DOWNLINK`).

These steps line up with the browser lessons [First contact](https://cubesatclass.org/quests/first-contact/), [Lose the signal](https://cubesatclass.org/quests/lose-the-signal/), [Bring back a picture](https://cubesatclass.org/quests/bring-back-a-picture/) and [Light and shadow](https://cubesatclass.org/quests/light-and-shadow/), so a class can run the simulation first and then the bench CubeSat.

## Talking to the CubeSat

Open the Serial Monitor at **115200 baud** with the line ending set to **Newline**. Type a command and press Enter. Letter case doesn't matter.

| Command | What it does |
| --- | --- |
| `PING` | Replies `ACK - PING` with the latest sequence number |
| `SENSE` | Sends one reading now |
| `STATUS` | Mode, reading interval, stored readings, uptime and which sensors were found |
| `RATE 5` | One reading every 5 seconds (1 to 60) |
| `SAFE`, `NOMINAL` | Enter or leave safe mode |
| `LOS`, `AOS` | End or start a pass by hand |
| `ORBIT 60 60`, `ORBIT OFF` | Automatic passes: seconds in contact, seconds out of contact (10 to 600 each) |
| `CAL 250` | Arduino only: the light meter beside the sensor reads 250 lux, so scale the counts to match |
| `TIME 2026-11-25T14:30:00` | XIAO only: set the real-time clock |
| `PHOTO`, `DOWNLINK` | XIAO ESP32-S3 Sense only: take a 320 × 240 photo, then send it as hex |
| `HELP` | List the commands |

**Numbered commands.** Put a number first, such as `7 PING`, and the reply carries it: `ACK 7 PING`. Send `7 PING` again and the CubeSat answers `NAK 7 PING duplicate`. It remembers the last eight numbers. Real command links number their commands so that a repeated or replayed command is not carried out twice.

### What the lines mean

| Starts with | Meaning |
| --- | --- |
| `CSC1` | A live reading. This is the same format the station's `sensor-bridge` reads. |
| `REC` | A stored reading, sent after a pass begins. It is old, so it never uses the `CSC1` prefix and the station never shows it as live. |
| `BCN` | Beacon, every 10 seconds while in contact |
| `ACK`, `NAK` | Command accepted or refused, with the reason |
| `EVT` | Mode change or start of a pass |
| `IMG` | Photo data (XIAO ESP32-S3 Sense) |
| `#` | Bench test port only. A real CubeSat could not send these, for example while it is still inhibited. |

Example from the Arduino build (spaces added between groups):

```text
CSC1 seq=42 t_ms=61000 mode=NOMINAL  temp_mc=27410 volt_mv=3297 pressure_pa=101210 humidity_mpct=38200
     light_clear=412 light_r=130 light_g=151 light_b=118  ax_mg=12 ay_mg=-8 az_mg=1003 ...
```

Units are whole numbers, as in the rest of the station: `temp_mc` is thousandths of a degree Celsius, `volt_mv` millivolts, `pressure_pa` pascals, `humidity_mpct` thousandths of a percent relative humidity, `light_lux` lux, `ax_mg` thousandths of a g, `gx_mdps` thousandths of a degree per second, and `mx_nt` nanotesla. `seq` counts up by one for every reading, stored or live, so a gap in `seq` shows that readings were lost.

### Connecting to the classroom station

The station's [`sensor-bridge`](ground-station.md#level-1-classroom-sensors) reads `CSC1` lines from either CubeSat, just as it reads the classroom sensor sketch. It uses `temp_mc`, `volt_mv`, `light_lux` and `pressure_pa` and ignores the other fields. Only one program can hold the USB port at a time, so close the Serial Monitor before you start the bridge. Commands can't be sent while the bridge is running.

The station needs temperature, voltage and light. Until it has all three, it reports "A sensor input is unavailable", by design:

- **Arduino:** `light_lux` appears only after the light sensor is calibrated (see below).
- **XIAO:** `volt_mv` appears only after a mentor wires the voltage divider.

The bridge removes readings older than five seconds. During `LOS`, in safe mode, or with `RATE` above 4, the station shows the sensors as unavailable. That is the "lose the signal" lesson, seen from the ground.

## Arduino CubeSat

### Parts

| Part | Job | Approx. price (US) |
| --- | --- | --- |
| [Arduino Nano 33 BLE Sense Rev2](https://store-usa.arduino.cc/products/nano-33-ble-sense-rev2) (ABX00069, or ABX00070 with headers) | Flight computer, every sensor, status light | $38.70 at the Arduino US store; $43.50 with headers at SparkFun |
| Micro-USB data cable | Power and the ground link | already in most kits |
| One jumper wire | Remove-before-flight pin | already in most kits |
| Optional: Rev B printed frame | Structure | filament |

It must be the **Rev2** board. Rev2 replaced the motion sensors (BMI270 and BMM150) and the temperature and humidity sensor (HS3003). A Rev1 board will report that those sensors are missing.

### Subsystems

| Subsystem | On this build | What a flight CubeSat adds |
| --- | --- | --- |
| Structure | The [Rev B rail frame](../hardware/README.md#rev-b-rail-fit-set) holds the board on the base plate | Qualified material, retainers, a deployer interface |
| Power (EPS) | USB 5 V, regulated to 3.3 V on the board. The sketch measures that 3.3 V rail itself and enters safe mode if it falls below 3.0 V. | Solar cells, a battery, charge control and separate power switches |
| Flight computer | nRF52840 running the sketch | A watchdog, radiation tolerance, flight-qualified firmware |
| Payload | HS3003 temperature and humidity, LPS22HB pressure, APDS9960 light and color | The mission instrument |
| Attitude sensors | BMI270 accelerometer and gyroscope, BMM150 magnetometer | Sun sensors, magnetorquers or reaction wheels to control attitude |
| Data storage | The last 64 readings, kept in memory while out of contact | Non-volatile storage that survives a reset |
| Communications | USB serial stands in for the radio. Bluetooth stays off. | A licensed radio with its own inhibit |
| Inhibit | Jumper from D2 to GND | Several independent hardware switches in the power path |

### Build

1. A mentor checks the board and the jumper before USB is connected. The only wire runs from **D2** to **GND**.
2. In the Arduino IDE, install **Arduino Mbed OS Nano Boards** in the Boards Manager. In the Library Manager, install **Arduino_HS300x**, **Arduino_LPS22HB**, **Arduino_APDS9960** and **Arduino_BMI270_BMM150**.
3. Open `firmware/arduino-cubesat/arduino-cubesat.ino`. Choose **Arduino Nano 33 BLE** and the port, then upload.
4. Open the Serial Monitor. The boot lines list any sensor that wasn't found.

**Status light:** off while inhibited; blue, blinking once a second, while the deploy timer runs; a green blink for each reading sent; a yellow blink for each reading stored out of contact; slow red in safe mode; white at the start of a pass.

### Check the readings

- **Temperature reads warm.** The sensor sits next to the processor. Compare it with a thermometer, record the difference, and say which number you used.
- **Voltage** is the board's regulated 3.3 V rail, measured by the processor itself. It should stay close to 3300 mV on laptop USB and on a power bank. That steadiness is the regulator at work. It is not a battery reading.
- **Pressure** should be close to your local weather report's station pressure. That is not the sea-level pressure most reports give, so check which one you are comparing.
- **Light** comes out as `light_clear` counts. To get lux, hold a light meter (or the BH1750 from the [classroom sensor kit](ground-station.md#level-1-classroom-sensors)) beside the sensor, send `SENSE`, then `CAL` with the meter's reading. To keep the result after a reset, copy the printed `lux_per_count` into `LUX_PER_COUNT` in the sketch. The counts saturate in bright light, and one calibration only holds for similar light.
- **Motion:** lying flat, `az_mg` should be close to 1000 (1 g). Spin the board and watch `gz_mdps`.

## Seeed XIAO CubeSat

### Parts

| Part | Job | Approx. price (US) |
| --- | --- | --- |
| [Seeed Studio XIAO ESP32-S3 Sense](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/) | Flight computer and camera | about $14–22 |
| or XIAO ESP32-C3, RP2040 or SAMD21 | Flight computer, no camera | about $4–8 |
| [Seeed Studio XIAO Expansion Board](https://wiki.seeedstudio.com/Seeeduino-XIAO-Expansion-Board/) (103030356) | OLED screen, real-time clock, microSD, button, buzzer, Grove ports | $14.90 at launch |
| [Grove BME280](https://wiki.seeedstudio.com/Grove-Barometer_Sensor-BME280/) (101020193) | Temperature, humidity, pressure | about $20–22 |
| [Grove Digital Light Sensor, TSL2561](https://wiki.seeedstudio.com/Grove-Digital_Light_Sensor/) (101020030) | Light in lux, 0.1 to 40,000 lux | about $6.50 |
| [Grove 6-Axis Accelerometer & Gyroscope, LSM6DS3](https://wiki.seeedstudio.com/Grove-6-Axis_AccelerometerAndGyroscope/) (105020012) | Motion | about $12.50 |
| Grove I2C Hub | Joins three Grove sensors to one port | about $2 (estimate) |
| microSD card, 32 GB or smaller, FAT32 | Data recorder | about $5 (estimate) |
| CR1220 coin cell | Keeps the clock running when unplugged | about $1 (estimate) |
| USB-C data cable | Power and the ground link | already in most kits |

Every part is optional. The sketch reports what it finds at boot and leaves out the readings it cannot make. A XIAO on its own still runs the whole mission, with the chip's own temperature (`chip_temp_mc`, on the ESP32 and RP2040 boards) as its only reading. That is the processor's temperature, not the room's, so the sketch never sends it as `temp_mc`.

**Check two fits before you buy.** Seeed's Expansion Board page lists the XIAO SAMD21, RP2040 and nRF52840 as compatible. It mentions the ESP32-C3 and ESP32-S3 elsewhere, but does not list them as compatible. Also, the S3 Sense's camera board mounts on the back of the XIAO, where it may hit the Expansion Board. We have not tested either fit. If the camera board doesn't clear, fly the S3 Sense without the Expansion Board and connect the Grove sensors with Grove-to-jumper cables to 3V3, GND, SDA (D4) and SCL (D5).

### Subsystems

| Subsystem | On this build | What a flight CubeSat adds |
| --- | --- | --- |
| Structure | The Rev B rail frame holds the Expansion Board on the base plate. The camera looks out through the 18 mm aperture in the camera plate. | Qualified material, retainers, a deployer interface |
| Power (EPS) | USB 5 V. The Expansion Board can also charge a 3.7 V lithium battery, at up to 460 mA. Leave the battery out unless a mentor has followed the [battery safety quest](../quest/m1/battery-safety.md). An optional divider on A0 measures a voltage of up to 6.6 V. | Solar cells, flight-qualified cells, charge control and protection |
| Flight computer | The XIAO running the sketch | A watchdog, radiation tolerance, flight-qualified firmware |
| Payload | BME280, TSL2561, camera (S3 Sense) | The mission instrument |
| Attitude sensors | LSM6DS3 accelerometer and gyroscope | A magnetometer, sun sensors, actuators |
| Data storage | The last 64 readings in memory. Every reading also goes to `CUBESAT.CSV` on the microSD card (or `CUBE0001.CSV` and up, if the card holds a log with different columns from an older sketch), and photos to `IMG0001.JPG` and onward. | Radiation-tolerant storage |
| Time | PCF8563 real-time clock, set with `TIME` | Time from GPS or the ground |
| Communications | USB serial stands in for the radio. Wi-Fi and Bluetooth stay off. | A licensed radio with its own inhibit |
| Inhibit | Expansion Board button (or D1 touched to GND) | Several independent hardware switches in the power path |

The I2C addresses don't collide: OLED 0x3C, clock 0x51, BME280 0x76 or 0x77, TSL2561 0x29, 0x39 or 0x49, LSM6DS3 0x6A or 0x6B. The sketch tries each one.

### Build

1. A mentor checks the parts before USB is connected. Seat the XIAO in the Expansion Board as the Seeed wiki shows. Plug the I2C Hub into one Grove I2C port and the three sensors into the hub. Insert the coin cell and the microSD card.
2. In the Arduino IDE Boards Manager, install the package for your XIAO: **esp32** by Espressif (ESP32-S3, ESP32-C3), **Raspberry Pi Pico/RP2040** by Earle Philhower (RP2040), or **Seeed SAMD Boards** (SAMD21). Seeed's getting-started page for each board gives the Boards Manager URL.
3. In the Library Manager, install **Adafruit BME280 Library**, **Adafruit TSL2561**, **Adafruit Unified Sensor** and **U8g2**. On the SAMD21, also install **SD**.
4. Open `firmware/xiao-cubesat/xiao-cubesat.ino`, choose your XIAO and its port, and upload. On the ESP32 boards, set **Tools → USB CDC On Boot → Enabled** if the Serial Monitor stays empty. On the S3 Sense, also set **Tools → PSRAM → OPI PSRAM** for the camera.
5. Open the Serial Monitor. Send `TIME` with the current time once the CubeSat has deployed.

**Status:** the OLED shows the mode, deploy countdown, contact, sequence number and the latest readings. The XIAO's user LED blinks with each reading (the ESP32-C3 has no user LED). The buzzer sounds at deployment and at the start of each pass. It is switched off on the S3 Sense when the camera is running.

### Voltage (optional, mentor-wired)

The XIAO boards have no built-in way to read a battery voltage. To add `volt_mv`, a mentor wires two 100 kΩ resistors in series from the voltage under test to A0 to GND, with a 100 nF capacitor from A0 to GND, then sets `USE_VOLTAGE_DIVIDER = true`. The divider halves the voltage, so the most it can measure is 6.6 V. Never connect more than that. For a first test, connect the top of the divider to the 3V3 pin. The reading should be close to 3300 mV.

### Bring back a picture (XIAO ESP32-S3 Sense)

1. Send `PHOTO`. The reply gives the photo's size and a CRC-32 checksum, and the photo is saved to the card.
2. Clear the Serial Monitor, send `DOWNLINK`, wait for `IMG END`, then copy all the output into a text file.
3. Run `python3 tools/serial_image.py capture.txt photo.jpg`. The script puts the chunks back in order and checks the size and checksum. It says the photo arrived intact only if every byte arrived and the checksum matches. If any lines are missing, it lists the lost byte ranges, reports the transfer as incomplete (even if the checksum happens to match the zero-filled gap), and the photo shows the damage.

A 320 × 240 JPEG is usually a few kilobytes to a few tens of kilobytes. Sent as hex, every byte becomes two characters, plus line overhead. Ask the class how long the downlink would take at 1200 bits per second, a common amateur packet radio speed, and how many passes it would need.

The camera sensor varies by batch. Seeed has shipped the S3 Sense with OV2640, OV3660 and, most recently, OV5640 cameras, and the sketch's camera driver supports all three. Autofocus on the OV5640 is not used.

## Mounting either CubeSat in the Rev B frame

The [Rev B rail frame](../hardware/README.md#rev-b-rail-fit-set) is 100 × 100 × 113.5 mm outside. From the CAD values in `hardware/generate.py`, the inside is about 88 mm square between the side walls (about 82 mm clear of the corner rails) and about 98 mm between the end-plate locator tabs. These are nominal CAD numbers, so measure your print.

- The Nano 33 BLE Sense is 45 × 18 mm. The XIAO is 21 × 17.8 mm, and the Grove sensor boards are about 20 × 20 to 20 × 40 mm. The Expansion Board's size isn't on its wiki page, so measure it. Seeed describes it as half the size of a Raspberry Pi 4.
- For the bench, hold the boards to the base plate with double-sided foam tape or cable ties. Leave the USB connector reachable through a gap between the side walls. Point the light sensor at the camera-plate aperture so the class can shade it.
- Printed board mounts are still a design task: see [printed payload mounts](../quest/m1/printed-payload-mounts.md).
- Weigh the assembled CubeSat and record it. The [mass and balance quest](../quest/m1/mass-and-balance.md) shows what to do with the number.

## Power

Measure the power instead of trusting a budget. Put a USB power meter between the computer and the CubeSat, record the current in each mode (inhibited, nominal, safe, and photo for the S3 Sense), and add the numbers to `cargo run --locked -- energy-budget`.

Published figures to compare against: Seeed lists the XIAO ESP32-S3 Sense at about 38 mA from 5 V (0.19 W) when idle with the camera board fitted, and about 110 mA with Wi-Fi on, which this sketch never uses. Taking photos draws more than idle. The Grove BME280 draws about 0.4 mA, and the LSM6DS3 0.9 to 1.25 mA. The OLED, the microSD card and the Nano 33 BLE Sense have no figure here: measure them.

## What these CubeSats leave out

A bench CubeSat is a model, so tell the class what it can't show:

- **No radio.** A real downlink needs a licensed operator, frequency coordination and authorization. See [mission and radio design](mission.md) and the [spectrum notes](../README.md#radio-and-safety). Turning on the boards' Wi-Fi or Bluetooth would not make this a satellite link.
- **No watchdog.** The sketch does not reset itself if it hangs. A flight computer must.
- **One inhibit, in software.** Flight inhibits are several independent hardware switches that cut power, not one pin read by the code. See [deployment inhibits](../quest/m1/deployment-inhibits.md).
- **No thermal, vacuum, vibration or radiation testing.** These parts were not designed for space.
- **Stored readings are lost at reset** on the Arduino build. The XIAO build also keeps them on the card.

## Sources

Arduino: [Nano 33 BLE Sense Rev2 store page](https://store-usa.arduino.cc/products/nano-33-ble-sense-rev2) · [Rev2 sensor changes (CNX Software)](https://www.cnx-software.com/2023/02/24/arduino-nano-33-ble-sense-rev2-switches-to-bmi270-bmm150-imus-hs3003-temperature-humidity-sensor/) · [SparkFun listing with dimensions](https://www.sparkfun.com/arduino-nano-ble-sense-rev2-with-headers.html)

Seeed Studio: [XIAO ESP32-S3 getting started](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/) · [XIAO Expansion Board](https://wiki.seeedstudio.com/Seeeduino-XIAO-Expansion-Board/) · [Expansion Board launch price (Hackster)](https://www.hackster.io/news/seeed-studio-s-compact-arduino-compatible-xiao-gets-an-expansion-board-f16d78794c08) · [S3 Sense launch price (CNX Software)](https://www.cnx-software.com/2023/03/29/xiao-esp32s3-sense-camera-and-a-microphone-round-touchscreen-display/) · [Grove BME280](https://wiki.seeedstudio.com/Grove-Barometer_Sensor-BME280/) and [listing](https://thepihut.com/products/grove-temperature-humidity-barometer-sensor-bme280) · [Grove BME280 price (RobotShop)](https://www.robotshop.com/products/grove-temperature-barometer-sensor-bme280) · [Grove Digital Light Sensor](https://wiki.seeedstudio.com/Grove-Digital_Light_Sensor/) · [Grove 6-Axis Accelerometer & Gyroscope](https://wiki.seeedstudio.com/Grove-6-Axis_AccelerometerAndGyroscope/) and [price (RobotShop)](https://robotshop.com/products/grove-6-axis-accelerometer-gyroscope) · [XIAO board prices (Hackster store)](https://hackster.io/products/buy/94650)

Prices were checked on 25 November 2026 and will change. Prices marked "estimate" are typical ranges, not quotes.
