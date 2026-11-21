# CubeSat Classroom quest log

## Goal

Pass working lessons and well-documented evidence from students, teachers and radio volunteers to the next crew. Use the [quick-start guide](../docs/quick-start.md) for the workflow. The Markdown format comes from [moq-dev/quest](https://github.com/moq-dev/quest).

## Plan

### Start with what you have

Write down the equipment you have, including the board or printer model, available sensors, and computer operating system. If you have several boards, pick one for the first run. No hardware is needed to get started.

| Your setup | First quest |
| --- | --- |
| Bambu Lab P1S | [Validate the Rev A case slice](/quest/m0/slice-p1s-case.md) |
| Computer only or unsure | [Teach the two-relay lesson](/quest/m0/repeat-the-station-lesson.md) |
| Arduino, such as an Uno | [Read one Arduino sample](/quest/m0/arduino-first-reading.md) |
| Seeed XIAO, such as ESP32-C3 or RP2040 | [Record a XIAO bench sample](/quest/m0/xiao-first-reading.md) |
| Raspberry Pi with Raspberry Pi OS | [Log a Pi reading](/quest/m0/pi-first-reading.md) |
| Any computer, no extra hardware | [Predict a satellite pass](/quest/m0/predict-a-pass.md) |
| Raspberry Pi Pico or another microcontroller | Use a vendor sample for your exact board and follow the [microcontroller quest structure](/quest/m0/arduino-first-reading.md). |

### Teacher pilot

Start in M0. After a run, tell us which steps the class completed, which instruction they got stuck on, and one change you would make. Share a redacted note through the [contribution guide](../CONTRIBUTING.md). Teachers can pilot and review lessons before any flight work.

### Bambu P1S case test track

Use the enclosed Rev A case for these bench quests. Start with PLA and move through the steps in order. The Rev B rail-fit and ISS work has its own review path below.

1. [Validate the slice in Bambu Studio](/quest/m0/slice-p1s-case.md)
2. [Print and measure the PLA fastener coupon](/quest/m0/print-p1s-case-coupon.md)
3. [Print the shell and lid](/quest/m0/print-p1s-case-parts.md)
4. [Test the assembly fit](/quest/m0/test-p1s-case-fit.md)
5. [Review an improvement against the fit evidence](/quest/m0/improve-p1s-case.md)
6. [Repeat the fit test](/quest/m0/retest-p1s-case.md)

### Demo and bench experiments

- [Rehearse the five-minute demo offline](/quest/m0/demo-offline-rehearsal.md) and record a restart procedure.
- [Trace a sensor value from file to display](/quest/m0/trace-sensor-fixtures.md) and verify what happens when an input is missing.
- [Measure what the camera lid blocks](/quest/m0/measure-case-camera-view.md) once the P1S case fit has been tested.

### Flight work

The ISS hardware work is listed in review priority. Start the provider inquiry and the P1S slice review in parallel. The numbered lists show the recommended order for review. Quest dependencies control what is ready to start. Run `quest unblocked` to see ready work and `quest check` to validate the board. See the [contribution guide](../CONTRIBUTING.md).

### M0 · Start now

1. [Identify an ISS path for printed rails](/quest/m0/iss-provider-route.md)
2. [Slice Rev B on the Bambu P1S profile](/quest/m0/slice-rev-b.md)
3. [Measure the PLA rail coupon](/quest/m0/print-and-fit.md)
4. [Measure the ASA rail coupon](/quest/m0/asa-rail-coupon.md)
5. [Measure the full printed rail frame](/quest/m0/full-frame-fit.md)
6. [Measure the full ASA rail frame](/quest/m0/full-asa-frame.md)
7. [Verify both printed end plates seat](/quest/m0/end-plate-fit.md)

### M1 · Close the design and test plan

1. [Create the ISS compliance matrix](/quest/m1/iss-compliance-matrix.md)
2. [Agree the printed material and process](/quest/m1/printed-structure-qualification.md)
3. [Design the printed chassis and rail interface](/quest/m1/flight-rail-cad.md)
4. [Design positive printed plate retainers](/quest/m1/printed-retainers.md)
5. [Design printable payload and harness mounts](/quest/m1/printed-payload-mounts.md)
6. [Close the mass and center-of-mass budget](/quest/m1/mass-and-balance.md)
7. [Verify the deployment inhibit design](/quest/m1/deployment-inhibits.md)
8. [Prepare the crewed-flight battery safety evidence](/quest/m1/battery-safety.md)
9. [Agree the ISS verification test plan](/quest/m1/iss-verification-plan.md)

### M2 · Verify the flight configuration

1. [Record the pre-test deployer fit](/quest/m2/pretest-deployer-fit.md)
2. [Complete assigned battery safety verification](/quest/m2/battery-verification.md)
3. [Verify layer-direction strength](/quest/m2/printed-strength.md)
4. [Verify creep under load and temperature](/quest/m2/printed-creep.md)
5. [Measure printed rail hardness and finish](/quest/m2/rail-surface.md)
6. [Verify printed rail wear and debris](/quest/m2/rail-wear-debris.md)
7. [Close outgassing and materials evidence](/quest/m2/material-outgassing.md)
8. [Verify integrated vibration](/quest/m2/integrated-vibration.md)
9. [Close the integrated shock requirement](/quest/m2/integrated-shock.md)
10. [Close the integrated EMC requirement](/quest/m2/integrated-emc.md)
11. [Verify thermal and vacuum behavior](/quest/m2/integrated-thermal-vacuum.md)
12. [Complete the provider’s deployer fit check](/quest/m2/deployer-fit-check.md)
13. [Submit the ISS qualification evidence](/quest/m2/iss-compliance-handover.md)

### Flight heritage and the suborbital research flight

These quests cover three rides for a classroom sensor kit: a glider, a suborbital research flight and the ISS. See the [suborbital flight plan](/docs/suborbital-flight-plan.md).

- [Fly with Perlan 2](/quest/m0/perlan-stratosphere.md)
- [Find the ISS: orbit, passes and deployment](/quest/m0/iss-orbit.md)
- [Predict the spaceflight](/quest/m0/predict-the-spaceflight.md)
- [One sensor, three rides](/quest/m0/one-sensor-three-rides.md)
- [Draft the suborbital payload package](/quest/m1/suborbital-payload-package.md)

### Satellites that help on Earth

Short examples of what small satellites, radio and sustainable materials do for people, pets and the kits themselves.

- [Wildfires: get a message out by satellite](/quest/m0/wildfire-message-relay.md)
- [Track a pet by its collar](/quest/m0/pet-collar-tracking.md)
- [Wool, fibre and static: what should cover the kit?](/quest/m0/wool-fiber-static.md)

### Parallel classroom, balloon and radio work

- [Teach the two-relay lesson](/quest/m0/repeat-the-station-lesson.md)
- [Compare a balloon bench log with flight data](/quest/m0/balloon-ground-log.md)
- [Predict a satellite pass](/quest/m0/predict-a-pass.md)
- [Review a supervised balloon flight](/quest/m1/balloon-partner-review.md)
- [Review the hosted experiment](/quest/m1/hosted-review.md)
- [Specify the first conducted RF test](/quest/m1/ground-station-rf.md)
- [Review the microscope collection plate proposal](/quest/m1/micro-collection-plate.md)
