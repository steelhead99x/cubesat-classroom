# [M] Does HaLow noise land in the receive bands?

## Goal

Before the Rev B pedal puts a Wi-Fi HaLow radio (902–928 MHz) in the same box as the receiver, measure how much a streaming HaLow node raises the noise on 2 m and 70 cm at different distances. Use the V4 as the receiver. This is the first look at Rev B risk 1 in the [Rev B plan](/hardware/sdr-receiver/rev-b-pedal-plan.md#risks). Label results "hardware bench".

## Plan

**Time:** 60 minutes. **Bring:**
- the V4 and its antenna
- two Seeed Wi-Fi HaLow boards for XIAO, each on a XIAO ESP32-S3, with their own antennas (one access point, one station)
- a tape measure

HaLow on 902–928 MHz is licence-free. Use only the antenna that came with the module, because its certification depends on it. Never connect a HaLow antenna socket to the dongle.

### Steps

1. **Baseline.** HaLow boards unplugged. Record `--band 2m` and `--band 70cm` at gain 30, `--label "HaLow off"`.
2. **Link up.** Power both boards and start a continuous stream from one to the other: the project's HaLow firmware from `steelhead99x/cubesat-classroom-rust`, or any example that sends without stopping. Write down the firmware and its version.
3. **Close.** Put the transmitting board 0.1 m from the V4's antenna. Record both bands, `--label "HaLow streaming, 0.1 m"`.
4. **Move it.** Repeat at 0.3 m, 1 m and 3 m. Then do 0.1 m once more with the link idle (associated but not sending).
5. **Compare** each run with the baseline: `csc-sweep compare off-2m.csv on-0.1m-2m.csv`, and the same for every distance and band.
6. **See HaLow itself.** With the transmitter at 3 m, record `--band halow --gain 0`. Write down its channel and bandwidth from the shape of the peak.
7. Share a table of distance, band, floor change and largest single-bin rise.

### Check your work

- Every sweep in a set used the same gain, antenna and dongle position.
- Repeat the baseline at the end. If it moved by more than 0.5 dB, the room changed, so treat smaller rises as noise.
- At 0.1 m the HaLow signal can overload the V4 itself. Part of the rise may be the dongle, not HaLow's emissions. Say so in your result. The CSC-R1's 70 cm filter is calculated to cut 915 MHz by about 46 dB, which the V4 doesn't have.

### Levels

- **Beginner:** find the distance where the noise stops changing.
- **Intermediate:** plot floor change against distance for both bands. Which band is hurt more, and why might that be?
- **Professional:** use the [test signal](/quest/m0/receiver-test-signal.md) at a level just above the floor on 145.9 MHz. Measure how far its peak-above-floor drops with HaLow streaming at each distance. That drop is the desense.

### Talk it over

- In the pedal the antennas are about 10–20 cm apart (an estimate). What does your result say about putting the receive antenna up the mast?
- Why does the plan mark I/Q samples taken while HaLow is sending?

## Required

- [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) - provides fixed-gain sweeps and the site's noise floor.
