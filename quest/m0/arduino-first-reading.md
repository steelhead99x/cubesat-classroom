# [S] Read one sample with Arduino

## Goal

Record one repeatable bench reading with an Arduino-compatible board and explain what it represents.

## Plan

1. Record the exact board model, cable and sensor you have. For a common starting point, choose an Arduino Uno. Follow the board documentation to select the correct board and port in the Arduino IDE.
2. Open **AnalogReadSerial** and follow its wiring example if your kit has the matching parts. If your kit has a different sensor, use the manufacturer’s example for that model. Ask a mentor to check voltage and wiring before powering the circuit.
3. Record two samples with their units or label them as raw counts. Change one thing, such as moving a potentiometer, and explain the change.
4. Share the board, example version, procedure, and observed result using the crew nickname. If you lack the parts, try the [simulator quest](/quest/m0/repeat-the-station-lesson.md).
5. Next step: with a mentor checking the wiring first, connect the TMP36, battery divider and BH1750 (the station needs all three) and send live readings to the station with the [classroom sensor sketch](/firmware/classroom-sensors/classroom-sensors.ino) and `sensor-bridge`. See [Level 1 of the ground-station guide](/docs/ground-station.md#level-1-classroom-sensors). Label the result "hardware bench". With a Nano 33 BLE Sense Rev2, you can also [fly the Arduino bench CubeSat](/quest/m0/arduino-bench-cubesat.md), which needs no extra sensors.
