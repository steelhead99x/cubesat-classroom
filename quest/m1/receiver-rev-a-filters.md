# [M] Measure the CSC-R1 band filters

## Goal

Measure the 2 m and 70 cm band filters on a real Rev A board with the noise-source method, and compare them with the desk calculation from `gen/rf_checks.py`. This turns finding 7 from calculated into measured. Label results "hardware bench".

## Plan

**Time:** 60 minutes. **Bring:**
- the noise-source setup from the [stub quest](/quest/m0/receiver-sweep-a-stub.md)
- a powered Rev A kit with bring-up firmware
- two U.FL-to-SMA cables
- a U.FL-to-U.FL cable for the reference sweep

### Background

Rev A has three RF break points, each a 0 Ω link with a U.FL on each side. Remove the links at break points 2 (LNA to filters) and 3 (filters to demodulator). The filter path is then reachable from two U.FLs, with the band switches still in circuit, as the receiver uses it.

### Steps

1. **Bias tee off on both.** Run `BIAS OFF` on the kit. The sweep script turns off the V4's bias tee. Inductors to ground in a filter short any DC, and DC can damage the noise source.
2. **Reference.** Noise source → attenuator → U.FL-to-U.FL through cable → V4. Sweep with `--band filter2m --gain 10 --out ref-2m.csv`.
3. **2 m filter.** Run `BAND 2M`. Noise in at break point 2's filter-side U.FL, V4 at break point 3's filter-side U.FL. Sweep `--band filter2m` with the same gain.
4. **Response:** `python3 sweep.py response --ref ref-2m.csv --dut f2.csv --at 88e6 98e6 108e6 145e6 437e6 --out f2-response.csv`.
5. **70 cm filter.** Run `BAND 70CM` and repeat with `--band filter70cm`, reporting `--at 146e6 436e6 915e6`.
6. Put your numbers next to the calculated ones:

   | Filter | Frequency | Calculated (desk simulation) | Measured |
   | --- | --- | --- | --- |
   | 2 m | 88 / 98 / 108 MHz | −31.5 / −21.5 / −12.1 dB | |
   | 2 m | in band | −1.4 dB | |
   | 2 m | 437 MHz | −41.3 dB | |
   | 70 cm | 146 MHz | −46.2 dB | |
   | 70 cm | in band | −1.6 dB | |
   | 70 cm | 915 MHz | −46.0 dB | |

7. Refit both 0 Ω links. Share the table and the response CSVs.

### Check your work

- The measured numbers include the band switches and the U.FL cables. Write down what the reference sweep did and didn't include.
- A measured rejection close to the method's limit (printed by `response`) means "at least that much", not that exact value.
- The in-band loss is the number most worth getting right. Repeat it three times.

### Levels

- **Intermediate:** find each filter's 3 dB edges and compare them with the band edges (144–146 and 435–438 MHz).
- **Professional:** add a return-loss bridge (about $15–30; estimate) and the same noise source to measure the demodulator's input match at break point 3, for finding 4. A NanoVNA or LiteVNA does this directly.

## Required

- [Sweep a filter with a noise source](/quest/m0/receiver-sweep-a-stub.md) - the crew has practised the method.
- [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md) - provides the BAND and BIAS commands.
