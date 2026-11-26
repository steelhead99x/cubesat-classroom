# [M] Sweep a filter with a noise source

## Goal

Turn the V4 and a broadband noise source into a filter tester (a scalar network analyzer). Practise on a filter you build from coax: a quarter-wave stub that notches out FM at 98 MHz. Compare the measured notch with the calculation. The same method later checks the CSC-R1 filters. Label results "hardware bench".

## Plan

**Time:** 60 minutes. **Bring:**
- the V4
- a broadband noise source module (often $15–30; check its supply voltage)
- 10 dB and 20 dB SMA attenuators
- an SMA tee
- about 1 m of RG-58 or RG-316 coax with an SMA on one end and the other end open
- side cutters and a ruler

A mentor checks the noise source's power before it's connected.

### Background

A noise source puts out roughly equal power at every frequency. Sweep it straight into the dongle, then through a filter. The difference at each frequency is the filter's response, and the dongle's own unevenness cancels out. A coax stub left open at the far end looks like a short circuit at the tee at the frequency where it is a quarter of a wavelength long. That makes a notch. Signals in coax travel slower than in air, by the velocity factor (about 0.66 for solid RG-58).

### Steps

1. **Work out the length:** `python3 sweep.py stub --freq 98e6 --vf 0.66`. Cut the coax about 3 cm longer than that, measured from the tee.
2. **Reference sweep.** Noise source → 10 dB → V4. Bias tee off. Try gain 0 and gain 10 first: if the floor rises much less than 10 dB, the dongle is overloading, so swap in the 20 dB attenuator.

   ```sh
   python3 sweep.py record --start 80e6 --stop 120e6 --step 100e3 --gain 10 --label "noise, through" --out through.csv --dc-checked
   ```

3. **Stub sweep.** Put the tee between the attenuator and the V4, with the stub on its third port. Record `--label "noise, stub" --out stub.csv` with the same settings.
4. **Find the notch:** `python3 sweep.py response --ref through.csv --dut stub.csv --at 98e6 --out stub-response.csv`.
5. **Trim.** The notch is low because the stub is long. Trim 1 cm at a time and re-sweep until the notch sits on 98 MHz. Write down the final length.
6. **Solve for the velocity factor:** `python3 sweep.py stub --freq <notch frequency> --length <final length in m>`.

### Check your work

- The reference and the stub sweep used the same gain, frequencies and attenuator.
- `response` prints how deep the deepest 5% of bins go. A notch that reads near that figure may be deeper than the method can show.
- Your velocity factor is within a few percent of the cable maker's figure. If not, re-measure the length from the centre of the tee.

### Levels

- **Beginner:** watch the notch move up as you trim the stub.
- **Intermediate:** the stub also notches 3 × 98 = 294 MHz. Sweep 250–330 MHz and check.
- **Professional:** put the stub on the antenna feed during the [FM quest](/quest/m0/receiver-fm-at-your-site.md) and measure how much FM drops and how much 2 m loses. Compare with the CSC-R1's calculated 2 m filter (−21.5 dB at 98 MHz, −1.4 dB in band).

### Talk it over

- Why does the dongle's own unevenness cancel out?
- Why is a notch at one frequency a poor filter for the whole FM band?

## Required

- [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) - the crew can record and compare fixed-gain sweeps.
