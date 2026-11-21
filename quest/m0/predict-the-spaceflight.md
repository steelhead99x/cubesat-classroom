# [S] Predict the spaceflight

## Goal

Before the flight, write down what each sensor in the kit should record in each phase of a suborbital flight. Label the result a desk simulation.

## Plan

1. Make the timeline. If you can't run the command, use the ready-made [flight-prediction.csv](/docs/assets/flight-prediction.csv) and skip to step 2.

   ```sh
   cargo run --locked -- flight-profile --output run/flight-prediction.csv
   ```

2. Open the CSV in a spreadsheet. Plot `altitude_m` against `elapsed_s`. Mark release, engine cut-off, the highest point and landing.
3. For each phase, predict temperature, pressure, acceleration, rotation, light, humidity and CO₂: up, down or flat, and why. Read [what the pressure sensor will and won't show](/docs/suborbital-flight-plan.md#what-the-pressure-sensor-will-and-wont-show) after you have guessed.
4. If you can run the command, change one input, such as `--apogee-miles 50`, and record what happens to the microgravity time.
5. Check your work: the default run should give about 193 seconds of microgravity and a highest point of 87.5 km.
6. Share the crew nickname, your chart and your predictions. Keep them: after the flight they are compared with the real data.

## Levels

- **Beginner:** colour the chart by phase and say when things would float.
- **Intermediate:** work out how many readings a once-a-second logger gets in the boost and in microgravity. Is that enough for each?
- **Professional:** read the model in `src/flight.rs`. List its assumptions (vertical climb, constant thrust, no drag) and estimate which one adds the most error.
