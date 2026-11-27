# [M] Fly the Seeed XIAO bench CubeSat

## Goal

Run a complete bench mission on a Seeed XIAO and the XIAO Expansion Board with Grove sensors: deploy it, set its clock, make first contact, record a pass on the microSD card and, with an ESP32-S3 Sense, bring back a photo. Label every result "hardware bench".

## Plan

**Time:** 60 minutes. **Group:** up to 4 learners per board. **Bring:** a Seeed XIAO (ESP32-S3 Sense for the photo step), the XIAO Expansion Board, a Grove I2C Hub, the Grove BME280, Digital Light Sensor and 6-Axis Accelerometer & Gyroscope, a microSD card, a CR1220 coin cell, a USB-C data cable, and a computer with the Arduino IDE and Rust (the station's `cargo` toolchain). No login is needed.

Read the [bench CubeSat guide](/docs/bench-cubesats.md#seeed-xiao-cubesat) first, including the two fit checks. A mentor checks every connection before USB is connected. Leave any lithium battery out.

1. **Integrate.** Seat the XIAO, plug in the hub and the three sensors, and insert the card and coin cell. Install the board package and libraries the guide lists, then upload [xiao-cubesat.ino](/firmware/xiao-cubesat/xiao-cubesat.ino).
2. **Check the boot report.** Open the Serial Monitor (115200 baud, Newline). Write down which parts were found and which were missing. Read the OLED.
3. **Deploy.** Press the Expansion Board button. Watch the countdown on the OLED and listen for the buzzer.
4. **Set the clock.** Send `TIME` with the current date and time, for example `TIME 2026-11-25T14:30:00`. Check that the next `CSC1` line has an `rtc=` field.
5. **First contact.** Send `3 PING` twice, then `STATUS` and `SENSE`. Record one `CSC1` line and say what each unit means.
6. **Record a pass.** Send `ORBIT 30 30` and let it run for three orbits. Shade the light sensor during one gap. Send `ORBIT OFF`.
7. **Read the recorder.** Unplug USB, move the card to a computer and open `CUBESAT.CSV`. Find the rows where `in_contact` is 0 and compare them with the `REC` lines you received.
8. **Bring back a picture (S3 Sense only).** Send `PHOTO`, clear the Serial Monitor, send `DOWNLINK`, then save the output to `capture.txt`. Run `cargo run --locked --release -p bench-tools --bin csc-serial-image -- capture.txt photo.jpg` from the repository folder. Then delete a few `IMG` lines from the file and run it again.

### Check your work

- Your log has the board model, sketch version, date and the parts found at boot.
- It has at least one `CSC1` line, one `REC` line and a few rows from `CUBESAT.CSV`, each labelled "hardware bench".
- You compared `temp_mc`, `pressure_pa` and `light_lux` with a reference and wrote down which reference you used.
- With an S3 Sense: the script reported the full capture as intact, and reported the gaps and an incomplete transfer after you deleted lines.

### Talk it over

- The card has every reading, but the ground only got the stored ones after each pass. What would a real mission do with the card copy?
- `chip_temp_mc` and `temp_mc` differ. Which one describes the room, and why does the sketch keep them apart?
- How long would your photo take to send at 1200 bits per second? How many passes would that need?
- What did the checksum tell you that looking at the photo did not?

### Next step

Have a mentor wire the A0 voltage divider (see the [guide](/docs/bench-cubesats.md#voltage-optional-mentor-wired)), then feed the `CSC1` lines to the station with `sensor-bridge`.

## Required

- [Record a bench sample with Seeed XIAO](/quest/m0/xiao-first-reading.md) - the crew can upload a sketch to their XIAO and read the Serial Monitor.
