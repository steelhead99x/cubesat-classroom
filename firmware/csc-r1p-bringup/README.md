# CSC-R1P pedal bring-up

The Rust console runs on the Seeed XIAO ESP32-S3 in the [CSC-R1P pedal](../../hardware/sdr-receiver/pedal/README.md). It sets both I²C expanders to known states, then lets you check the switches, pots, ENGAGE LED and OLED over USB. After successful initialization it keeps the bias tee, demodulator, ADC and PLL disabled and the audio muted. The PLL is enabled only during an explicit bench test. The PCB pulls PLL CE high until firmware can take control.

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
| `OLED TEST` | Look for a border at all four edges for about 1.5 seconds. The display should then turn off and the console should report both results. |
| `SAFE` | Retry both expander setups and turn the ENGAGE LED off. |
| `HELP` | List the commands. |

The console uses USB Serial/JTAG, leaving XIAO D6 available for the control interrupt. Check that line during reset before adding interrupt-driven encoder handling. Future firmware needs ADC and DAC audio, encoder tracking, live OLED status and the station's receiver telemetry.

## Check the local oscillator

Connect the onboard LO MON port to a 50 Ω instrument or termination before running the test. Use the non-Sense XIAO ESP32-S3 so the PLL SPI pins are free of the Sense expansion board.

| Command | Nominal LO | VCO divider |
| --- | ---: | ---: |
| `PLL TEST 2M` | 145.800 MHz | 16 |
| `PLL TEST 70CM` | 437.000 MHz | 8 |

The console checks that MUXOUT starts low and waits up to 100 ms for digital lock. After lock, the onboard monitor output is available for five seconds at the lowest ADF4351 output setting. It then sets the power-down register and drives CE low. Check the serial shutdown results, record the measured frequency and level, and repeat for the other band. The other RF controls and audio remain off throughout the test.

The nominal profiles use the board's 25 MHz reference and the modeled 2.5 mA charge-pump setting. [ADF4351 register map and timing](https://www.analog.com/media/en/technical-documentation/data-sheets/ADF4351.pdf) · [XIAO ESP32-S3 pinout](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/)
