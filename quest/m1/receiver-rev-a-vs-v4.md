# [L] Compare the CSC-R1 with the RTL-SDR V4 on a real pass

## Goal

Compare the Rev A receiver with an RTL-SDR Blog V4 on the same antenna: first on the bench with the weak test signal, then on a real satellite pass. Label the bench result "hardware bench" and the pass "received RF".

## Plan

**Time:** 45 minutes on the bench, plus one pass. **Bring:**
- a Rev A kit
- the V4
- a two-way 50 Ω power splitter
- the shielded [test signal](/quest/m0/receiver-test-signal.md)
- the station antenna
- a pass prediction from [Predict a satellite pass](/quest/m0/predict-a-pass.md)

Both receivers only listen. Keep the bias tee off on both unless the antenna has a powered preamplifier, and then power it from one receiver only.

### Steps

1. **Bench.** Test signal → splitter → Rev A and V4. Lower the test signal until each receiver just loses the tone. The difference in the attenuation at that point is the sensitivity difference. Do it at 145.9 and 436.0 MHz.
2. **Settings.** Write down the Rev A's ADC gain and the V4's gain. Try Rev A ADC gains of 0, 20 and 30 dB: the README expects 20–30 dB to be needed.
3. **Pass.** Antenna → splitter → both receivers. Record I/Q from both for a whole pass of an FM satellite or the ISS. Write down the satellite, time (UTC) and highest elevation.
4. **Compare.** For the same moments in the pass, compare how far the signal sits above each receiver's floor. Note when each first and last hears the satellite.
5. Share both results with the labels, settings and splitter model.

### Check your work

- The splitter costs both receivers the same 3 dB or so, so the comparison is fair. Write down its model.
- The bench result and the pass result are labelled separately.
- If the Rev A does worse, write down which block you suspect and why, with evidence from earlier quests.

### Talk it over

- Why is a splitter fairer than swapping the antenna between receivers during a pass?
- A V4 dongle costs about $30–40. What would the CSC-R1 have to do better to be worth building?

## Required

- [Check the CSC-R1 local oscillator and 5 V under load](/quest/m1/receiver-rev-a-lo-and-supply.md) - the receiver runs from a known-good LO and supply.
- [Measure the CSC-R1 band filters](/quest/m1/receiver-rev-a-filters.md) - the filters are known before blaming them.
- [Build a shielded test signal](/quest/m0/receiver-test-signal.md) - provides the weak, repeatable tone.
