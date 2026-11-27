# [S] Build a shielded test signal

## Goal

Make a known, very weak test signal on the bench from an ADF4351 module (the same synthesizer chip as the CSC-R1's local oscillator), fixed attenuators and a metal tin. Check its frequency and harmonics with the V4. Later quests use it to test sensitivity and desense without a lab signal generator. Label results "hardware bench".

## Plan

**Time:** 45 minutes. **Bring:**
- an ADF4351 module that comes with its own controller and buttons, so no programming is needed (roughly $30–60; check before buying)
- SMA attenuators: 10, 20 and 30 dB, plus two more of 30 dB
- SMA cables
- a metal tin with a lid, such as a cookie tin
- the V4

A mentor checks the setup before anything is powered.

### Safety and licensing

- This is a signal source. It goes into attenuators and a cable only, never into an antenna.
- 145 and 437 MHz are amateur bands. A licensed operator approves the frequencies and the setup, and the crew follows their lead.
- Never connect any transmitter straight to the dongle. That includes this module without attenuators, a HaLow board and a radio. The dongle's input is easily damaged.

### Steps

1. **Build the chain.** Module → 30 dB → 30 dB → 10 dB → V4. With the module at its lowest output that's about 70 dB down, so the dongle sees a weak but clear signal. Put the module and its battery or USB power bank in the tin with the lid closed. Only the coax comes out.
2. **Set 145.900 MHz.** Run `csc-sweep record --band 2m --gain 20 --label "ADF4351 145.9, -70 dB" --out sig-2m.csv --dc-checked` and `summary`. Write down the frequency of the peak and how far it is above the floor.
3. **Look for harmonics.** The ADF4351's output is close to a square wave, so it has strong odd harmonics. Sweep 70 cm with the same source: 3 × 145.9 = 437.7 MHz. Write down how much weaker the harmonic is.
4. **Check the shielding.** Open the lid and sweep again. If the level rises, the module was leaking past the attenuators. That leak limits how weak a test signal you can make.
5. **Make it weaker.** Add the 20 dB attenuator, then another 30 dB, sweeping each time. Find where the signal disappears into the floor. Write down that total attenuation.

### Check your work

- The frequency error in step 2 is the dongle's error plus the module's. Write it in kHz and in ppm (kHz ÷ MHz × 1000).
- Each step's sweep says how much attenuation was fitted.
- The level drops by about the attenuation added until it nears the floor. If it doesn't, the signal is leaking around the attenuators.

### Levels

- **Beginner:** watch the peak shrink each time you add an attenuator.
- **Intermediate:** make a table of attenuation against peak above the floor, and plot it. Where does the line stop being straight?
- **Professional:** the CSC-R1 drives its demodulator from this same chip. With the LO at 145.8 MHz, its third harmonic mixes 437.4 MHz down too. Use the 2 m filter's calculated −41 dB at 437 MHz and your harmonic measurement to explain why the filter has to be in front of the demodulator.

## Required

- [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md) - the crew can record and compare fixed-gain sweeps.
