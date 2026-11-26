# Rev C: a 1U CubeSat frame built to the CubeSat spec, in Rust

Rev C is a fully printed 1U frame (100 × 100 × 113.5 mm) that holds an **Arduino Uno R3** and an **Arduino Nano 33 BLE Sense Rev2**. Its CAD, its checks and its print files all come from one Rust crate, [`crates/cubesat-cad`](../../crates/cubesat-cad/src/lib.rs). The board firmware is Rust too: [`firmware/nano33-cubesat`](../../firmware/nano33-cubesat/README.md).

Every dimension is checked against the [CubeSat Design Specification (CDS) Rev 14.1](https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf) from Cal Poly ([cubesat.org](https://www.cubesat.org/)), using the processes in NASA's [CubeSat 101](https://www.nasa.gov/wp-content/uploads/2017/03/nasa_csli_cubesat_101_508.pdf). The full result is in the [compliance report](cds-report.md).

## What it is

| Part | What it does |
| --- | --- |
| Side panel ×2 (identical) | One face plus two corner rails: 9 mm square, every long edge rounded to 1 mm. An end wall at each end takes the snap pegs. |
| Cover ×2 (identical) | Closes a ±X face between the rails. A port for USB and the D2 jumper, and vent slots. |
| −Z end plate | Inserted first. Four columns carry the payload plate. Eight snap pegs: four for the panels, four for the covers. |
| +Z end plate | Opening over the payload. Eight snap pegs. |
| Payload plate | Uno R3 on four pins with two edge clips; Nano on two rails with an end stop and a strap. |
| Nano strap | Snaps over the Nano. |

There are no screws, nuts or metal standoffs. Each end plate clicks onto the side panels and covers with eight snap pegs, the same peg as the AMSAT kit's, whose snap test printed well on a P1S.

All six faces are closed, so fingers, cables and dropped tools stay out of the boards. The Uno's USB socket faces the −X cover's port, and the Nano's USB and the D2 jumper are reachable through it too. The panel and cover faces sit 0.5 mm inside the rail planes, so in a deployer only the rails slide on the walls.

## How it meets the spec

| CDS Rev 14.1 | Rev C |
| --- | --- |
| §2.2.1: 100 × 100 × 113.5 mm, origin at the centre | Same, in CAD. Measure the print. |
| §2.2.3: protrusions ≤ 6.5 mm | Nothing outside the rail envelope, boards included |
| §2.2.5, §2.2.7: rails ≥ 8.5 mm, edges rounded ≥ 1 mm | 9.0 mm rails, 1.0 mm radius |
| §2.2.8: rail ends give ≥ 6.5 × 6.5 mm contact | 7.0 × 7.0 mm flat; end plates and panels stop 1.5 mm short of the ends |
| §2.2.9: ≥ 75% rail contact | Continuous rails: 100% |
| §2.2.3, §2.2.9: rails guide the CubeSat | Faces 0.5 mm inside the rail planes; only the rails can touch |
| §2.1.9: venting | About 15 800 mm² of openings for 930 cm³ inside: 2.3 in, far under the 2000 in limit |
| §2.2.10, §2.2.11: mass ≤ 2.00 kg, centre of mass within ±2 cm | About 270 g solid; centre of mass about 7 mm towards −Z |
| §2.2.6, §2.2.12, §2.2.13, §2.1.7: aluminium, anodised, low outgassing | **Deviation.** Printed PLA, for the classroom |
| §2.3.2, §2.3.5: deployment switch, RBF pin | **Deviation.** The D2 jumper inhibits the firmware; there is no rail-end switch |
| §2.4.5: 45 min radio silence after deployment | The firmware's `Timing::Cds` setting |

A printed classroom frame can't meet the material and electrical requirements. The report lists them as deviations rather than hiding them, because a flight version would need a different material or part, or the launch provider's approval.

## Print

Bambu Lab P1S, PLA, supports off. Use the profile for your nozzle (0.30mm Standard for 0.6 mm).

| Plate | Parts | 0.6 mm nozzle |
| --- | --- | --- |
| [00-snap-test](p1s/00-snap-test.3mf) | Peg on a boss, and a standing wall with the teardrop hole | 7 min, 2 g |
| [01-side-panels](p1s/01-side-panels.3mf) | Both side panels | 1 h, 65 g |
| [02-covers](p1s/02-covers.3mf) | Both covers | 52 min, 49 g |
| [03-end-plates](p1s/03-end-plates.3mf) | −Z and +Z end plates | 1 h 31 min, 59 g |
| [04-payload](p1s/04-payload.3mf) | Payload plate and Nano strap | 49 min, 31 g |

That's about 4 h 20 min and 210 g for the whole frame (PrusaSlicer estimates with a P1S 0.6 mm profile; Bambu Studio's will differ a little). **Print the snap test first:** the side panels' holes print sideways as teardrops, unlike the AMSAT kit's round holes, so check that the peg still clicks in and holds.

[assembly.stl](assembly.stl) shows every part in place.

## Assemble

1. Uno onto the four pins, USB port towards the clip between the jacks. Press until both clips click.
2. Nano (headers soldered, pins down) onto the rails, USB end away from the end stop. Push it against the stop and snap the strap over the two posts.
3. Payload plate onto the −Z end plate's four columns until they click.
4. Snap both side panels onto the −Z end plate, rails outward.
5. Slide each cover down between the rails, port towards −Z, and snap it on.
6. Snap the +Z end plate on top. To reach the jumper or USB, use the covers' ports; to reach the boards, unsnap the +Z plate and a cover.

## Launch loads and protection

Rev C is checked against the loads it would see on a launch, not only on a desk. The numbers are hand calculations in [`loads.rs`](../../crates/cubesat-cad/src/loads.rs), each with its source, and the report lists every margin. They are estimates for a design review. A shake or drop test is the evidence.

| Load case | Level | Source |
| --- | --- | --- |
| Quasi-static | 20 g on each axis | Common CubeSat design load |
| Classroom rocket boost | 10 g | AeroTech F39T, thrust-to-weight about 11 |
| Ejection and landing shock | 50 g | **Assumption.** Measure it with a flight accelerometer and change `ROCKET_SHOCK_G` |
| Orbital launch, random vibration | 14.1 Grms, 3σ response by Miles' equation, Q = 10 | GEVS (GSFC-STD-7000A), which the CDS (§3) uses when a launch provider gives no levels |

- **Factor of safety 2.0** on everything (NASA-STD-5001B for bonded structures: a print is layers bonded together).
- **Printed PLA** allowables of 30 MPa within a layer and 15 MPa between layers, halved for shear. These are assumptions rounded down from published tests. Anything that bends across the layers uses the lower figure.
- **Margins:** columns, all three kinds of snap, the panels' and covers' end walls, the Uno's pins and clips, and the Nano strap. Each must have a positive margin at the classroom level (50 g), or the check fails.
- **Stiffness:** each part's lowest mode must be at least 100 Hz (JAXA J-SSOD), so it doesn't resonate with the launcher. The weakest is the −Z end plate at about 150 Hz.
- **GEVS:** Rev C has positive margins at the qualification level too. The 8 mm columns are what make that true; with 7 mm columns the report shows a deviation instead of hiding it.

Protection from the rest of a launch:

- **Covers** on ±X keep debris and fingers out, and every face sits inside the rail planes.
- **Vents** let the air out as the fairing depressurises (§2.1.9), so the covers can't be pushed off.
- **Root chamfers** on the columns and bosses spread the load where printed posts usually snap.
- **Watchdog:** the [firmware](../../firmware/nano33-cubesat/README.md#watchdog-and-reset-cause) restarts itself if it hangs, and the next boot reports why.
- **Heat:** PLA softens at about 55 °C. Don't leave Rev C in a car or in direct sun. For a balloon or rocket flight that may sit on a hot pad, print in PETG or ASA and rerun your fit checks.

**Rockets.** A 1U CubeSat doesn't fit a classroom rocket built for a 24 mm motor. To fly on one, take the payload plate out as a FlatSat (see the project's rockoon plan). A whole 1U needs a larger rocket and its club's range safety rules. Either way, a licensed mentor or club leads the launch.

## Test it like a flight unit

Space programs test one unit hard (the qualification unit) and fly a second one tested more gently (the flight unit). A class can do the same with two prints. Label every result: *desk simulation* for these calculations, *hardware bench* for what you measure.

1. **Inspect and measure.** Calipers on the envelope (100.0 × 100.0 × 113.5 mm ±0.1), the rails and the rail ends. Weigh it, and balance it on a pencil along each axis to find the centre of mass.
2. **Function test.** Power on, `STATUS`, one beacon. Repeat this after every test below; a test only passes if this still works afterwards.
3. **Drop test** (qualification unit only). Drop it from 30 cm onto a carpeted floor, once on each face. Look for cracks at the column and boss roots, and check every snap still holds.
4. **Shake test.** Tape a phone running an accelerometer app to the frame and shake it by hand, or mount it on a speaker playing a tone sweep. Look for the frequency where it rattles and compare it with the report's estimates.
5. **Thermal check.** An hour in a fridge, then an hour in a warm room. Re-measure and run the function test.

## Checks

`cargo run -p cubesat-cad --release -- check` runs every check. The same command with `build hardware/rev-c` also rewrites the files here. The checks cover:

- **CDS geometry:** size, protrusions, rails, rail ends, rail contact, recessed faces, venting, mass and centre of mass
- **Launch loads:** margins of safety for every load-carrying feature, and the lowest frequency of each part
- **Fit:** parts don't overlap, every snap peg passes through its hole, and the boards clear the posts, clips, rails, strap, solder joints, header pins and frame
- **Latching:** barb overlap, clip reach, and clip and peg bending strain against PLA
- **Printing:** each part is one closed solid, fits the P1S, and has no overhang that needs support

`cargo test -p cubesat-cad --release` also breaks the design 23 ways on purpose and confirms each check catches its fault.

## Where it comes from

Rev C is new geometry. It borrows ideas, not files:

- The CDS and CubeSat 101 for the requirements.
- The [AMSAT CubeSatSim](https://cubesatsim.org) frame by Alan Johnston, KU2Y, for the split into end plates and sides, and the payload plate and snap pegs from this project's [AMSAT kit](../amsat-cubesatsim/README.md).
- The [Teachers in Space](https://www.tis.org/) frame's Multiplate, for carrying an Arduino on a plate.

It's licensed like the rest of the project: MIT or Apache 2.0.
