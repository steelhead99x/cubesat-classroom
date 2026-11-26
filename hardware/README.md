# Hardware

The printed 1U CubeSat frame (100 × 100 × 113.5 mm) and the CSC-R1 ground-station receiver board.

| Design | What it is | Start here |
| --- | --- | --- |
| **AMSAT CubeSatSim frame + Arduino payload** | AMSAT's CubeSatSim v2.0 frame with a printed plate for an Arduino Uno R3 and a Nano 33 BLE Sense. 100% printed, no supports, no screws. The payload plate is modeled on the Teachers in Space (TIS) Multiplate. **Status: nearly verified.** The design checks pass in CI; the last print-and-fit checks are wrapping up. | [Build guide](amsat-cubesatsim/README.md) |
| Teachers in Space frame | The frame TIS uses in its Arduino CubeSat workshops, and the model for our payload plate. Files come from TIS. | [Links and comparison](tis/README.md) |
| CSC-R1 receiver dev kit | Receive-only SDR board for the 2 m and 70 cm satellite bands, around a XIAO. Designed and rule-checked; not built or tested. Plus the plan for a pedal-enclosure version with HaLow. | [Design and review](sdr-receiver/README.md) |

Build the AMSAT frame with the Arduino payload. Print its test coupons first.

## Credit

- **Frame:** AMSAT [CubeSat Simulator](https://cubesatsim.org) v2.0 by Alan Johnston, KU2Y, from [alanbjohnston/CubeSatSim](https://github.com/alanbjohnston/CubeSatSim/tree/master/hardware/frame/v2.0) (GPL-3.0). Our fork and its parts are GPL-3.0 too ([LICENSE](amsat-cubesatsim/LICENSE)).
- **Payload plate:** modeled on the `Multiplate` from the [Teachers in Space](https://www.tis.org/) 1U frame (see [tis/README.md](tis/README.md)).
- This is an independent build, not made or endorsed by AMSAT or TIS.

Every structural part, cover, mount, retainer and fit tool the project designs must have a version that prints on a common desktop printer with a 256 mm bed. Electronics, wiring, cells, sensors and RF parts are chosen separately.

## Regenerate the AMSAT kit

The AMSAT kit is built in Rust: `cargo run --release -p amsat-kit` (see [its README](amsat-cubesatsim/README.md#regenerate)). CI fails if the committed files differ from the output.

For the limits of printed parts in flight, see [NASA CubeSat 101](https://science.nasa.gov/wp-content/uploads/2023/06/nasa_csli_cubesat_101_508.pdf).

## Archived experiments

Earlier frame experiments are no longer offered as builds. Their files stay in this folder because CI and the flight-research quests still build or reference them; see [ARCHIVE.md](ARCHIVE.md).
