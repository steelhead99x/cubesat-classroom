# Rev C: a 1U CubeSat frame built to the CubeSat spec, in Rust

Rev C is a fully printed 1U frame (100 × 100 × 113.5 mm) that holds an **Arduino Uno R3** and an **Arduino Nano 33 BLE Sense Rev2**. Its CAD, its checks and its print files all come from one Rust crate, [`crates/cubesat-cad`](../../crates/cubesat-cad/src/lib.rs). The board firmware is Rust too: [`firmware/nano33-cubesat`](../../firmware/nano33-cubesat/README.md).

Every dimension is checked against the [CubeSat Design Specification (CDS) Rev 14.1](https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf) from Cal Poly ([cubesat.org](https://www.cubesat.org/)), using the processes in NASA's [CubeSat 101](https://www.nasa.gov/wp-content/uploads/2017/03/nasa_csli_cubesat_101_508.pdf). The full result is in the [compliance report](cds-report.md).

## What it is

| Part | What it does |
| --- | --- |
| Side panel ×2 (identical) | One face plus two corner rails: 9 mm square, every long edge rounded to 1 mm. An end wall at each end takes the snap pegs. |
| Cover ×2 (identical) | Closes a ±X face between the rails, with vent slots. On the −X side its port passes both USB plugs and the D2 jumper. |
| −Z end plate | Inserted first. Four hollow columns carry the payload plate, each with a cross-bar for a quarter-turn stud. Eight snap pegs: four for the panels, four for the covers. |
| +Z end plate | Opening over the payload. Eight snap pegs. |
| Payload plate | Uno R3 on four posts, held by four snap latches. The Nano 33 BLE Sense stands on its edge in a V-groove card cage. |
| Nano clamp | Caps the Nano's top edge, held by two quarter-turn studs. |
| Lock keys ×4 | Clip in behind the Uno latches so they can't open. Print them red, like flight safety pins. |
| Quarter-turn studs ×6, spring washers ×6 | Four hold the plate to the columns; two hold the Nano clamp. |

There are no screws, nuts or metal standoffs. Each end plate clicks onto the side panels and covers with eight snap pegs, the same peg as the AMSAT kit's, whose snap test printed well on a P1S. The payload uses the AMSAT kit's board mounts and fasteners, with the same numbers: Uno latches with lock keys, the Nano card cage, and printed quarter-turn studs (a generic push-and-turn panel fastener) with spiral spring washers. 

All six faces are closed, so fingers, cables and dropped tools stay out of the boards. Both USB sockets face −X, and the −X cover's port is sized from the plug envelopes so both cables fit, with the D2 jumper reachable the same way. The panel and cover faces sit 0.5 mm inside the rail planes, so in a deployer only the rails slide on the walls.

## How it meets the spec

| CDS Rev 14.1 | Rev C |
| --- | --- |
| §2.2.1: 100 × 100 × 113.5 mm, origin at the centre | Same, in CAD. Measure the print. |
| §2.2.3: protrusions ≤ 6.5 mm | Nothing outside the rail envelope, boards included |
| §2.2.5, §2.2.7: rails ≥ 8.5 mm, edges rounded ≥ 1 mm | 9.0 mm rails, 1.0 mm radius |
| §2.2.8: rail ends give ≥ 6.5 × 6.5 mm contact | 7.0 × 7.0 mm flat; end plates and panels stop 1.5 mm short of the ends |
| §2.2.9: ≥ 75% rail contact | Continuous rails: 100% |
| §2.2.3, §2.2.9: rails guide the CubeSat | Faces 0.5 mm inside the rail planes; only the rails can touch |
| §2.1.9: venting | About 15 300 mm² of openings for 930 cm³ inside: 2.4 in, far under the 2000 in limit |
| §2.2.10, §2.2.11: mass ≤ 2.00 kg, centre of mass within ±2 cm | About 280 g solid; centre of mass about 8 mm towards −Z |
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
| [02-covers](p1s/02-covers.3mf) | Both covers | 54 min, 50 g |
| [03-end-plates](p1s/03-end-plates.3mf) | −Z and +Z end plates | 1 h 33 min, 61 g |
| [04-payload](p1s/04-payload.3mf) | Payload plate and Nano clamp | 1 h 4 min, 33 g |
| [05-fasteners](p1s/05-fasteners.3mf) | 4 lock keys, 4 plate studs, 2 clamp studs, 6 spring washers | 19 min, 5 g at the profile's infill |

That's about 5 hours and 215 g (PrusaSlicer estimates with a P1S 0.6 mm profile; Bambu Studio's will differ a little). Print the fasteners at 100% infill: they're small and carry the load. **Print the snap test first:** the side panels' holes print sideways as teardrops, unlike the AMSAT kit's round holes, so check that the peg still clicks in and holds.

[assembly.stl](assembly.stl) shows every part in place.

## Assemble

1. **Plate to columns first.** Set the payload plate on the −Z end plate's four columns. Put a spring washer, feet down, over each hole, push a plate stud through, and turn it a quarter turn clockwise until it clicks. Three of the studs sit under the Uno, so they go in before it.
2. **Uno.** USB port towards −X. Press it straight down until all four latches click, then push a lock key into the slot behind each latch until it clips in.
3. **Nano** (bare board, no headers). Stand it on its long edge in the groove, parts facing the Uno, USB end towards −X against the corner stop.
4. **Nano clamp.** Lay it over the Nano's top edge, a washer and clamp stud at each pillar, and turn both studs a quarter turn until they click.
5. Snap both side panels onto the −Z end plate, rails outward.
6. Slide each cover down between the rails, port towards −Z, and snap it on. The −X cover's port goes on the Nano's side.
7. Snap the +Z end plate on top.

To undo a stud, push it in and turn it back. To take the plate off, take the Uno off first. To reach the boards, unsnap the +Z plate and a cover.

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
- **Margins** for the columns, the plate studs' bars, the end-plate snaps, the panels' and covers' end walls, the Uno's pins and latch hooks, and the Nano clamp. Each must be positive at the classroom level (50 g), or the check fails.
- **Stiffness:** each part's lowest mode must be at least 100 Hz (JAXA J-SSOD), so it doesn't resonate with the launcher. The weakest is the −Z end plate at about 140 Hz.
- **The plate lifts above about 30 g.** That's where the payload outweighs the spring washers. In a 50 g shock it lifts a fraction of a millimetre and drops back. The studs' bars still hold it, with a margin of +0.77.
- **GEVS:** those bars are the one item short at the orbital qualification level (margin −0.24). The report lists that as a deviation rather than hiding it. A flight version would need stronger bars or metal studs.

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
3. **Drop test** (qualification unit only). Drop it from 30 cm onto a carpeted floor, once on each face. Look for cracks at the column and boss roots, and check every snap, latch and stud still holds.
4. **Shake test.** Tape a phone running an accelerometer app to the frame and shake it by hand, or mount it on a speaker playing a tone sweep. Look for the frequency where it rattles and compare it with the report's estimates.
5. **Thermal check.** An hour in a fridge, then an hour in a warm room. Re-measure and run the function test.

## Checks

`cargo run -p cubesat-cad --release -- check` runs every check. The same command with `build hardware/rev-c` also rewrites the files here. The checks cover:

- **CDS geometry:** size, protrusions, rails, rail ends, rail contact, recessed faces, venting, mass and centre of mass
- **Launch loads:** margins of safety for every load-carrying feature, and the lowest frequency of each part
- **Fit:** parts don't overlap, every snap peg passes through its hole, and every plate hole lines up with its column
- **Uno:** pushes down past its latches, every hook reaches over it, every key locks its latch, and nothing hits the board, its solder joints or any of its 16 parts
- **Nano:** sits 1.62 mm into each V-groove, is held in all six directions, can be lowered in with the clamp off, and has room for its USB plug, clear of the frame rails
- **USB:** both plugs pass through the −X cover's port
- **Latching:** latch, key and peg strain against PLA, about 7 N down per latch, and how far a latch can open with its key in
- **Quarter-turn studs:** each passes its bar going in, locks clear and is held when pulled; washer strain, detent, a hold-down of 21 N against 20 N for the payload at 20 g, and bar stress
- **Printing:** each part is one closed solid, fits the P1S, and has no overhang that needs support

`cargo test -p cubesat-cad --release` also breaks the design 37 ways on purpose and confirms each check catches its fault.

## Where it comes from

Rev C is new geometry. It borrows ideas, not files:

- The CDS and CubeSat 101 for the requirements.
- The [AMSAT CubeSatSim](https://cubesatsim.org) frame by Alan Johnston, KU2Y, for the split into end plates and sides, and the snap pegs, board mounts and quarter-turn fasteners from this project's [AMSAT kit](../amsat-cubesatsim/README.md).
- Arduino's own board CAD (CC BY-SA 4.0) for the Uno R3 and Nano 33 BLE Sense Rev2 outlines and parts, as in the AMSAT kit's [reference](../amsat-cubesatsim/reference/README.md).
- The [Teachers in Space](https://www.tis.org/) frame's Multiplate, for carrying an Arduino on a plate.

It's licensed like the rest of the project: MIT or Apache 2.0.
