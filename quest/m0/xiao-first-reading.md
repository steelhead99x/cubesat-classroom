# [S] Record a bench sample with Seeed XIAO

## Goal

Read one sensor with a Seeed XIAO board and label where the value came from.

## Plan

1. Read the model printed on the board. A XIAO ESP32-C3 or XIAO RP2040 is a place to start if your kit already includes one. Identify the sensor module and use the documentation for your exact board and sensor.
2. Use the board maker’s USB setup and the sensor maker’s readout example. Have a mentor verify pins and sensor voltage before wiring. Turn off power while changing connections.
3. Record a reading with units and repeat it after changing one thing. Without a matching sensor, run the board’s USB serial example and label the output as a connection check.
4. Share the exact model, setup, and what another crew should expect to see.
5. Next step: with a mentor checking the wiring first, connect the TMP36, battery divider and BH1750 (the station needs all three) and send live readings to the station with the [classroom sensor sketch](/firmware/classroom-sensors/classroom-sensors.ino) and `sensor-bridge`. See [Level 1 of the ground-station guide](/docs/ground-station.md#level-1-classroom-sensors). Label the result "hardware bench". With the XIAO Expansion Board and Grove sensors, you can also [fly the Seeed XIAO bench CubeSat](/quest/m0/xiao-bench-cubesat.md).
