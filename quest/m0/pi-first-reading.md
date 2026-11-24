# [S] Log one bench reading with Raspberry Pi

## Goal

Run a repeatable reading on a Raspberry Pi and keep enough context for another classroom to reproduce it.

## Plan

1. Record the model and operating system. A Raspberry Pi running Raspberry Pi OS can start this quest. If the board says **Pico**, choose the microcontroller path and use a Pico example.
2. Use a teacher-approved sensor and its documented Raspberry Pi example. Check voltage and pin assignments with a mentor before connecting it. With a Pi and no external sensor, read the Pi’s own CPU temperature as the source.
3. Take two timestamped readings. Include units and label the source as an external sensor or the Pi’s own CPU. Explain which readings compare fairly.
4. Share a short CSV with the crew nickname, model, software version, and steps.
5. Next step: record the CPU temperature once a second. In one terminal run `cargo run --locked -- sensor-bridge --out run/sensors --map temp_mc=/sys/class/thermal/thermal_zone0/temp`. In a second terminal run `cargo run --locked -- balloon-log --sensor-dir run/sensors --output pi-run.csv --samples 60`. See [the Raspberry Pi section of the ground-station guide](/docs/ground-station.md#raspberry-pi-without-an-arduino). Label it as the Pi's own CPU, not the room.
