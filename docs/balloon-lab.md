# Weather balloon lab

**Question.** How do air pressure and temperature change during a balloon flight? How do we know if part of the record is missing?

The ground lab uses the same timestamped file format the proposed balloon experiment would return. It works with a pressure sensor and a thermometer, or with two fixture files for a class without hardware. The first class does the work on a bench. A flight is a separate activity led by experienced adults.

## Build and rehearse on the ground

Give the class four jobs: payload lead, station operator, data recorder and reviewer. For younger grades, read values and units aloud and mark what changed. For older grades, graph both series against elapsed time and explain timing, units, calibration and missing points.

1. Start the [classroom ground station](../README.md). Get a reading, start the video, then practise a contact interruption and download the crew log. The station demonstrates communications and separately labels its simulated camera.
2. Connect an instructor-approved pressure sensor and thermometer using their supplied adapter or driver. Expose current integer values in two files, `pressure_pa` and `temperature_mc`, in one folder. Pressure is in pascals and temperature is in millidegrees Celsius. Check units against the instrument documentation. A local reference reading and a note on placement help the next group interpret the data.
3. Run the logger for one minute. It writes once per second, with a sequence, local UTC timestamp, elapsed time and data status. Observe the live instrument while it records. If one reading fails, its cell stays empty and the other reading remains available.

   ```sh
   cargo run --locked -- balloon-log --sensor-dir ./sensors --output ground-run.csv --samples 60
   ```

4. Repeat with a new output file. During this second test disconnect a **fixture input** or stop its adapter briefly under instructor supervision. Mark that time in the crew log, reconnect it, and verify the CSV records loss and recovery. If only software is available, create ordinary fixture files using the example below. Label their data as a simulation.
5. Open the CSV in a spreadsheet. Plot `pressure_pa / 100` as hPa and `temperature_mc / 1000` as °C. Keep missing values as gaps. Compare the shape with the May 18, 2026 Teachers in Space high-altitude balloon dataset and distinguish the bench and flight conditions. [Teachers in Space flight-data archive](https://www.tis.org/flightdata)

```sh
mkdir -p run/balloon-fixture
printf '101325\n' > run/balloon-fixture/pressure_pa
printf '21500\n' > run/balloon-fixture/temperature_mc
cargo run --locked -- balloon-log --sensor-dir run/balloon-fixture --output run/fixture.csv --samples 10
```

A passing ground rehearsal returns the planned sample count, explicit gaps for the deliberately disconnected input, and a new valid reading after it returns. The reviewed worksheet should include hardware or fixture provenance, units, run date, a chart, and one limitation. The current station reads temperature, voltage and light on command. The balloon logger records pressure and temperature independently of that link.

## Prepare for a supervised flight

Use a partner experienced in high-altitude balloon launches and recovery. Ask them to define the payload interface, mounting and retention, pressure sensing exposure, sensor range, temperature limits, power and battery handling. Have the team prove that the logger keeps recording if the communications link is absent. Use the launch partner's approved tracking and recovery system independently of the experiment. A successful first flight is retrieval of a useful, timestamped data record. Treat optional radio reception as its own experiment.

Before any launch, the adult flight lead owns the go/no-go decision and confirms airspace and flight rules, notifications or approvals as applicable, ground hazards, acceptable weather and recovery access. For a U.S. launch, review the actual package and operation against the applicable provisions of 14 CFR Part 101 with the launch partner. Verify spectrum authorization separately before any transmitting configuration. [14 CFR Part 101](https://www.ecfr.gov/current/title-14/chapter-I/subchapter-F/part-101) · [FAA unmanned balloon procedures](https://www.faa.gov/air_traffic/publications/atpubs/atc_html/chap9_section_6.html)

The PLA and ASA parts supplied here are desktop mockups. An actual balloon payload needs its own host-approved mounting, materials, battery and recovery design.
