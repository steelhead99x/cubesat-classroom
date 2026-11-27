# [L] Write the CSC-R1 Rev A bring-up firmware

## Goal

Write bring-up firmware in Rust for a XIAO ESP32-S3 in the CSC-R1 Rev A kit. It should drive each block on its own from serial commands, so the bench quests can test one block at a time. Use the build setup of the HaLow firmware in `steelhead99x/cubesat-classroom-rust`. No firmware exists yet.

## Plan

Pins, I²C addresses and expander bits are in the [receiver README](/hardware/sdr-receiver/README.md). Every command answers with one line a script can read.

| Command | Does |
| --- | --- |
| `SCAN` | Lists I²C devices. Expect 0x20 (expander), 0x4C (ADC) and 0x60 (DAC). |
| `BAND 2M` / `BAND 70CM` | Sets expander bit P0 |
| `LO <MHz> [power 0–3]` | Programs the ADF4351 (÷16 for 2 m, ÷8 for 70 cm) and reports lock from MUXOUT |
| `DEMOD ON/OFF`, `BIAS ON/OFF` | Expander bits P2 and P1. Bias stays off at power-up. |
| `TRIM <ch> <code>` | Sets an MCP4728 channel (DC offset and IIP2 trims) |
| `ADC START <gain dB>` | Starts the TLV320ADC3140 at 192 kHz as clock master (MCLK on GPIO1), I/Q on I²S |
| `STREAM USB` | Sends I/Q to the computer in a format an SDR app can open, or as raw files for `csc-sweep`-style analysis |
| `STATUS` | Band, LO, lock, gain, temperature, and which blocks are on |

### How to check

- [ ] `SCAN` finds all three devices on a powered kit.
- [ ] `LO 437` and `LO 145.8` both report lock, and the V4 sees the LO at the monitor port (see [LO and 5 V check](/quest/m1/receiver-rev-a-lo-and-supply.md)).
- [ ] With the [test signal](/quest/m0/receiver-test-signal.md) at break point 3, the streamed I/Q shows a tone at the expected offset. Q's sign accounts for the LO polarity swap in the README.
- [ ] Nothing switches the bias tee on without a command.
- [ ] The code is in a pull request with an `Assisted-by:` line if an AI helped, reviewed by someone other than the author.

## Required

- [Bring up CSC-R1 power and control](/quest/m1/receiver-rev-a-power-up.md) - the rails and I²C are proven before the radio blocks are driven.
