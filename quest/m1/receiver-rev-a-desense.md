# [L] Measure HaLow desense on the CSC-R1

## Goal

Measure how much a nearby HaLow transmitter reduces the Rev A receiver's ability to hear a weak signal on 2 m and 70 cm, at distances like those in the Rev B pedal. The result decides the Rev B layout and antenna plan. Label results "hardware bench".

## Plan

**Time:** two sessions of 60 minutes. **Bring:**
- a Rev A kit streaming I/Q
- the shielded [test signal](/quest/m0/receiver-test-signal.md)
- the HaLow pair from the [HaLow noise quest](/quest/m0/receiver-halow-noise.md)
- the V4 for a side-by-side check

A licensed operator approves the test-signal frequencies.

### Steps

1. **Weak tone.** Test signal at 145.9 MHz, attenuated until the Rev A shows the tone about 10 dB above its noise floor. Write down the total attenuation.
2. **HaLow off.** Record a minute of I/Q and the tone's height above the floor.
3. **HaLow streaming.** With the HaLow antenna at 0.1, 0.2, 0.5 and 1 m from the Rev A's antenna port (or the U.FL break point 1 on a short cable, to stand in for a pedal-mounted antenna), record again. The drop in the tone's height is the desense.
4. **Try the fixes.** At 0.2 m, repeat with the HaLow board inside a grounded tin, then with the receive antenna 2 m away on coax. These are the two Rev B options.
5. Repeat on 70 cm at 436.0 MHz.
6. **Side by side.** Repeat steps 2–3 with the V4 on the same tone. The Rev A's 70 cm filter is calculated to cut 915 MHz by about 46 dB, and the V4 has no such filter. Does that show?
7. Share a table of band, distance, fix and desense in dB.

### Check your work

- The test signal stayed in its closed tin and its level didn't change between runs. Check with HaLow off at the start and the end.
- Firmware marks or drops I/Q blocks taken while HaLow is sending, if the HaLow node reports it. Say which you used.
- "Hardware bench" on every result, with the HaLow bandwidth, its transmit power setting and its channel.

## Required

- [Check the CSC-R1 local oscillator and 5 V under load](/quest/m1/receiver-rev-a-lo-and-supply.md) - the receiver runs from a known-good LO and supply.
- [Does HaLow noise land in the receive bands?](/quest/m0/receiver-halow-noise.md) - provides the HaLow pair, the firmware and a V4 baseline.
- [Build a shielded test signal](/quest/m0/receiver-test-signal.md) - provides the weak, repeatable tone.
