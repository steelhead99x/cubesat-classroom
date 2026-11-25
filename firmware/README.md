# Firmware

Arduino sketches for the classroom hardware. Each one prints `CSC1` lines over USB, which the station's `sensor-bridge` reads. None of them turns on a radio.

| Sketch | Hardware | What it does | Guide |
| --- | --- | --- | --- |
| [classroom-sensors](classroom-sensors/classroom-sensors.ino) | Arduino Uno R3 or Nano, or a Seeed XIAO, with a TMP36, a BH1750 and a battery divider | Sends temperature, voltage and light once a second | [Ground station, level 1](../docs/ground-station.md#level-1-classroom-sensors) |
| [arduino-cubesat](arduino-cubesat/arduino-cubesat.ino) | Arduino Nano 33 BLE Sense Rev2 and one jumper wire | A complete bench CubeSat using the board's built-in sensors | [Bench CubeSats](../docs/bench-cubesats.md#arduino-cubesat) |
| [xiao-cubesat](xiao-cubesat/xiao-cubesat.ino) | Seeed XIAO ESP32-S3 (Sense), ESP32-C3, RP2040 or SAMD21, the XIAO Expansion Board and Grove sensors | A complete bench CubeSat with a screen, clock, data recorder and, on the S3 Sense, a camera | [Bench CubeSats](../docs/bench-cubesats.md#seeed-xiao-cubesat) |

CI compiles every sketch for each board it names (see the `firmware` job in [.github/workflows/ci.yml](../.github/workflows/ci.yml)). A successful compile is not a hardware test: label readings "hardware bench" and check them against a reference.
