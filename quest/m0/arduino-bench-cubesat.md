# [M] Fly the Arduino bench CubeSat

## Goal

Run a complete bench mission on one Arduino Nano 33 BLE Sense Rev2: deploy it, make first contact, lose and regain the signal, enter safe mode, and calibrate its light sensor. Label every result "hardware bench".

## Plan

**Time:** 50 minutes. **Group:** up to 4 learners per board. **Bring:** an Arduino Nano 33 BLE Sense Rev2, a micro-USB data cable, one jumper wire, a computer with the Arduino IDE, and a light meter or the classroom BH1750 kit for step 7. No login is needed.

Read the [bench CubeSat guide](/docs/bench-cubesats.md#arduino-cubesat) first. A mentor checks the jumper before USB is connected.

1. **Integrate.** Put the jumper from **D2** to **GND**. This is the remove-before-flight pin. Install the board package and the four libraries the guide lists, then upload [arduino-cubesat.ino](/firmware/arduino-cubesat/arduino-cubesat.ino).
2. **Check the boot report.** Open the Serial Monitor (115200 baud, Newline). Write down which sensors were found. Type `PING`. What happens, and why?
3. **Deploy.** Pull the jumper. Watch the blue light and the countdown. Write down the time of the first `BCN` line.
4. **First contact.** Send `7 PING`, then `7 PING` again. Send `STATUS` and `SENSE`. Record one `CSC1` line and say what each unit means.
5. **Lose the signal.** Send `LOS`, wait about 30 seconds, then send `PING`. Send `AOS`. Count the `REC` lines and check the `seq` numbers for gaps. Then try `ORBIT 20 20` for two orbits.
6. **Safe mode.** Send `SAFE`, then `RATE 2`. Read the reply. Send `NOMINAL`.
7. **Calibrate light.** Hold the light meter beside the sensor and send `SENSE`, then `CAL` with the meter's reading. Move both to a darker spot and compare `light_lux` with the meter again.
8. **Check against references.** Compare `temp_mc` with a thermometer and `pressure_pa` with your local station pressure. Lay the board flat and check `az_mg`.

### Check your work

- Your log has the board model, sketch version, date, and at least one `CSC1` and one `REC` line.
- Each reading has a unit and the label "hardware bench".
- You wrote down how far the temperature and light readings were from the references, and which reference you used.

### Talk it over

- Why does the CubeSat ignore commands before it deploys, and while it is out of contact?
- What would happen to readings stored during a long gap if the 64-slot store filled up? How would the ground crew find out?
- Why does a repeated `7 PING` get refused? When would that matter on a real command link?
- The 3.3 V reading barely changes. What does that tell you about the power system, and what would a real CubeSat measure instead?

### Next step

Close the Serial Monitor and feed the board's `CSC1` lines to the station with `sensor-bridge`, as the [guide](/docs/bench-cubesats.md#connecting-to-the-classroom-station) explains. Then compare what the station shows during `ORBIT` with what the Serial Monitor showed.

## Required

- [Read one sample with Arduino](/quest/m0/arduino-first-reading.md) - the crew can upload a sketch and read the Serial Monitor.
