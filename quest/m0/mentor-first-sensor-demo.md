# [M] Mentor: guide a first sensor reading

## Goal

A mentor guides a small group to wire one sensor to an Arduino, read it, change something and explain the change. Every learner connects at least one wire and records a reading.

## Plan

Follow the [mentor rules](/quest/m0/mentor-get-ready.md#mentor-rules). The mentor checks every circuit before it is powered.

**Time:** 45 minutes. **Group:** up to 4 learners per board. **Bring:** an Arduino Uno or compatible board, a USB cable, a breadboard, jumper wires, a potentiometer, and a computer with the Arduino IDE. No login needed.

1. **Safety talk (mentor, 5 min).** Say: "We only power the board after I check the wires. USB power is low voltage, but a wrong wire can still damage a part."
2. **Wire it (everyone, 10 min).** Using the **AnalogReadSerial** example's wiring, each learner places one wire. The mentor checks the circuit against the diagram before plugging in.
3. **Upload (learner, 5 min).** A learner opens **File → Examples → Basics → AnalogReadSerial**, chooses the board and port and presses Upload while the mentor watches.
4. **Read (everyone, 10 min).** Open the Serial Monitor. Each learner writes down two numbers, labelled "raw counts". Each learner turns the knob and writes down what changed.
5. **Explain (everyone, 10 min).** Ask: "Why does the number change when you turn the knob? What would a unit like volts tell us that counts do not?" Let learners answer first.
6. **Pack up (mentor, 5 min).** Unplug USB before touching wires. Write the mentor log: board model, example version, how many learners wired and read, and one change for next time.

### Next step

With the mentor checking wiring first, the group can send live readings to the station using the [classroom sensor sketch](/firmware/classroom-sensors/classroom-sensors.ino). Label those results "hardware bench".

### Check your work

Every learner placed a wire and recorded a reading with its label, and the mentor checked the circuit before power.

## Required

- [Mentor: get ready to guide a demo](/quest/m0/mentor-get-ready.md) - the mentor has agreed to the rules.
- [Read one sample with Arduino](/quest/m0/arduino-first-reading.md) - the mentor has done the reading on their own first.
