# CSC-R1P pedal bring-up

The Rust console runs on the Seeed XIAO ESP32-S3 in the [CSC-R1P pedal](../../hardware/sdr-receiver/pedal/README.md). It sets both I²C expanders to known states, then lets you check the switches, pots and ENGAGE LED over USB. It keeps the bias tee, demodulator, ADC and PLL disabled and the audio muted after successful initialization. The PCB pulls PLL CE high until firmware can take control.

## Build

Install the Espressif Rust toolchain with `espup install` and source the environment file that espup reports. From this directory build and flash the XIAO with espflash:

```sh
cargo build --release --locked
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/csc-r1p-bringup
```

If Cargo uses a custom target directory, use that directory in the flash command. For the host tests, run from the repository root:

```sh
cargo +stable test --locked --manifest-path firmware/csc-r1p-bringup/Cargo.toml --lib
```

## Check the first board

1. With the XIAO and control board removed, check the unpowered board for shorts. Use a current-limited 9 V supply and verify the 5 V and 3.3 V rails. Switch off before installing either board.
2. Install the XIAO, connect USB and power the main board. In the serial terminal, send one uppercase command per line. Start with `SAFE` and confirm the main expander reports `RF safe=Ok(())`. Confirm that the bias feed stays off and audio stays muted. With no control board installed, the console reports an I²C error for the controls.
3. Switch off and unplug USB before stacking the control board. Check all eight pins against pin 1 in the board guide. Restore power and USB. Send `SAFE` again. Both results should now show `Ok(())`.
4. Use the commands below to check the panel. Record the board revision, supply current, rail voltages and control responses.

| Command | What to check |
| --- | --- |
| `CONTROLS` | Send it before and while holding each switch. Each `true` means the input is low. BAND closed selects 70 cm on the panel. Step the encoder slowly to sample A and B. |
| `POTS` | Turn GAIN and LEVEL separately. Each should change its own raw counts. `Ok(None)` means conversion timed out. |
| `LED ON` / `LED OFF` | Check that the ENGAGE LED follows the command. |
| `SAFE` | Retry both expander setups and turn the ENGAGE LED off. |
| `HELP` | List the commands. |

The console uses USB Serial/JTAG, leaving XIAO D6 available for the control interrupt. Check that line during reset before adding interrupt-driven encoder handling. Future firmware needs PLL lock detection, ADC and DAC audio, encoder tracking, OLED status and the station's receiver telemetry.
