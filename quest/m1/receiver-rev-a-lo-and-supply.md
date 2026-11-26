# [M] Check the CSC-R1 local oscillator and 5 V under load

## Goal

Close two open review findings by measurement: the LO's frequency and relative level (finding 5), and the demodulator's supply voltage with everything running (finding 8). See the [receiver README](/hardware/sdr-receiver/README.md#review-findings). Label results "hardware bench".

## Plan

**Time:** 45 minutes. **Bring:**
- a powered-up Rev A kit running the bring-up firmware
- the V4
- 40 dB of SMA attenuation (30 dB + 10 dB) and a U.FL-to-SMA cable
- a multimeter

Optional:
- an AD8318 log power-detector board (about $20; estimate). It reads absolute level in dBm once checked against a known source.

### Steps

1. **Frequency.** Connect the LO monitor U.FL → 40 dB → V4. Run `LO 145.8`, then `python3 sweep.py record --start 145.6e6 --stop 146.0e6 --step 1e3 --gain 0 --label "LO 145.8, monitor port, -40 dB" --out lo-2m.csv`. Write down the peak frequency and its error in ppm. Repeat for `LO 437` around 437 MHz.
2. **Relative level.** Step the ADF4351 output power 0 to 3 with `LO 437 <n>` and sweep each time. Each step should add about 3 dB. If it doesn't, the dongle may be overloading: add attenuation.
3. **Absolute level (optional).** With the AD8318 on the monitor port, read the level at power 3. The LTC5584 wants 0 to +10 dBm at its LO pins. The monitor port is a second output, so its level is only a guide to what the demodulator sees. If it reads well below 0 dBm, fit the 27 nH chokes in place of the 51 Ω pull-ups (noted on those parts) and measure again.
4. **5 V under load.** Run `BIAS ON` (with a powered LNA or a DC-safe load on the antenna port, never a short), `DEMOD ON` and `ADC START 20`. Measure the voltage on the demodulator side of the `I_DEMOD` jumper, first on USB-C and then on a bench supply through `I_DEMOD` 2-3. The LTC5584 needs at least 4.75 V.
5. Share the frequencies, ppm, relative steps, any absolute reading, and both 5 V readings with the supply used for each.

### Check your work

- The ppm error you measure includes the V4's own error. Compare with the V4's error from the [test signal quest](/quest/m0/receiver-test-signal.md).
- The 5 V readings were taken with everything on, not at idle.
- Results go into the README's findings table only through a pull request, labelled "hardware bench", reviewed by someone else.

## Required

- [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md) - provides the LO, DEMOD, BIAS and ADC commands.
