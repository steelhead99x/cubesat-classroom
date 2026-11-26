# CSC-R1 Rev B plan: satellite stompbox with HaLow

Rev B of the [CSC-R1 receiver](README.md) goes in a guitar-pedal enclosure, follows pedal design conventions, and adds a Wi-Fi HaLow radio so it can send its stream long distances when no ordinary Wi-Fi is in reach. Rev A stays an open dev kit.

**Status: plan and faceplate concept only.** No Rev B schematic or layout exists. Rev B should not be laid out until Rev A has been built and measured. Everything below is desk design.

Date: 25 November 2026.

## How it sends its stream

The receiver itself never fans out to many viewers. It sends **one copy** of its stream, and a MoQ relay does the fan-out, as in the [classroom receiver design](../../docs/classroom-receiver.md).

| Order | Link | Comes from | Typical use |
| --- | --- | --- | --- |
| 1 | USB | XIAO ESP32-S3 | On the bench, to a laptop running SDR++ or SatDump |
| 2 | 2.4 GHz Wi-Fi | Every ESP32 XIAO | Station and relay in the same building |
| 2 | 5 GHz Wi-Fi | XIAO ESP32-C5 only | Same, on a less crowded band |
| 3 | **Wi-Fi HaLow, 902–928 MHz** | On-board MM6108 module | **Long-range fallback**: roof to another building, or no Wi-Fi in reach |
| none | Bluetooth LE | Every ESP32 XIAO, and nRF XIAOs | Setup and status from a phone, not streaming |

HaLow carries ordinary IP, so the same QUIC/MoQ publisher runs over it unchanged. On the far end it joins the station's HaLow access point or a classroom XIAO acting as one, and from there a MoQ relay (classroom relay or moq.pro) fans out to phones, browsers and other classrooms.

**What fits over HaLow:** the project's lab boards measured about 2.7–3.3 Mbit/s at 4 MHz bandwidth, with the boards centimetres apart. Raw I/Q at 192 kHz is about 6.1 Mbit/s, so it does not fit. Over HaLow the pedal sends what it has already demodulated: the `stats` track, `receive-audio` (Opus) and, if there is room, decimated I/Q (48 kHz of I/Q is about 1.5 Mbit/s). Raw I/Q stays on USB and ordinary Wi-Fi.

HaLow is also the candidate **low-power link**, for a station on battery or solar. That is a claim to test, not assume; see the next section.

## Power comparison: 2.4 GHz vs 5 GHz vs HaLow

Goal: measure which link costs the least energy for the same stream, so a crew can pick one for a battery or solar station. No figures exist yet.

**What Rev B adds for it:** two I²C power monitors (INA226 class, part and addresses to confirm), one on the XIAO's 5 V feed and one on the HaLow module's 3.3 V feed. The firmware logs voltage, current and energy once a second and can publish them as their own track. On Rev A, the XIAO's current can already be measured through its 5 V jumper with an external meter.

**Test method:**

| Step | Detail |
| --- | --- |
| Same stream on every link | `stats` + `receive-audio` (Opus), then the same with decimated I/Q |
| Same placement | Fixed distance and antennas, then repeat at two longer distances |
| Links | XIAO ESP32-S3 on 2.4 GHz; XIAO ESP32-C5 on 2.4 and 5 GHz; MM6108 HaLow at 1, 2 and 4 MHz bandwidth |
| States | Off, associated and idle, streaming; with and without each radio's power-save mode |
| Record | Average power (mW), energy per delivered megabyte (mJ/MB), delivered throughput, packet loss, and the receiver's sensitivity at the same time (desense) |
| Duration | At least 10 minutes per case; repeat each case three times |
| Label | Hardware bench, with date, firmware revision and XIAO model |

Compare the radios' own rails, not the whole pedal: the receiver chain draws far more than any radio and would hide the difference. Report the whole-pedal figure separately.

## HaLow radio

| Item | Choice |
| --- | --- |
| Chip | Morse Micro MM6108, the chip the project's Rust HaLow firmware already drives on the lab boards |
| Module | Quectel FGH100M-H (FCC and CE certified module, MM6108 inside). The same module is on Seeed's Wi-Fi HaLow board for XIAO. |
| Host link | SPI from the XIAO |
| Software | Morse Micro's ESP-IDF component `morsemicro/halow` (Apache-2.0; MM6108 tested on ESP32-C3, C5, C6 and S3; STA, AP and both at once), under the project's Rust firmware as in `steelhead99x/cubesat-classroom-rust` |
| Antenna | Its own SMA on the far end of the box from the receive antenna |

### Which features each XIAO gives

| XIAO | 2.4 GHz | 5 GHz | BLE | HaLow | USB streaming | Audio OUT |
| --- | --- | --- | --- | --- | --- | --- |
| ESP32-S3 | yes | no | yes | yes | yes | no (pins) |
| ESP32-S3 Plus | yes | no | yes | yes | yes | yes, on its extra pins |
| ESP32-C5 | yes | yes | yes | yes | no | no (pins) |
| ESP32-C6 | yes | no | yes | yes | no | no (pins) |
| ESP32-C3 | yes | no | yes | yes | no | no (pins) |
| nRF54L15, nRF52840 | no | no | yes | no | no | no |
| RP2350 | no | no | no | no | no | no |

HaLow is "yes" only where Morse Micro's microcontroller software runs: the ESP32 family. The C3, C5 and C6 have no USB device controller for streaming, only a serial/debug port. Every XIAO still tunes the receiver and drives the knobs and display.

### XIAO pins with HaLow on board

An 11-pin XIAO has to carry the ADC's I²S, I²C, the synthesizer and the HaLow SPI link. It fits only by sharing:

| XIAO pin | Signal |
| --- | --- |
| D0, D1, D2 | I²S bit clock, frame sync, ADC data |
| D3 | HaLow IRQ |
| D4, D5 | I²C SDA, SCL |
| D6 | HaLow BUSY |
| D7 | HaLow chip select |
| D8 | SPI clock (HaLow and synthesizer CLK) |
| D9 | SPI MISO (HaLow) |
| D10 | SPI MOSI (HaLow and synthesizer DATA) |

- **The synthesizer shares the SPI bus.** The ADF4351 only loads its shift register into its settings on a rising edge of LE, so HaLow traffic on the shared clock and data lines leaves it unchanged. LE moves to the I²C expander. To retune, the firmware pauses HaLow traffic, sends the 32-bit word, then pulses LE. Retuning is slow (an I²C write per register), which is fine for Doppler steps.
- **Slow lines move to the expander:** synthesizer LE and lock detect, HaLow RESET_N and WAKE, and the lid board's interrupt. The main board's 8-bit TCA9534 becomes a 16-bit TCA9535.
- **Audio OUT has no pin left** on 11-pin XIAOs. The S3 Plus gives it one from its extra pads. On other XIAOs the pedal is silent but still streams.
- The ADC master clock comes from the on-board oscillator.

## Pedal conventions

| Convention | How Rev B follows it |
| --- | --- |
| Signal flows right to left | Receive antenna IN (SMA) on the right side, audio OUT on the left |
| 1/4 inch jacks | OUT is a 1/4 inch jack, for an amp, a mixer or the next pedal |
| 9 V DC, 2.1 mm barrel, center-negative | Power jack on the top end, so it runs from a pedalboard supply. USB-C also powers it. |
| Knobs at the top, footswitch at the bottom | Keeps a foot clear of the knobs |
| Status LED above the footswitch | Lit when the receiver is engaged |
| Mini toggle for modes | BAND toggle, 2 m / 70 cm |
| Standard enclosure | Hammond 125B-style die-cast aluminum box |
| Printed enclosure art | A faceplate print per batch |

"True bypass" does not apply: there is no audio input to pass through, so the footswitch mutes in firmware.

## Enclosure

- 125B-style box. Inside is roughly 118 × 62 mm from memory; confirm against the drawing before layout.
- The box is the shield and the heatsink (thermal pad from the demodulator to the wall).
- Top end: 9 V DC jack, USB-C, and the HaLow antenna SMA.
- The XIAO's own Wi-Fi and BLE antenna also has to leave the box. The XIAO ESP32-S3 and C5 have a U.FL connector for an external antenna; a pigtail to a panel SMA, or a small antenna window, is needed. Decide per XIAO.
- Good for the first tens to low hundreds of units. For larger volumes, the same board should drop into an extruded enclosure with machined end plates.

## Controls

| Control | Part type | Job |
| --- | --- | --- |
| TUNE | Rotary encoder with push switch, large knob | Turn to tune. Push to change step size. |
| GAIN | Potentiometer | Front-end drive: sets the ADC gain, 0–42 dB |
| LEVEL | Potentiometer | Output volume |
| BAND | Mini toggle switch | 2 m / 70 cm |
| ENGAGE | Soft-touch momentary footswitch + status LED | Tap to mute or unmute. Hold for the next preset (ISS 145.800 MHz first). |
| Display | Small I²C OLED | Preset, mode, signal strength, frequency, link in use (USB, Wi-Fi, HaLow), mini spectrum |
| SIGNAL | RGB LED | Brightens with signal strength |

## Electronics added in Rev B

Controls go on a small lid board, wired to the main board by one cable, so the RF layout is not disturbed.

| Block | Approach | Notes |
| --- | --- | --- |
| HaLow | Quectel FGH100M-H on the main board, in its own shielded corner | SPI to the XIAO; see pins above |
| Encoder, toggle, footswitch, LEDs | Second TCA9534 I²C expander on the lid at 0x21 | Interrupt goes to the main expander |
| Pots (GAIN, LEVEL) | Small I²C ADC (part to pick) | Check its address against 0x20, 0x21, 0x4C, 0x60, 0x3C |
| OLED | SSD1306-class I²C display | Usually 0x3C; confirm the module |
| Audio out | I²S DAC plus an output buffer for the 1/4 inch jack (parts to pick) | Only with the S3 Plus |
| 9 V input | Reverse-polarity protection and a 9 V to 5 V buck regulator (part to pick) | See risks |
| Power monitors | 2 × INA226-class I²C monitors: XIAO 5 V feed, HaLow 3.3 V feed | For the power comparison; addresses to confirm |
| Hacker ports | Qwiic/STEMMA I²C connector and a badge add-on (SAO) header, inside the box | |

## Firmware added in Rev B

- HaLow bring-up through the project's existing Rust MM6108 work
- Link choice: USB, then XIAO Wi-Fi, then HaLow, with the active link on the display, or a forced low-power choice
- Power logging from the two monitors
- QUIC/MoQ publisher for `stats`, `receive-audio` and, where it fits, I/Q
- FM demodulation on the ESP32 for audio and for the `receive-audio` track
- Encoder, pots, toggle and footswitch; presets; OLED interface; signal LED
- Everything in Rev A's firmware plan (none of which is written yet)

## Main-board changes for Rev B

- Shrink to fit the 125B (target about 100 × 60 mm; Rev A is 116 × 78 mm)
- Add the HaLow module and its SMA, and a TCA9535 in place of the TCA9534
- Receive SMA on the right edge; power, USB-C and HaLow SMA on the top edge
- Drop the dev-kit break points and current jumpers, or leave them unpopulated
- Add the lid-board connector and the 9 V input stage
- Apply whatever Rev A testing shows (matching, filters, baseband gain)

## Risks

1. **HaLow transmits next to the receiver while it is receiving.** The pedal's whole job is to stream during a pass, so the HaLow transmitter (902–928 MHz) is on while the receiver listens for very weak signals at 145 and 435 MHz, in the same metal box. The modelled 70 cm filter gives only about 21 dB of rejection at 900 MHz. Plan for:
   - a sharp low-pass filter (cut-off around 600 MHz) at the receive input, before the LNA
   - the HaLow module in its own shielded compartment
   - the two antennas at opposite ends of the box, or the HaLow antenna on a short cable away from it
   - firmware that marks I/Q samples taken while HaLow is transmitting

   Measure desense early: put a XIAO ESP32-S3 with Seeed's Wi-Fi HaLow board for XIAO next to a Rev A kit and compare sensitivity with HaLow idle and busy.
2. **The XIAO's 2.4 and 5 GHz transmitters** are further from the receive bands, but their clocks and harmonics still need a spur check (the 40 MHz crystal's 11th harmonic is at 440 MHz).
3. **Pedalboard power may not be enough.** Rev A's parts draw roughly 0.5–0.8 A at 5 V. HaLow transmit adds more. Through a buck regulator that is roughly 0.4–0.6 A at 9 V. Many pedal supply outlets give 100–500 mA, so the pedal needs a high-current outlet. Measure Rev A's real draw first.
4. **The buck regulator is a noise source** next to a sensitive receiver. It needs filtering and a switching frequency away from the baseband. USB-C power bypasses it.
5. **HaLow throughput** is the lab boards' close-range figure. The distance test has not been run.
6. **Pin sharing** (synthesizer on the HaLow SPI bus) needs care in firmware, and audio OUT needs the S3 Plus.
7. **Certification grows.** Rev B now has intentional transmitters (the HaLow module and the XIAO's radios). Both use certified modules, but the product still needs integration testing, and each module's grant limits which antennas can be used.
8. Lid holes leak a little RF. Use metal-shaft controls and ground the lid.
9. More parts and a second board raise the unit cost. Not yet estimated.

The receiver itself remains receive-only. HaLow and Wi-Fi are Part 15 ground links that carry the received stream; they never carry anything to an amateur transmitter.

## Order of work

1. Build and measure Rev A, including its current draw.
2. Run the HaLow desense test above with Seeed's HaLow board for XIAO beside a Rev A kit, and a first power comparison with an external meter.
3. Breadboard the lid controls against a Rev A kit through its controller and AUX headers.
4. Write the standalone firmware on that setup, reusing the Rust HaLow work.
5. Pick the open parts (pot ADC, audio DAC and buffer, buck regulator, receive low-pass filter).
6. Lay out the Rev B main board and lid board; drill a 125B by hand for the first prototype.
7. Certify after the prototypes work.
