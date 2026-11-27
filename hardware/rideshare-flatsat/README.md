# Rideshare payload integration lab

A teacher plays the payload integrator. Each crew delivers an instrument record and the evidence for one configuration. Use one, three, or five nodes in **one classroom experiment**. These counts describe independent loggers. The flight provider and dispenser integrator determine the spacecraft envelope, interfaces and verification baseline.

## Start with a desk simulation

From this directory, run the commands below with Python 3. Each simulation creates a new directory to preserve earlier evidence. The CSV files contain generated acceleration in milligravity (mg), where 1000 mg is one standard gravity.

```sh
python3 lab.py simulate 1 run-1
python3 lab.py compare run-1/ground.csv run-1/node-*.csv
python3 lab.py simulate 3 run-3
python3 lab.py compare run-3/ground.csv run-3/node-*.csv
python3 lab.py simulate 5 run-5
python3 lab.py compare run-5/ground.csv run-5/node-*.csv
python3 lab.py compare run-5/ground-recovered.csv run-5/node-*.csv
```

Each run also writes `ground-recovered.csv`, the complete ground feed. In the five-node run, `ground.csv` deliberately misses two of node 3's readings; compare both files for the discrepancy report. The last line of each comparison counts the discrepancies. Add `--strict` to make the command fail (exit status 1) when there are any, for example in a script that checks a run:

```sh
python3 lab.py compare --strict run-5/ground.csv run-5/node-*.csv            # fails: 2 missing
python3 lab.py compare --strict run-5/ground-recovered.csv run-5/node-*.csv  # passes
```

| Nodes | Crew activity | Deliverable for the integration review |
|---|---|---|
| 1 | Tip the simulated sensor after five samples. Identify which axis responds. | A labeled axis sketch and a reference to the raw record. Relate the sketch to guide §2.3. |
| 3 | Compare three sensor mounting angles during the same tip. In a bench run, mark the shared event since clocks start independently. | A configuration drawing identifying each sensor, its axes and connector. Relate it to Appendix C. |
| 5 | Compare local records with the intentionally interrupted ground feed. Identify the affected node, boot and sequence numbers. | A discrepancy report. Repeat with a recovered ground file and explain what changed. Relate it to §6.4. |

## Baseline hardware for this class

Use one of the existing all-in-one XIAO development boards per node. Count the complete carrier, installed XIAO, SD card and power accessories as one configuration. Use three or five of the same revision when comparing nodes. Start on USB with the coin-cell holder and charging connector empty. Record the product code, processor, SD slot and sensor on the configuration drawing before selecting firmware. Verify the coin-cell role, supported chemistry and charger connector against the exact carrier manual before fitting a cell. An RTC backup cell and a rechargeable main battery serve different purposes.

If the carrier is Seeed’s XIAO Expansion Board (103030356), the [existing bench instructions](../../docs/bench-cubesats.md) and [carrier firmware](../../firmware/xiao-cubesat/xiao-cubesat.ino) support its microSD slot and RTC. Each crew should save one native SD log under its node ID, power-cycle and check that the earlier readings are still present. Preserve the native CSV header and units. The simulation comparison tool expects the eight-field format below. The exact board revision, onboard sensors and HaLow interface still need a bench check.

## Optional paired-board reference setup

For each node use a Seeed XIAO **MG24 Sense**, a XIAO **ESP32S3 Sense with its Sense expansion board**, a compatible formatted microSD card and USB power. With power disconnected, connect MG24 D6/TX to S3 D7/RX and join GND. Power each board by its own USB input. The only inter-board connections for this experiment are data and ground. Seeed documents the [MG24 IMU and library](https://wiki.seeedstudio.com/xiao_mg24_sense_built_in_sensor/) and the [S3 board and microSD interface](https://wiki.seeedstudio.com/xiao_esp32s3_getting_started/).

1. Select the XIAO MG24 Sense target in the Silicon Labs Arduino core, install Seeed Arduino LSM6DS3, and upload `mg24_sensor/mg24_sensor.ino`. Select XIAO ESP32S3 in the Espressif Arduino core, enable USB CDC on boot and set a unique NODE value in `s3_recorder/s3_recorder.ino`. Insert the card and upload the recorder sketch.
2. Open the S3 USB serial monitor at 115200. Set the board still, turn it onto another face, and observe the axes. The nominal interval is about 100 ms. Use the recorded sensor timestamps for analysis. A missing IMU at startup yields no samples. A recorder open or write failure produces an SD error on USB.
3. Each headerless row contains evidence label, node, recorder session ID, sequence, sensor uptime in ms, then X/Y/Z acceleration in mg. Logs on the card are named by node and session ID. After stopping and powering down, copy each card's CSV files into its own folder. Preserve raw copies. Record firmware revision, board revision, date and orientation beside them.
4. For classroom HaLow telemetry, connect a sending Linux host and a receiving host through an already configured HaLow IP bridge. The S3 sends its saved rows over USB to the sending host. Set the destination and bind addresses to the HaLow interfaces. This example requires the external bridge and a USB host beside the recorder. It implements no onboard HaLow radio driver. Keep the card as the authoritative copy. A radio mentor reviews the module, antenna and permitted operating mode before any RF test.
5. On the receiving host, use a fresh output filename. On the sending host, close the serial monitor and substitute the actual S3 port. Start one forwarder per S3. Stop capture with Ctrl-C and compare against the card copies.

```sh
# Receiver (substitute its HaLow-side address)
python3 lab.py receive 192.0.2.2 > ground-run.csv
# Sender (Linux)
stty -F /dev/ttyACM0 115200 raw -echo
python3 lab.py forward 192.0.2.2 < /dev/ttyACM0
# After copying the cards
python3 lab.py compare ground-run.csv cards/*/*.csv
```

A matching UDP capture proves host delivery. To label a run **received RF**, also record the bridge interfaces, route and observed radio-link counters. The desk and bench activities leave flight verification open.

## Rideshare review worksheet

Use a fresh copy of this table per configuration. Fill in revision, evidence filename, date, reviewer role and disposition. Each row begins open. The references below use SpaceX Rideshare Payload User’s Guide **version 11, August 2026**. The accepted request identifies the applicable guide version. Obtain the signed interface control document (ICD) and applicable dispenser requirements from the integrator before assigning flight acceptance criteria. [Public guide](https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf) · [Rideshare terms, applicable documents](https://storage.googleapis.com/rideshare-static/SpaceX_Rideshare_Terms_and_Conditions.pdf)

| Guide reference | Classroom evidence to collect | Input or review still required for flight | Evidence / disposition |
|---|---|---|---|
| §2, Appendix A — interface dimensions; Appendix C — CAD deliverables | Dimensioned assembly drawing with axes, connectors and protrusions | Selected dispenser drawing, tolerances and integrator fit disposition | Open |
| §3.1 — mass properties | Weighed as-built configuration, scale resolution and reference axes | Accepted mass properties including center of gravity and inertia as applicable | Open |
| §§4, 5.2–5.3 — environment and construction | Parts list, materials, fasteners and configuration photos | Provider environmental envelope, materials review and retention evidence | Open |
| §§6.1–6.7 — verification | Test procedure, configuration revision, raw files, discrepancies and retest record | Integrator-approved verification plan and applicable test levels for the integrated or containerized unit | Open |
| §7 — electrical interfaces | Power and data diagram, USB/SD/radio states and bench current measurements | Approved electrical interface and mission-specific power and RF constraints | Open |
| §§9.4–9.5, §11, Appendices F–H — integration and safety | Inventory, radio description, evidence index and open decisions | Mission documentation, applicable authorizations, safety review and customer deliverables | Open |

AMSAT CubeSatSim supplies the educational context. The local frame currently has an Uno/Nano carrier. Measure the all-in-one carrier and check its fit separately. Cal Poly’s CubeSat Design Specification provides a further dimensional reference once a form factor and dispenser are selected. [AMSAT CubeSatSim](https://www.cubesatsim.org/) · [Local frame](../amsat-cubesatsim/README.md) · [Cal Poly specification index](https://www.cubesat.org/cubesatinfo)
