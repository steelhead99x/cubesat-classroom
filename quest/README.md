# CubeSat Classroom quest log

## Goal

Pass working lessons and well-documented evidence from students, teachers and radio volunteers to the next crew. Use the [quick-start guide](../docs/quick-start.md) for the workflow. The Markdown format comes from [moq-dev/quest](https://github.com/moq-dev/quest).

## Plan

### Start with what you have

Write down the equipment you have, including the board or printer model, available sensors, and computer operating system. If you have several boards, pick one for the first run. No hardware is needed to get started.

| Your setup | First quest |
| --- | --- |
| Bambu Lab P1S | [Print the AMSAT frame and Arduino payload](/hardware/amsat-cubesatsim/README.md), starting with the 5-minute snap test |
| Computer only or unsure | [Teach the two-relay lesson](/quest/m0/repeat-the-station-lesson.md) |
| Arduino, such as an Uno | [Read one Arduino sample](/quest/m0/arduino-first-reading.md) |
| Seeed XIAO, such as ESP32-C3 or RP2040 | [Record a XIAO bench sample](/quest/m0/xiao-first-reading.md) |
| Arduino Nano 33 BLE Sense Rev2 | [Fly the Arduino bench CubeSat](/quest/m0/arduino-bench-cubesat.md) |
| Seeed XIAO with the XIAO Expansion Board and Grove sensors | [Fly the Seeed XIAO bench CubeSat](/quest/m0/xiao-bench-cubesat.md) |
| Raspberry Pi with Raspberry Pi OS | [Log a Pi reading](/quest/m0/pi-first-reading.md) |
| Any computer, no extra hardware | [Predict a satellite pass](/quest/m0/predict-a-pass.md) |
| Raspberry Pi Pico or another microcontroller | Use a vendor sample for your exact board and follow the [microcontroller quest structure](/quest/m0/arduino-first-reading.md). |

### Mentor quests

Step-by-step, hands-on demos that a teacher, guardian or mentor guides for a small group. Every mentor quest follows the [mentor rules](/quest/m0/mentor-get-ready.md#mentor-rules): the mentor takes part for 100% of the demo and every learner does each step, no login is ever shared with anyone under 13, and the mentor checks safety before anything is powered.

1. [Get ready to guide a demo](/quest/m0/mentor-get-ready.md)
2. [Guide a first ground station demo](/quest/m0/mentor-ground-station-demo.md)
3. [Guide a first sensor reading](/quest/m0/mentor-first-sensor-demo.md)
4. [Guide a satellite pass prediction](/quest/m0/mentor-satellite-pass-demo.md)
5. [Help a learner lead the next demo](/quest/m0/mentor-pass-it-on.md)

### Teacher pilot

Start in M0. After a run, tell us which steps the class completed, which instruction they got stuck on, and one change you would make. Share a redacted note through the [contribution guide](../CONTRIBUTING.md). Teachers can pilot and review lessons before any flight work.

### Bambu P1S case test track

Print the Rev A case with the [P1S print guide](/hardware/p1s/README.md), then use these quests to record what you measured so another crew can repeat it. Start with PLA and go in order. The Rev B rail-fit and ISS work has its own path below.

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
- [Fly the Arduino bench CubeSat](/quest/m0/arduino-bench-cubesat.md) or the [Seeed XIAO bench CubeSat](/quest/m0/xiao-bench-cubesat.md): deploy, lose the signal, recover stored readings, and bring back data. See the [bench CubeSat guide](/docs/bench-cubesats.md).

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

### CSC-R1 receiver bench

Measure the [CSC-R1 receiver](/hardware/sdr-receiver/README.md) and its Rev B radios with cheap gear, starting with an RTL-SDR Blog V4. The [bench kit](/hardware/sdr-receiver/bench/README.md) has the scripts, a shopping list and the safety rules. The first six need no CSC-R1 board.

1. [First light with the RTL-SDR Blog V4](/quest/m0/receiver-v4-first-light.md)
2. [Measure FM broadcast at your antenna](/quest/m0/receiver-fm-at-your-site.md)
3. [Build a shielded test signal](/quest/m0/receiver-test-signal.md)
4. [Sweep a filter with a noise source](/quest/m0/receiver-sweep-a-stub.md)
5. [Does HaLow noise land in the receive bands?](/quest/m0/receiver-halow-noise.md)
6. [Compare radio power: 2.4 GHz, 5 GHz and HaLow](/quest/m0/receiver-radio-power.md)
7. [Bring up CSC-R1 power and control](/quest/m1/receiver-rev-a-power-up.md)
8. [Write the CSC-R1 Rev A bring-up firmware](/quest/m1/receiver-rev-a-firmware.md)
9. [Check the CSC-R1 local oscillator and 5 V under load](/quest/m1/receiver-rev-a-lo-and-supply.md)
10. [Measure the CSC-R1 band filters](/quest/m1/receiver-rev-a-filters.md)
11. [Measure HaLow desense on the CSC-R1](/quest/m1/receiver-rev-a-desense.md)
12. [Compare the CSC-R1 with the RTL-SDR V4 on a real pass](/quest/m1/receiver-rev-a-vs-v4.md)

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

### Public site and Worker

Maintainer work for the live classroom site. These quests stay in the log after the pull requests land, so the next crew can re-check the same behavior.

- [Fail the Worker deploy when Cloudflare secrets are missing](/quest/m0/fail-deploy-without-secrets.md)
- [Redirect www hosts to their apex on the public Worker](/quest/m0/redirect-www-hosts.md)
- [Make classroom quest check pass on the existing lesson files](/quest/m0/classroom-quest-check-headings.md)
- [Move cubesat.app to the app Worker and align site branding](/quest/m0/cubesat-app-hostname-move.md)
- [Patch the url advisory and add a dependency audit to the classroom Rust workspace](/quest/m0/classroom-rust-advisories.md)
- [Align the public site copy with the volunteer review](/quest/m0/classroom-public-copy.md)
- [Run cubesat-classroom CI on PrairieDog runners](/quest/m0/prairiedog-ci.md)

### Parallel classroom, balloon and radio work

- [Teach the two-relay lesson](/quest/m0/repeat-the-station-lesson.md)
- [Compare a balloon bench log with flight data](/quest/m0/balloon-ground-log.md)
- [Predict a satellite pass](/quest/m0/predict-a-pass.md)
- [Review a supervised balloon flight](/quest/m1/balloon-partner-review.md)
- [Review the hosted experiment](/quest/m1/hosted-review.md)
- [Specify the first conducted RF test](/quest/m1/ground-station-rf.md)
- [Review the microscope collection plate proposal](/quest/m1/micro-collection-plate.md)
