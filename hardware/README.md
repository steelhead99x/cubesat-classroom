# Hardware

The printed 1U CubeSat frame (100 × 100 × 113.5 mm), the 1U solar face and the CSC-R1 ground-station receiver board.

| Design | What it is | Start here |
| --- | --- | --- |
| **AMSAT CubeSatSim frame + Arduino payload** | AMSAT's CubeSatSim v2.0 frame with printed plates for an Arduino Uno R3, a Nano 33 BLE Sense and a 3000 mAh battery. 100% printed, no supports, no screws, no loose parts. The payload plate is modeled on the Teachers in Space (TIS) Multiplate. **Status: nearly verified.** The design checks pass in CI; the last print-and-fit checks are wrapping up. | [Build guide](amsat-cubesatsim/README.md) |
| Outdoor ground station | The same payload plate in a sealed box with a gasket, cable gland, vent, battery and a 1/4"-20 tripod thread. Design checks only. | [Build guide](outdoor-station/README.md) |
| Outdoor XIAO station | The outdoor box with a XIAO plate: XIAO ESP32-S3 on Wi-Fi HaLow, XIAO MG24 Sense, battery, two SMA antenna ports. Design checks only. | [Build guide](outdoor-xiao/README.md) |
| Teachers in Space frame | The frame TIS uses in its Arduino CubeSat workshops, and the model for our payload plate. Files come from TIS. | [Links and comparison](tis/README.md) |
| CSC-R1 receiver dev kit | Receive-only SDR board for the 2 m and 70 cm satellite bands, around a XIAO. Designed and rule-checked; not built or tested. Plus the plan for a pedal-enclosure version with HaLow. | [Design and review](sdr-receiver/README.md) |
| 1U solar face | One 82.5 × 98 mm side-panel board in two versions that differ only in their cells: 24 silicon SolarBITs for the bench, or two AZUR triple-junction space cells for flight. Designed and rule-checked; cell land patterns not yet verified, so no Gerbers yet. | [Design and review](solar-face/README.md) |

Build the AMSAT frame with the Arduino payload. Print its test coupons first. Every printed design follows the [design guide](DESIGN-GUIDE.md): no loose pieces where possible, rocket-launch ready and durable, and easy enough for kids.

## Credit

- **Frame:** AMSAT [CubeSat Simulator](https://cubesatsim.org) v2.0 by Alan Johnston, KU2Y, from [alanbjohnston/CubeSatSim](https://github.com/alanbjohnston/CubeSatSim/tree/master/hardware/frame/v2.0) (GPL-3.0). Our fork and its parts are GPL-3.0 too ([LICENSE](amsat-cubesatsim/LICENSE)).
- **Payload plate:** modeled on the `Multiplate` from the [Teachers in Space](https://www.tis.org/) 1U frame (see [tis/README.md](tis/README.md)).
- This is an independent build, not made or endorsed by AMSAT or TIS.

Every structural part, cover, mount, retainer and fit tool the project designs must have a version that prints on a common desktop printer with a 256 mm bed. Electronics, wiring, cells, sensors and RF parts are chosen separately.

## Regenerate the AMSAT kit and outdoor cases

The AMSAT kit and both outdoor cases are built in Rust: `cargo run --release -p amsat-kit` (see [its README](amsat-cubesatsim/README.md#regenerate)). CI fails if the committed files differ from the output.

For the limits of printed parts in flight, see [NASA CubeSat 101](https://science.nasa.gov/wp-content/uploads/2023/06/nasa_csli_cubesat_101_508.pdf).

## Archived experiments

Earlier frame experiments are no longer offered as builds. Their files stay in this folder because CI and the flight-research quests still build or reference them; see [ARCHIVE.md](ARCHIVE.md).
