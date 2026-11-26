# CSC-R1 dev kit, Rev A

Receive-only SDR development kit for the 2 m and 70 cm amateur satellite bands (144–146 MHz, 435–438 MHz), built around a Seeed XIAO.

Part of the [cubesat-classroom hardware](../README.md). Results here are **desk design only**: nothing has been built or measured.

**Status: designed and rule-checked. Not built, not tested, not reviewed by an RF engineer. Do not order boards before working through "Review findings" below.**

Date: 25 November 2026 (Rev A, fix round 1). Tool: KiCad 7.0.11.

## What is in this package

| Path | What it is |
| --- | --- |
| `kicad/csc-r1.kicad_pro`, `.kicad_sch`, `.kicad_pcb` | KiCad 7 project. Import these into Flux. |
| `kicad/CSC_R1.kicad_sym`, `kicad/sym-lib-table` | Project symbols for the LTC5584, TLV320ADC3140 and PE4259 |
| `kicad/CSC_R1.pretty/`, `kicad/fp-lib-table` | Project footprint for the LTC5584 (UF24 package, to its drawing) |
| [`kicad/csc-r1-schematic.pdf`](kicad/csc-r1-schematic.pdf) | Schematic. Nets are joined by label name, not by drawn wires. |
| [`kicad/csc-r1-layout-top.pdf`](kicad/csc-r1-layout-top.pdf), [`kicad/csc-r1-top.png`](kicad/csc-r1-top.png) | Top copper and silkscreen |
| `fab/gerbers/` | Gerbers for 4 copper layers, mask, paste, silk, outline, and the drill file |
| [`fab/csc-r1-bom.csv`](fab/csc-r1-bom.csv) | Bill of materials (103 lines, 212 fitted parts, 5 not fitted) |
| `fab/csc-r1-cpl-top.csv` | Pick-and-place positions. All parts are on the top side. |
| [`fab/drc-report.txt`](fab/drc-report.txt) | KiCad design rule check report |
| [`bench/`](bench/README.md) | Bench scripts, a power-logger sketch, a shopping list and the bench quests for measuring the kit with an RTL-SDR V4 and other low-cost gear |
| `gen/` | The Python that generates everything. `gen/build.sh` rebuilds it all; `gen/rf_checks.py` repeats the desk calculations below. |

![CSC-R1 Rev A top side](kicad/csc-r1-top.png)

## The design

Signal chain: SMA → bias tee → PGA-103+ LNA → band filters (2 m or 70 cm, switched by two PE4259) → balun → LTC5584 I/Q demodulator → RC filter → TLV320ADC3140 ADC → I²S → XIAO.

Band filters are 0.1 dB Chebyshev designs: 2 m is a 7th-order high-pass at 125 MHz plus a 5th-order low-pass at 185 MHz; 70 cm is a 5th-order high-pass at 370 MHz plus a 7th-order low-pass at 520 MHz.

The local oscillator is an ADF4351 with a 25 MHz TCXO. An MCP4728 DAC drives the demodulator's four trim pins. A TCA9534 expander drives the band switch, bias tee and enables.

- **Board:** 120 × 78 mm, 4 layers, 1.6 mm. Layer 2 is an unbroken ground plane. Minimum track 0.2 mm, minimum clearance 0.16 mm, vias 0.6/0.3 mm, thermal vias 0.2 mm drill.
- **Stackup to order:** 1.6 mm, 4 layers, with about 0.21 mm of 7628 prepreg (εr about 4.4) between layers 1 and 2. The 0.35 mm RF tracks are 50 Ω on that stackup (calculated). A thinner first prepreg makes them about 32 Ω, so check the board maker's stackup before ordering.
- **Power:** 5 V from USB-C or a screw terminal, selected by jumper. Use a supply of at least 1.5 A. Four LP5907 3.3 V regulators (VCO, PLL, ADC, digital). USB VBUS has an SMAJ5.0A surge clamp; the screw terminal has P-channel MOSFET reverse-polarity protection (AO3401A, about 50 mΩ).
- **Demodulator supply jumper (I_DEMOD / SRC):** 1-2 feeds the LTC5584 from the kit's fused 5 V; 2-3 feeds it straight from the screw terminal, past the fuse, for a bench supply set to 5.0–5.2 V.
- **I²C addresses:** ADC 0x4C, DAC 0x60, expander 0x20.

### Dev-kit features

- Three RF break points (antenna/LNA, LNA/filters, filters/demodulator). Each is a 0 Ω link with a U.FL on each side.
- U.FL monitor port on the synthesizer's second output.
- Six current-measurement jumpers, one per supply rail.
- 2×10 controller header. With shunts fitted, the XIAO drives the kit. With shunts removed, any other controller can be wired to the odd pins.
- Baseband test header (I+, I−, Q+, Q−), test points on VTUNE, lock detect and PDBRF, LEDs for power, 3.3 V, PLL lock, bias tee and user.

### XIAO pins

| XIAO pin | Signal |
| --- | --- |
| D0, D1, D2 | I²S bit clock, frame sync, data from ADC |
| D3 | Synthesizer MUXOUT (lock detect) |
| D4, D5 | I²C SDA, SCL |
| D8, D9, D10 | Synthesizer CLK, LE, DATA |
| D6 | Optional ADC master clock (jumper), else spare |
| D7 | Spare |

### Expander bits (TCA9534)

| Bit | Signal | Default at power-up |
| --- | --- | --- |
| P0 | BAND_SEL: low = 2 m, high = 70 cm | 2 m |
| P1 | BIAS_EN: bias tee on | off |
| P2 | DEMOD_EN | off |
| P3, P4 | DEMOD_EDC, DEMOD_EIP2: enable the trim inputs | off |
| P5 | ADC_SHDNZ: high = ADC running | shut down |
| P6 | PLL_CE | on |
| P7 | User LED, active low | off |

## What was checked

| Check | Tool | Result |
| --- | --- | --- |
| Schematic netlist equals the design source | KiCad netlist export, compared by script | 146 nets, 228 parts, 0 mismatches |
| Every symbol pin has a footprint pad | script | 0 problems |
| Board pads equal the schematic netlist | KiCad netlist vs board, compared by script | 228 footprints, 0 mismatches |
| Design rule check | KiCad DRC | 0 unconnected, 0 clearance, 0 shorts, 0 courtyard overlaps |
| Remaining DRC items | KiCad DRC | 5 edge-clearance errors, all on the SMA edge connector's pads, which sit on the board edge by design |
| Loop filter | `rf_checks.py` (calculation) | 59–67° phase margin, 15–40 kHz bandwidth, both bands, VCO gain 40–80 MHz/V |
| Band filters | `rf_checks.py` (calculation, inductor Q 40) | 2 m: 1.4 dB loss; −12 dB at 108 MHz, −22 dB at 98 MHz, −32 dB at 88 MHz, −41 dB at 437 MHz. 70 cm: 1.6 dB loss; −46 dB at 146 MHz, −46 dB at 915 MHz |
| 50 Ω width | `rf_checks.py` (calculation) | 0.35 mm = 50.5 Ω on the stackup above |

DRC categories set to "ignore": silkscreen overlaps and text size, and library-footprint comparison. The silkscreen has not been tidied.

Pin tables were taken from manufacturer datasheets (LTC5584, TLV320ADC3140, ADF4351, PE4259, TC1-1-13M+, PGA-103+). The other parts use KiCad's library symbols.

## Review findings

Fix round 1 (25 November 2026) closed what could be closed at the desk. Status is one of: **fixed**, **resolved** (answered from a source, no change needed), **open**.

| # | Finding | Status | What was done or is still needed |
| --- | --- | --- | --- |
| 1 | No RF engineer has reviewed the layout | **open** | Needs a person. Everything except the RF path was routed by `gen/route.py`, which obeys clearances but knows nothing about noise coupling. Check VTUNE, the reference clock, the baseband pairs and the I²S clocks. |
| 2 | ADC AREG and DREG capacitors from memory | **open** | The datasheet text read here gives only VREF (1 µF minimum, as fitted). AREG and DREG keep 10 µF + 0.1 µF. Check the TLV320ADC3140 typical-application figure. |
| 3 | ADC master mode at 192 kHz unconfirmed | **resolved** | TI's controller-mode application note (SBAA382) lists 192 kHz and 384 kHz with the PLL on, and 24.576 MHz among the supported MCLK frequencies. MCLK must enter on GPIO1, which is how the board is wired. |
| 4 | LTC5584 RF input matching not populated | **fixed (footprints), open (values)** | Added a differential shunt-capacitor position across RF+/RF− beside the existing shunt-inductor and π pads, so any two-element L-C match can be fitted. The datasheet's values (450 MHz: 15 nH and 1.0 pF; 140 MHz: 68 nH and 8.0 pF) need their positions confirmed from its Figure 4, and one match cannot suit both bands. |
| 5 | LO port has no matching network | **open (measure)** | Kept as a direct drive: the ADF4351's outputs go straight to the LO pins through 10 nF. The LTC5584 wants 0 to +10 dBm; the ADF4351 gives up to +5 dBm into 50 Ω with a choke pull-up, and less with the 51 Ω pull-ups fitted. Set the ADF4351 to its highest output, check the level at the monitor port, and if it is low fit 27 nH chokes in place of the 51 Ω pull-ups (noted on those parts). The LO polarity swap stays; firmware must account for the sign of Q. |
| 6 | PLL loop filter not simulated | **fixed** | The old values gave only 33–45° of phase margin. New values (3.9 nF, 120 Ω + 180 nF, 560 Ω, 680 pF) give 59–67° and 15–40 kHz bandwidth across both bands and the data sheet's VCO-gain range. Still a calculation, not a measurement. |
| 7 | Weak FM broadcast rejection on 2 m | **fixed** | Both filters redesigned as Chebyshev ladders. 2 m rejection at 98 MHz improved from about −9 dB to −22 dB, and in-band loss dropped from 2.1 to 1.4 dB. The 70 cm filter now also rejects 915 MHz by about 46 dB (it was 22 dB), which helps Rev B's HaLow. Calculated; measure on the bench. |
| 8 | 5 V headroom for the LTC5584 | **fixed (option), open (measure)** | Added the I_DEMOD / SRC jumper so a bench supply can feed the demodulator past the fuse. Measure the demodulator's supply under load. |
| 9 | Land patterns | **fixed (LTC5584), resolved (SMA), open (TCXO)** | New LTC5584 footprint from LTC drawing 05-08-1697 Rev B: 2.45 mm exposed pad, 0.70 × 0.25 mm pads. The SMA footprint is drawn for a 1.57 mm board, matching the stackup. The TCXO footprint is a generic 2.5 × 2.0 mm 4-pad; check it against the TCXO chosen. |
| 10 | 50 Ω track width assumed | **resolved** | Calculated 50.5 Ω for 0.35 mm on the stackup now written above. Confirm with the board maker. |

### Known gaps

- **Fixed:** USB-C VBUS surge clamp; screw-terminal reverse-polarity protection.
- **Still open:**
  - No shield-can footprints. The Rev B plan treats shielding as a must.
  - No LNA bypass switch and no external-LO input. Use the U.FL break points instead.
  - The LO monitor port is a U.FL, not an SMA.
  - The expander's outputs are not on a header.
  - The top-left third of the board is empty. A Rev B could be about 100 × 60 mm.
  - Noise: with the ADC gain at 0 dB, the ADC's own noise will probably dominate. Expect to run 20–30 dB of ADC gain. A baseband amplifier may be needed in Rev B.

### Not done

- Firmware. None is written.
- Any measurement. Nothing here has been powered.
- Certification.

## Bring-up order

1. Fit no XIAO. Remove all six current jumpers. Apply 5 V and check for shorts. Check the reverse-polarity FET by briefly connecting the bench supply backwards at low current limit: nothing should draw current.
2. Fit the 3V3_DIG jumper, then the others one at a time, measuring current through each.
3. Fit the XIAO. Scan I²C: expect 0x20, 0x4C (after ADC_SHDNZ goes high) and 0x60.
4. Program the ADF4351 for 437 MHz. Check the lock LED and the LO monitor port with a spectrum analyzer. Repeat at 145.8 MHz.
5. Enable the demodulator. Inject a signal at break point 3 and look at the baseband header with a scope.
6. Start the ADC, capture I/Q, and check for the injected tone.
7. Refit the links, then measure sensitivity at 145.8 and 437 MHz against an RTL-SDR Blog V4 (or V3).
8. Look for spurs near 440 MHz (the XIAO's 40 MHz crystal, 11th harmonic).

Each step has a bench quest with scripts and a low-cost way to measure it: see the [bench kit](bench/README.md).

## Rev B: satellite stompbox

The plan for a pedal-enclosure version with knobs, a footswitch, an audio output and a Wi-Fi HaLow fallback link is in [rev-b-pedal-plan.md](rev-b-pedal-plan.md). It includes a desk RF review of putting the receiver and its radios in a metal box.

## Rebuilding

`gen/build.sh` regenerates the schematic, placement, routing, pours, checks and the fab package from `gen/design.py`. It needs KiCad 7 with its Python module (on Ubuntu 24.04: `apt-get install kicad kicad-symbols kicad-footprints`). Routing takes about half a minute per pass and repeats until every net connects. The Gerbers, drill, BOM and placement files in `fab/` were exported with `kicad-cli` from the board in `kicad/`.

The router is randomised by the order nets fail in, so a rebuild gives a different but equally checked layout. Commit a rebuild only after reviewing it.
