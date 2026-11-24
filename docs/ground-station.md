# Build a classroom ground station

A ground station is the equipment on the ground that listens to a satellite. This guide grows one in steps, from a laptop with no hardware to a Raspberry Pi that receives real amateur satellites. Each step works on its own, so a class can stop at any level.

Prices are US prices as shown on the pages listed under [Sources](#sources) when this guide was written in November 2026, before tax and shipping. Prices marked "estimate" are typical ranges, not quotes. Confirm prices before you buy. Power figures come from published measurements, also listed under Sources.

## Pick a level

| Level | What the class does | Main parts | Typical cost |
| --- | --- | --- | --- |
| 0. Desk simulation | Run the station and the quests on a laptop | Any laptop | $0 |
| 1. Classroom sensors | Real temperature, battery and light readings feed the station | Arduino or Seeed XIAO, three small sensors | $15–40 |
| 2. Satellite predictions | Know when a satellite will be overhead | Level 0 plus a free orbit file | $0 |
| 3. Receive-only radio | Hear and decode real satellites and CubeSatSim | Raspberry Pi 4, RTL-SDR, antenna | $150–300 |
| 4. Classroom satellite | Decode a working satellite model in the room | AMSAT CubeSatSim kit plus level 3 | about $550–700 |

In the United States, receiving satellites needs no license; check local rules elsewhere. Transmitting does need one: anything that sends on amateur bands, including the CubeSatSim and APRS balloon trackers, must be run by a licensed operator, and students follow that operator's lead.

## Level 1: classroom sensors

The sketch in [firmware/classroom-sensors](../firmware/classroom-sensors/classroom-sensors.ino) runs on an Arduino Uno or Nano and on the Seeed XIAO SAMD21, RP2040, ESP32-C3 and ESP32-S3. It prints one line a second over USB:

```text
CSC1 temp_mc=21500 volt_mv=3912 light_lux=460
```

Before power goes on, a mentor checks the wiring, the battery voltage on the divider (no more than twice the board's reference voltage, as the sketch explains) and that nothing gets hot.

The station's `sensor-bridge` command turns those lines into the files the station already publishes. Unknown keys and out-of-range values are dropped, and a reading older than five seconds is removed so the station reports it missing instead of showing an old number. When the bridge stops, it removes all its files for the same reason.

The classroom station needs all three readings: temperature, voltage and light. If one sensor is missing or fails, the station answers "A sensor input is unavailable" rather than inventing a value.

```sh
# Linux. If the port says "Permission denied", a mentor adds your user to the dialout group.
stty -F /dev/ttyACM0 115200 raw -echo
cargo run --locked -- sensor-bridge --input /dev/ttyACM0 --out run/sensors
# macOS. Find the name with `ls /dev/cu.usbmodem*`, then use that name in both commands.
stty -f /dev/cu.usbmodem1101 115200 raw -echo
cargo run --locked -- sensor-bridge --input /dev/cu.usbmodem1101 --out run/sensors
# In a second terminal, on either system
cargo run --locked -- classroom --sensor-dir run/sensors
```

The sketch compiles for those boards. It has not been run on hardware. Label its readings "hardware bench" and check them against a thermometer and a meter before trusting them. An unplugged sensor can still produce a believable number, so that check matters.

On an Arduino Uno or Nano, SDA and SCL are 5 V signals. A BH1750 board powered from 3.3 V can be damaged by those pins unless the board is 5 V tolerant or a mentor adds a level shifter. Seeed XIAO boards are 3.3 V on both the sensor supply and I2C, which matches the BH1750 directly. A mentor checks the exact module before power is connected.

| Part | Job | Approx. price |
| --- | --- | --- |
| Seeed XIAO ESP32-C3, or an Arduino Uno R3 you already have | Reads the sensors | $5 (XIAO) |
| TMP36 temperature sensor | Temperature, 10 mV per °C | $2 (estimate) |
| BH1750 light sensor board | Light in lux, over I2C | $3 (estimate) |
| Two 100 kΩ resistors and a 100 nF capacitor | Halves a battery voltage so the board can read it | under $1 |
| Breadboard and jumper wires | No soldering | $5–10 (estimate) |

An Arduino Uno draws about 40 mA from USB, about 0.2 W. The XIAO draws less. Either runs all day from a laptop port.

### Raspberry Pi without an Arduino

A Pi can feed the bridge from files it already has. These examples give one or two readings, not the three the classroom station needs, so use them with the balloon logger (which records whatever pressure and temperature it finds) or alongside the Arduino sketch. The CPU temperature is in millidegrees, the same unit the station uses:

```sh
cargo run --locked -- sensor-bridge --out run/sensors \
  --map temp_mc=/sys/class/thermal/thermal_zone0/temp
```

A BME280 sensor on the Pi's I2C pins appears through the Linux IIO driver. Its pressure file is in kilopascals, so scale it by 1000 to get pascals. The path varies by board, so check yours with `ls /sys/bus/iio/devices/`:

```sh
cargo run --locked -- sensor-bridge --out run/sensors \
  --map 'pressure_pa=/sys/bus/iio/devices/iio:device0/in_pressure_input*1000'
# In a second terminal: record a one-minute bench log
cargo run --locked -- balloon-log --sensor-dir run/sensors --output pi-run.csv --samples 60
```

The CPU temperature is the chip's temperature, not the room's. Label it that way.

## Level 2: predict a pass

Satellites can only be heard while they are above the horizon. The `passes` command reads an orbit file and lists the next passes for your location.

1. Download fresh orbital elements in three-line format from [CelesTrak's amateur radio group](https://celestrak.org/NORAD/elements/gp.php?GROUP=amateur&FORMAT=tle) and save the file as `amateur.tle`. Elements older than two weeks give poor times, and the command warns about them.
2. Use your school's approximate latitude and longitude. A town-level position is enough; don't use a home address.

```sh
cargo run --locked -- passes --tle amateur.tle --satellite "ISS" --lat 39.95 --lon -75.16 --hours 24
```

Output lists the rise time, the highest elevation and the set time in UTC, with the compass direction at rise and set. Passes that climb above about 30° are the easiest to hear with a simple antenna. Add `--json` to load the passes into a spreadsheet. For live tracking, Doppler correction and antenna rotators, use Gpredict, the open-source tracker that drives radios and rotators through Hamlib.

## Level 3: receive real satellites

For a lower-cost first build, see the [classroom receiver design](classroom-receiver.md). It starts with a laptop and a 2 m dipole and adds live phone stats over MoQ, a teacher microphone and a standalone speaker.

This is the station the SatNOGS project recommends for beginners: a fixed antenna, a Raspberry Pi and an inexpensive RTL-SDR software-defined radio dongle. SatNOGS lists the Pi 3B, 3B+ and 4 as known to work. The Pi Zero 2 W is fine for this project's station and bridge but too slow for SatNOGS decoding.

| Part | Job | Approx. price |
| --- | --- | --- |
| Raspberry Pi 4, 2 GB | Runs SatNOGS or FoxTelem | $67.50 (list price from October 1, 2026) |
| RTL-SDR Blog V4 | Receives 2 m and 70 cm satellite signals | about $30 (dongle only; launch price, 2023) |
| Antenna: homemade turnstile or QFH | Fixed, sees the whole sky | $20–40 in parts (estimate) |
| Antenna, alternative: Arrow II handheld Yagi | Hand-pointed, more gain, also used by licensed operators | $139.95 without duplexer (GigaParts) |
| 5 V 3 A USB-C supply, 32 GB microSD, coax | Power and storage | $25–35 (estimate) |

Raspberry Pi raised Pi 4 and Pi 5 prices several times in 2025–2026, citing memory costs. It has said Zero 2 W prices are not affected.

Then choose the software for the job:

- **SatNOGS client.** Flash the SatNOGS Raspberry Pi image, register the station on the SatNOGS Network, and the station records scheduled passes and shares them with the worldwide network. A good long-running classroom project.
- **AMSAT FoxTelem.** Decodes telemetry from AMSAT's Fox satellites and from the CubeSatSim. It runs on Windows, Mac and Linux, and a ready-made Pi image boots straight into it.

Keep the RTL-SDR's bias tee turned off unless a powered preamplifier is fitted. SatNOGS warns it can damage the dongle when connected to a loop or eggbeater antenna.

## Level 4: a satellite in the classroom

The [AMSAT CubeSatSim](https://cubesatsim.org) is a working satellite model with solar panels, batteries and a UHF telemetry transmitter. On November 24, 2026 the [AMSAT store](https://www.amsat.org/product/cubesatsim-beta-kit/) listed the Beta kit at $400, including shipping to United States addresses, and it was out of stock. The kit needs a little soldering for the solar-panel wires and the sensor headers. Blank PCB sets were $35 for groups who solder the boards and print the frame. Adding the level 3 station makes the parts budget about $550–700. Check the store again before you buy, including whether it will ship to your address. Receive the model with the level 3 station and FoxTelem, then compare its telemetry with this project's station. AMSAT also lends CubeSatSims for events. The CubeSatSim transmitter is low power, but it transmits on amateur frequencies, so a licensed operator runs it.

## Power budget

Run `python3 tools/energy_budget.py` to recompute these numbers. Replace each estimate with your own USB power meter reading.

| Setup | Draw | 6 hours with 2× margin |
| --- | --- | --- |
| Pi 4 at full CPU load (6.4 W) + RTL-SDR V4 (250–270 mA at 5 V) | about 7.8 W | 93 Wh |
| Pi Zero 2 W (up to 1.2 W) + RTL-SDR V4 | about 2.6 W | 31 Wh |
| Arduino Uno with sensors | about 0.2 W | 2.4 Wh |

For a field day away from mains power, a 12 V 20 Ah battery stores 240 Wh, enough for the Pi 4 station with room to spare after converter losses. Follow your school's battery-handling rules.

## Sources

- SatNOGS wiki: [main page](https://wiki.satnogs.org/), [omnidirectional station](https://wiki.satnogs.org/Omnidirectional_Station_How_To), [SDR](https://wiki.satnogs.org/SDR)
- AMSAT: [FoxTelem](https://www.amsat.org/foxtelem-software-for-windows-mac-linux/), [CubeSatSim kit](https://www.amsat.org/product/cubesatsim-beta-kit/), [CubeSatSim ground station](https://github.com/alanbjohnston/CubeSatSim/wiki/v1.1-3.-Ground-Station)
- [RTL-SDR Blog V4 datasheet](https://www.rtl-sdr.com/wp-content/uploads/2024/12/RTLSDR_V4_Datasheet_V_1_0.pdf)
- Raspberry Pi: [power by model](https://pip-assets.raspberrypi.com/categories/685-app-notes-guides-whitepapers/documents/RP-010139-WP-1-Use-case-specific%20thermal%20performance%20of%20Raspberry%20Pi%20SBCs.pdf), [measured draw](https://pidramble.com/node/27), [2026 prices](https://www.raspberrypi.com/news/a-new-3gb-raspberry-pi-4-for-83-75-and-more-memory-driven-price-increases/), [October 2026 prices](https://www.omgubuntu.co.uk/2026/10/raspberry-pi-2gb-costs-how-much-now/amp)
- [Seeed XIAO ESP32-C3](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/)
- [Arrow II antenna at GigaParts](https://www.gigaparts.com/arrow-ii-dual-handheld-yagi.html)
- [Gpredict](https://packages.debian.org/sid/gpredict)
- [CelesTrak GP data](https://celestrak.org/NORAD/elements/)
