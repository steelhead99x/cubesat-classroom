---
title: Follow the Doppler shift
patch: On frequency
order: 8
summary: Tune the CSC-R1P receiver pedal, then keep a passing satellite's signal in the passband as its frequency drifts.
minutes: 15
skill: Tuning and Doppler shift
tool:
  name: Satellite pedal trainer
  href: /#pedal-lab
  note: The pedal on the home page. The pedal is designed but not built yet, so everything here is a desk simulation.
beginner:
  steps:
    - Do the three pedal tasks shown above the pedal. Tap ENGAGE, tune to 145.800 MHz, then set GAIN and LEVEL.
    - Write down the frequency, gain and level that worked.
    - Check the ENGAGE light is on, then press **Start a pass**. When the satellite rises, turn TUNE to keep the readout saying **carrier heard**. The score only counts time with the pedal engaged.
  check: You have the three settings written down and the percent of the pass you heard, from the result line.
  discuss: The satellite's signal seems to change frequency as it goes past. Have you heard a siren change pitch as it drives by?
intermediate:
  steps:
    - Before each pass, check the ENGAGE light. If it is off, tap ENGAGE. The score only counts seconds while the pedal is engaged, so a muted pass scores 0%. **Restart** turns ENGAGE off again.
    - Set the TUNE step to 5 kHz before starting the first pass. Record the percent heard.
    - Use the TUNE step button to select 1 kHz, then run another pass. Record the percent heard again.
    - During the second pass, write down the Doppler value at rise, at the highest point and at set.
  check: You have two scores, one for each step size, and three Doppler values with units.
  discuss: The receiver hears signals within 1.5 kHz of where it is tuned. Using that, explain why 5 kHz steps miss the start of the pass.
professional:
  steps:
    - Estimate the largest Doppler shift with Δf = f × v / c, using 145.8 MHz and the satellite's speed along your line of sight. Compare with the readout at rise.
    - Read `makePass` in `site/src/lib/pass.mjs`. Find where it decides a carrier is heard and where it marks a reading as decoded.
    - Run `cargo run --locked -- receiver-sim --track-doppler --seconds 660` and again without `--track-doppler`. Compare how many snapshots say `acquired`.
  check: Your estimate is within about 10% of the readout, and you can name the two conditions for decoded and the difference Doppler tracking makes.
  discuss: On 70 cm the shift is about three times bigger. What would you change on the pedal, or in software, to follow it?
---
