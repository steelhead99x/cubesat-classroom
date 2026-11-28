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
| `ENCODER TEST` | Turn TUNE steadily in one direction for five seconds, then repeat in the other direction. Record forward, reverse and missed counts. |
| `OLED TEST` | Look for a border at all four edges for about 1.5 seconds. The display should then turn off and the console should report both results. |
| `SAFE` | Retry both expander setups and turn the ENGAGE LED off. |
| `HELP` | List the commands. |

The console uses USB Serial/JTAG, leaving XIAO D6 available for the control interrupt. Check that line during reset before adding interrupt-driven encoder handling. The encoder diagnostic polls approximately every 2 ms and reports raw quadrature edges. Forward and reverse describe channel phase order. A missed count means both channels changed between samples and their order is unknown. A full cycle between samples is invisible to this test. Record a slow turn and a fast turn in each direction before choosing a production polling interval. Future firmware needs ADC and DAC audio, encoder-driven tuning, live OLED status and receiver telemetry.

## Check the local oscillator

Connect the onboard LO MON port to a 50 Ω instrument or termination before running the test. Use the non-Sense XIAO ESP32-S3 so the PLL SPI pins are free of the Sense expansion board.

| Command | Nominal LO | VCO divider |
| --- | ---: | ---: |
| `PLL TEST 2M` | 145.800 MHz | 16 |
| `PLL TEST 70CM` | 437.000 MHz | 8 |
| `PLL TEST <kHz>` | 144000–146000 or 435000–438000 kHz, in 1 kHz steps | 16 or 8 |

The console checks that MUXOUT starts low and waits up to 100 ms for digital lock. After lock, the onboard monitor output is available for up to five seconds at the lowest ADF4351 output setting. The test powers down early if lock drops. It then sets the power-down register and drives CE low. Check the serial shutdown results, record the measured frequency and level, and repeat for the other band. The other RF controls and audio remain off throughout the test.

The nominal profiles use the board's 25 MHz reference and the modeled 2.5 mA charge-pump setting. [ADF4351 register map and timing](https://www.analog.com/media/en/technical-documentation/data-sheets/ADF4351.pdf) · [XIAO ESP32-S3 pinout](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/)

## Check the receiver ADC

After verifying the rails, send `ADC CHECK`. It releases the TLV320ADC3140 from hardware shutdown, reads DEV_STS0 and DEV_STS1 over I²C, then reasserts shutdown. Record both status bytes and confirm the console reports `shutdown=Ok(())`. The converter stays in its reset sleep mode during the read. The test checks control wiring and the address strap. Clock and audio capture checks are separate. [TI TLV320ADC3140 datasheet](https://www.ti.com/lit/ds/symlink/tlv320adc3140.pdf)

After `ADC CHECK` passes, use `ADC CLOCK TEST` with an oscilloscope or logic analyzer. The ADC drives 192 kHz FSYNC and 12.288 MHz BCLK for up to five seconds. Read the printed ASI status as a diagnostic and confirm the actual rates and alignment at the ADC and the XIAO. The DAC stays muted. The I²S edge alignment uses TI's zero-offset controller-mode workaround. [TI application note SBAA382](https://www.ti.com/lit/an/sbaa382/sbaa382.pdf)

### Check both I²S data paths

Run `AUDIO BUS TEST` after the clock test. The ADC supplies both clocks. The XIAO's BCLK and FSYNC pins are inputs for both receive and transmit. U14 stays muted throughout the test.

1. Set the logic analyzer to trigger on data at the DAC DIN input before sending the command. Decode Philips I²S with two 32-bit slots per frame. Check for left word `0x40000000` and right word `0xc0000000`.
2. Check that the console reports an RX block and successful TX completion. Compare the first two reported RX words with ADC SDOUT. Verify slot order and the one-bit shift.
3. Confirm `shutdown=Ok(())` and confirm XSMT stayed low. A missing frame-sync edge, timeout, or DMA error fails the test. Continuous audio, fault mute and RF sensitivity still need their own tests.

### Check continuous narrow FM on the bench

Use `FM STREAM TEST` with a known differential I/Q fixture, or `FM RF TEST 2M` and `FM RF TEST 70CM` with a shielded RF source and the normal antenna input. The RF variants tune the same presets as `PLL TEST`, acquire lock, then select the filter and enable the ADC and demodulator. Bias and DAC unmute stay low.

Use a 1 kHz FM test tone. At DAC DIN, decode the two identical 32-bit slots and verify the recovered tone. The two-second diagnostic reports completed frame count and peak digital magnitude. It rejects a frame count below 95% of the nominal 384,000 frames, and stops on a reported PLL unlock, DMA stall, or transmit underrun. Repeat the command and verify the first frames are silent before the recovered audio arrives. Keep a scope record of the RX and TX words. This is a digital path test with muted analog output. Verify the listening path, timing margin, panel behavior and RF sensitivity separately on the first article.
