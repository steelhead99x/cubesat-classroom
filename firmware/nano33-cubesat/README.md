# Nano 33 BLE Sense CubeSat firmware, in Rust

The Rust version of [`../arduino-cubesat`](../arduino-cubesat/arduino-cubesat.ino) for an **Arduino Nano 33 BLE Sense Rev2**. It flies the same bench mission with the same commands, and sends the same `CSC1` lines, so the station's `sensor-bridge` and the [bench CubeSat guide](../../docs/bench-cubesats.md) work unchanged.

- The mission logic (modes, commands, store and forward, telemetry) is the [`cubesat-mission`](../../crates/cubesat-mission/src/lib.rs) crate, tested on a computer with `cargo test -p cubesat-mission`.
- This folder connects it to the board: USB serial is the ground link, the RGB LED is the status light, a jumper from **D2** to **GND** is the remove-before-flight pin, and the built-in sensors are the payload.
- The radio is never switched on. Bluetooth stays off.

**Status:** written 25 November 2026. It compiles for the nRF52840 but has not been run on a board yet. Label every reading "hardware bench" and check it against a thermometer, a weather report and a light meter.

## What it reads

| Sensor | Reading | Notes |
| --- | --- | --- |
| HS3003 | `temp_mc`, `humidity_mpct` | Sits beside the processor, so it reads warm |
| LPS22HB | `pressure_pa` | Station pressure, not sea-level pressure |
| APDS9960 | `light_clear`, `light_r/g/b`; `light_lux` after `CAL` | Counts until calibrated |
| Processor ADC | `volt_mv` | The regulated 3.3 V rail, not a battery |
| BMI270, BMM150 | not read yet | Needs the BMI270's 8 KB configuration upload; use the Arduino sketch for motion |

## Timing

`CONFIG.timing` in [src/main.rs](src/main.rs) sets the wait between pulling the jumper and the first transmission:

- `Timing::Bench`: 30 seconds, for a lesson (the default).
- `Timing::Cds`: 45 minutes, the radio silence the [CubeSat Design Specification](https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf) requires after deployment (Rev 14.1 §2.4.5).

Either way, refitting the jumper puts the CubeSat back to pre-launch and the timer starts again from zero, as §2.3.2.4 requires of a deployment switch.

## Watchdog and reset cause

On orbit nobody can press reset, so a hung satellite stays hung unless the hardware restarts it. The nRF52840's watchdog does that here:

- The main loop and the sensor task each check in. If either stops for **5 seconds** (a sensor holding the I2C bus, a panic, a bug), the watchdog restarts the board. It pauses while a debugger has the processor halted.
- At boot the firmware reads the reset-reason register, prints `# bench: reset=watchdog` (or `power_on`, `reset_pin`, `software`, `lockup`), and `STATUS` repeats it as `reset=`. A watchdog or lockup restart also prints a warning line.
- The Arduino bootloader runs before this firmware and may clear the register. If a reset you caused shows as `power_on`, that is why. Test it on the bench before trusting it.

After a restart with the jumper out, the mission starts the deployment timer again from zero. A flight design would save "deployed" in flash so a restart on orbit doesn't wait another 45 minutes. That is a good design question for a crew.

## Build

```sh
rustup target add thumbv7em-none-eabihf
cd firmware/nano33-cubesat
cargo build --release
```

Then make a flat binary with `rust-objcopy` (from `cargo install cargo-binutils` and `rustup component add llvm-tools`) or `arm-none-eabi-objcopy`:

```sh
rust-objcopy -O binary target/thumbv7em-none-eabihf/release/nano33-cubesat nano33-cubesat.bin
```

## Flash

The firmware is linked at 0x10000, after the Arduino bootloader, so the bootloader stays in place and you can always go back to Arduino sketches.

1. **Double-tap the reset button.** The orange LED pulses: the bootloader is waiting.
2. Upload with the `bossac` that the Arduino IDE installed for this board (under `~/.arduino15/packages/arduino/tools/bossac/1.9.1-arduino2/` on Linux and macOS):

   ```sh
   bossac -d --port=/dev/ttyACM0 -U -i -e -w nano33-cubesat.bin -R
   ```

   On Windows the port is `COM3` or similar. On a Mac it's `/dev/cu.usbmodem…`.
3. Open a serial terminal at 115200 baud with the line ending set to **Newline**, the same way as with the Arduino sketch.

The board then shows up as `nano33-cubesat` (USB ID 1209:0001). To flash again, or to go back to the Arduino IDE, double-tap reset first.
