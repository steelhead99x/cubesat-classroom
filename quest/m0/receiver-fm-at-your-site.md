# [M] Measure FM broadcast at your antenna

## Goal

Measure how strong FM broadcast stations (88–108 MHz) are at your station antenna compared with the 2 m band. This tells the reviewer whether the CSC-R1 2 m filter is enough at your site (finding 7 in the [receiver README](/hardware/sdr-receiver/README.md#review-findings)). Label the results "hardware bench".

## Plan

**Time:** 30 minutes. **Bring:** the V4 setup from [first light](/quest/m0/receiver-v4-first-light.md) and the antenna in its usual place.

### Background

FM transmitters are thousands of times more powerful than a satellite, and often much closer. In the CSC-R1 the low-noise amplifier comes first, so FM reaches it at full strength. The 2 m filter that follows is calculated to cut FM at 98 MHz by about 21 dB (a desk simulation, not a measurement), which protects the demodulator and the ADC.

### Steps

1. Sweep both bands at the same gain, without moving the antenna:

   ```sh
   python3 sweep.py record --band fm --gain 10 --label "antenna, FM" --out fm.csv
   python3 sweep.py record --band 2m --gain 10 --label "antenna, 2 m" --out 2m.csv
   python3 sweep.py summary fm.csv 2m.csv
   ```

   Use a low gain, because strong FM can overload the dongle. If the FM summary changes a lot between gains 0 and 10, the dongle was overloading; use the lower gain.
2. Write down the three strongest FM stations and their levels (re-run `summary` or open the CSV).
3. Work out how far the strongest station sits above the 2 m noise floor, in dB. Then take off 21 dB for the filter. What would still be left, compared with that noise floor?
4. Share the numbers, the antenna's location and which way it points. Say where the nearest FM transmitter is, if you know.

### Check your work

- Both sweeps used the same gain.
- The V4 has its own filtering for strong broadcast signals; check its datasheet. If it cuts FM, the FM levels you measured are lower than what reaches the CSC-R1, so treat them as a minimum.
- The dongle's response isn't flat, so a difference between 98 MHz and 145 MHz is only approximate. The professional level corrects for this.

### Levels

- **Beginner:** find the strongest FM station and how many dB louder it is than the quiet 2 m band.
- **Intermediate:** turn the antenna or move it 2 m and repeat. How much does the FM level change?
- **Professional:** correct for the dongle's response. Use the [test signal](/quest/m0/receiver-test-signal.md) at the same level at 98 MHz and at 145.9 MHz, record the difference, and apply it to your result.

## Required

- [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) - provides the fixed-gain sweeps and the 2 m noise floor.
