# CSC-R1P: the receiver in a pedal box

CSC-R1P is the [CSC-R1 Rev A](../README.md) receiver rebuilt to fit a guitar-pedal box, so the first board built is already the pedal. It receives the 2 m and 70 cm amateur satellite bands and plays the audio out of a ¼ inch jack. It only listens: there is no HaLow radio on this board. The XIAO's own Wi-Fi does transmit when the firmware turns it on.

**Status:** the control board is routed and passes the KiCad 10.0.8 design-rule check. The main board still has open connections. Both boards need independent layout review and a physical fit check before fabrication.

| | |
| --- | --- |
| Box | Hammond 1590N1 ("125B"), 121.2 × 65.5 × 39.8 mm outside, die-cast aluminium |
| Main board | 54 × 104 mm, 4 layers (In1 is a solid ground plane), notched around the lid posts, with a slot for the footswitch body |
| Control board | 54 × 72 mm, 4 layers, hung from the face by the pot and encoder nuts |
| Power | 9 V centre-negative, the guitar-pedal standard |
| Out | ¼ inch mono jack, line level, from a PCM5102A DAC |
| Antenna | SMA bulkhead on the right side, U.FL pigtail to the board |
| Simulator | `cargo run --locked --bin cubesat-classroom -- receiver-sim --source pedal` publishes what this pedal would report ([contract](../../../docs/classroom-receiver.md#stats-version-1)); the website's [pedal trainer](https://cubesatclass.org/#pedal-lab) is drawn from the face template here |

## Files

| Path | What |
| --- | --- |
| [face-template.svg](face-template.svg) | 1:1 drilling template for the face and the three drilled walls. Print at 100 % and check the 10 mm scale bar. |
| `kicad/csc-r1p.*` | Main board: schematic, board, project |
| `kicad/csc-r1p-ctl.*` | Control board |
| `kicad/stack.json`, `kicad/face.json` | Where the stacking header, standoffs, shafts, LED and OLED sit, written by the main-board placer and checked by the control-board placer |
| [`../gen/design_r1p.py`](../gen/design_r1p.py), [`design_r1p_ctl.py`](../gen/design_r1p_ctl.py) | The two designs (parts and nets) |
| [`../gen/gen_pcb_r1p.py`](../gen/gen_pcb_r1p.py), [`gen_pcb_r1p_ctl.py`](../gen/gen_pcb_r1p_ctl.py) | The two placers, with [`pcbkit.py`](../gen/pcbkit.py) |
| [`../gen/check_shared.py`](../gen/check_shared.py) | Checks that the pedal's RF, demodulator, synthesizer and ADC blocks are the same circuit as Rev A's, apart from a listed set of differences |

## What changed from Rev A

The radio is the same circuit: bias tee, PGA-103+ LNA, the two switched band filters, LTC5584 I/Q demodulator, ADF4351 synthesizer with its 25 MHz TCXO, MCP4728 trim DAC and TLV320ADC3140 ADC. `check_shared.py` compares them part by part and reports "identical apart from the listed differences":

- The antenna input is a U.FL for a pigtail to the wall-mounted SMA, not an edge SMA. The BP1 test break is gone.
- The dev-kit extras are gone: current-measurement jumpers, CTRL and AUX headers, the baseband test header, the MCLK jumper, the user LED, the lock LED and the LD and PDBRF test points. The synthesizer's lock output still reaches the XIAO on PLL_MUXOUT.

Added for the pedal:

- **9 V input**: JST-PH to the panel jack, AO3401A reverse-polarity FET with a 10 V gate zener, SMAJ15A clamp, 1 A PTC fuse and ferrite, then a TPS62933F buck at about 2.1 MHz (forced PWM, so its switching tone stays at one frequency) making 5.09 V. Five LP5907 3.3 V regulators feed the VCO, synthesizer, ADC, digital and audio rails.
- **Audio**: PCM5102A on the same I²S clocks as the ADC, its own 3.3 V rail, 470 Ω / 2.2 nF output filter, to the OUT jack on a JST-PH. It starts muted (XSMT pulled low) until the firmware unmutes it.
- **Control board**: SSD1306 OLED, EC11E encoder for TUNE (push to change step), Alpha 9 mm pots for GAIN and LEVEL read by an ADS1015, a second TCA9534 for the encoder, footswitch, BAND toggle and ENGAGE LED.

## Inside the box

```
 face (top of the box when it sits on the floor)
 ─────────────────────────────────────────────
   pots + encoder bushings through the face; nuts hold the control board
 ┌──────────── control board, 54 × 72 ─────────┐   OLED on top, behind the window
 └───────────────▲─────────────────────────────┘
          2×4 socket (underside) on 2×4 header
 ┌──────────── main board, 54 × 104 ───────────────────────┐   on three M2.5 standoffs
 └─────────────────────────────────────────────────────────┘
 ─────────────────────────────────────────────  back plate
```

- The main board sits on three 15 mm M2.5 standoffs at (4.6, 62.6), (50.4, 23.3) and (50.4, 66.2) mm from its top-left corner. The control board has holes in the same three places.
- The stacking header is at (42.4, 59.3), pin 1. The control-board placer reads `stack.json` and stops unless all 8 socket pins land on the header pins.
- The notches clear the lid posts by 5.5 mm. The footswitch body drops through the 16 mm slot at the bottom of the main board.
- Everything on the panel is wired to the boards with JST-PH leads, so the boards come out without desoldering: DC jack, OUT jack, footswitch, toggle, and the two U.FL pigtails. USB-C reaches the XIAO through a short panel-mount extension.

## Panel

All positions are from [face-template.svg](face-template.svg), measured from the outside top-left corner of the face with the jacks end at the top.

| Hole | Position (mm) | Size |
| --- | --- | --- |
| GAIN, TUNE, LEVEL shafts | (15.8, 20.0), (32.8, 20.0), (49.8, 20.0) | 7.5 mm |
| OLED window | dashed outline on the template | check against your module |
| ENGAGE LED | (32.8, 55.6) | 5 mm, for a 3 mm LED bezel |
| BAND toggle | (45.75, 87.6) | 6.5 mm |
| ENGAGE footswitch | (32.75, 100.6) | 12.2 mm |
| Top wall | DC 9 V at 20 mm (8 mm), USB-C at 36 mm (cut-out), Wi-Fi RP-SMA at 50 mm (6.5 mm) | |
| Left wall | OUT at 40 mm from its left end, seen from outside (9.5 mm) | |
| Right wall | ANT SMA at 40 mm from its left end, seen from outside (6.5 mm) | |

Check every hole against the parts in hand before drilling. Bushing diameters vary between makers.

## Pins and addresses

XIAO ESP32-S3 (sockets J9 and J10):

| Pin | Net | Job |
| --- | --- | --- |
| D0, D1 | I2S_BCLK, I2S_FSYNC | I²S clocks, shared by the ADC and the DAC (64 × 192 kHz) |
| D2 | I2S_DOUT | I/Q samples from the ADC |
| D3 | PLL_MUXOUT | Synthesizer lock detect |
| D4, D5 | SDA, SCL | I²C to both expanders, the trim DAC, the ADC's control port, the pot ADC and the OLED |
| D6 | CTL_INT | Control-board expander interrupt |
| D7 | XD7 | Audio samples to the DAC |
| D8, D9, D10 | PLL_CLK, PLL_LE, PLL_DATA | Synthesizer serial port |

I²C:

| Address | Part | Board |
| --- | --- | --- |
| 0x20 | TCA9534: P0 BAND_SEL, P1 BIAS_EN, P2 DEMOD_EN, P3 DEMOD_EDC, P4 DEMOD_EIP2, P5 ADC_SHDNZ, P6 PLL_CE, P7 DAC_XSMT | main |
| 0x21 | TCA9534: P0 ENC_A, P1 ENC_B, P2 ENC_SW, P3 FOOT_SW, P4 BAND_SW, P5 ENGAGE LED, P6–P7 spare | control |
| 0x3C | SSD1306 OLED | control |
| 0x48 | ADS1015: AIN0 GAIN pot, AIN1 LEVEL pot | control |
| 0x60 | MCP4728 trim DAC (Rev A default) | main |
| 0x4C | TLV320ADC3140 control port (Rev A address) | main |

Stacking header: 1 3V3_DIG, 2 GND, 3 SDA, 4 SCL, 5 CTL_INT, 6 GND, 7 +5V, 8 GND.

## Routing

KiCad 10.0.8 results for the PCB files in Git, using the checked-in project rules:

| Board | DRC violations | Unconnected items | Status |
| --- | ---: | ---: | --- |
| Main | 0 | 393 | Placement and initial RF traces |
| Control | 0 | 0 | 381 track segments, 22 vias and four filled GND pours |

The earlier maze-router estimates (about 15 opens on the main and one on the control) came from trial routes. The control board now uses Freerouting copper with the project’s ground pours and silkscreen. The project’s schematic and PCB checks agree on all 19 nets and 33 footprints. A test export produced Gerbers, drill, BOM and placement files. KiCad’s optional schematic-parity check still flags pre-existing generator differences in net names, the intentionally unused ADC alert pin, MPN fields and mounting-hole BOM attributes.

A main-board routing trial kept the original RF traces and left 30 open items and 15 clearance violations in KiCad. The trial remains outside the design files. For the main board, preserve the hand-routed RF and the In1 ground plane. Review placement and escapes around the ADF4351, especially PLL_LE, PLL_DATA, PLL_CLK, PLL_CE, MUXOUT, supply pin 10 and the TCXO output. Refill the finished routes and run the DRC gate before export.

An independent reviewer must check both boards before ordering. See [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## Open items

- Finish main-board routing and pours. Review the copper and prepare Gerbers, BOM and placement files for prototype fabrication.
- Stack height: check the 15 mm standoffs, the 2×4 header and socket, and the pot bushing length against the 35 mm inside height with real parts.
- Panel jacks and switches are not on the BOM yet: DC jack, ¼ inch jack, SMA and RP-SMA bulkheads, U.FL pigtails, USB-C panel extension, footswitch, toggle, LED bezel, knobs.
- The TCXO land pattern is still generic, as on Rev A.
- The buck's switching frequency (about 2.1 MHz) and its harmonics need checking against the 2 m and 70 cm bands on the bench with the [desense quest](../../../quest/m1/receiver-rev-a-desense.md).
- Reading the encoder through an I²C expander works at hand speed; confirm in firmware that fast spins don't skip.
- Firmware for the pedal: not started. The station simulator shows what it should publish.

## Rebuilding

```sh
cd hardware/sdr-receiver/gen
CSC_BOARD=csc-r1p ./build.sh        # main board
CSC_BOARD=csc-r1p-ctl ./build.sh    # control board (place the main board first: it reads stack.json)
python3 check_shared.py             # pedal RF blocks against Rev A
python3 face_template.py            # drilling template
```

These commands rebuild PCB files from placement and run the maze router. Run the control-board rebuild in a separate checkout to preserve the routed copper in Git. To verify the checked-in boards, run `CSC_BOARD=csc-r1p-ctl python3 drc.py gate` and `CSC_BOARD=csc-r1p python3 drc.py gate` from the generator directory. The main board currently fails the connectivity gate.
