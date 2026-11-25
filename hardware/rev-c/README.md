# Rev C: a 1U CubeSat frame built to the CubeSat spec, in Rust

Rev C is a fully printed 1U frame (100 × 100 × 113.5 mm) that holds an **Arduino Uno R3** and an **Arduino Nano 33 BLE Sense Rev2**. Its CAD, its checks and its print files all come from one Rust crate, [`crates/cubesat-cad`](../../crates/cubesat-cad/src/lib.rs). The board firmware is Rust too: [`firmware/nano33-cubesat`](../../firmware/nano33-cubesat/README.md).

Every dimension is checked against the [CubeSat Design Specification (CDS) Rev 14.1](https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf) from Cal Poly ([cubesat.org](https://www.cubesat.org/)), using the processes in NASA's [CubeSat 101](https://www.nasa.gov/wp-content/uploads/2017/03/nasa_csli_cubesat_101_508.pdf). The full result is in the [compliance report](cds-report.md).

## What it is

| Part | What it does |
| --- | --- |
| Side panel ×2 (identical) | One face plus two corner rails: 9 mm square, every long edge rounded to 1 mm. An end wall at each end takes the snap pegs. |
| −Z end plate | Inserted first. Four columns carry the payload plate. |
| +Z end plate | Opening over the payload. |
| Payload plate | Uno R3 on four pins with two edge clips; Nano on two rails with an end stop and a strap. |
| Nano strap | Snaps over the Nano. |

There are no screws, nuts or metal standoffs. Each end plate clicks onto both side panels with four snap pegs, the same peg as the AMSAT kit's, whose snap test printed well on a P1S. The ±X faces stay open: the Uno's USB port faces −X, and the D2 jumper and the Nano's USB are reachable from there too.

## How it meets the spec

| CDS Rev 14.1 | Rev C |
| --- | --- |
| §2.2.1: 100 × 100 × 113.5 mm, origin at the centre | Same, in CAD. Measure the print. |
| §2.2.3: protrusions ≤ 6.5 mm | Nothing outside the rail envelope, boards included |
| §2.2.5, §2.2.7: rails ≥ 8.5 mm, edges rounded ≥ 1 mm | 9.0 mm rails, 1.0 mm radius |
| §2.2.8: rail ends give ≥ 6.5 × 6.5 mm contact | 7.0 × 7.0 mm flat; end plates and panels stop 1.5 mm short of the ends |
| §2.2.9: ≥ 75% rail contact | Continuous rails: 100% |
| §2.2.10, §2.2.11: mass ≤ 2.00 kg, centre of mass within ±2 cm | About 210 g solid; centre of mass about 9 mm towards −Z |
| §2.2.6, §2.2.12, §2.2.13, §2.1.7: aluminium, anodised, low outgassing | **Deviation.** Printed PLA, for the classroom |
| §2.3.2, §2.3.5: deployment switch, RBF pin | **Deviation.** The D2 jumper inhibits the firmware; there is no rail-end switch |
| §2.4.5: 45 min radio silence after deployment | The firmware's `Timing::Cds` setting |

A printed classroom frame can't meet the material and electrical requirements. The report lists them as deviations rather than hiding them, because a flight version would need a different material or part, or the launch provider's approval.

## Print

Bambu Lab P1S, PLA, supports off. Use the profile for your nozzle (0.30mm Standard for 0.6 mm).

| Plate | Parts | 0.6 mm nozzle |
| --- | --- | --- |
| [00-snap-test](p1s/00-snap-test.3mf) | Peg on a boss, and a standing wall with the teardrop hole | 7 min, 2 g |
| [01-side-panels](p1s/01-side-panels.3mf) | Both side panels | 1 h 2 min, 66 g |
| [02-end-plates](p1s/02-end-plates.3mf) | −Z and +Z end plates | 1 h 7 min, 51 g |
| [03-payload](p1s/03-payload.3mf) | Payload plate and Nano strap | 48 min, 31 g |

That's about 3 hours and 150 g for the whole frame. **Print the snap test first:** the side panels' holes print sideways as teardrops, unlike the AMSAT kit's round holes, so check that the peg still clicks in and holds.

[assembly.stl](assembly.stl) shows every part in place.

## Assemble

1. Uno onto the four pins, USB port towards the clip between the jacks. Press until both clips click.
2. Nano (headers soldered, pins down) onto the rails, USB end away from the end stop. Push it against the stop and snap the strap over the two posts.
3. Payload plate onto the −Z end plate's four columns until they click.
4. Snap both side panels onto the −Z end plate, rails outward, then the +Z end plate on top.

## Checks

`cargo run -p cubesat-cad --release -- check` runs every check. The same command with `build hardware/rev-c` also rewrites the files here. The checks cover:

- **CDS geometry:** size, protrusions, rails, rail ends, rail contact, mass and centre of mass
- **Fit:** parts don't overlap, every snap peg passes through its hole, and the boards clear the posts, clips, rails, strap, solder joints, header pins and frame
- **Latching:** barb overlap, clip reach, and clip and peg bending strain against PLA
- **Printing:** each part is one closed solid, fits the P1S, and has no overhang that needs support

`cargo test -p cubesat-cad --release` also breaks the design 16 ways on purpose and confirms each check catches its fault.

## Where it comes from

Rev C is new geometry. It borrows ideas, not files:

- The CDS and CubeSat 101 for the requirements.
- The [AMSAT CubeSatSim](https://cubesatsim.org) frame by Alan Johnston, KU2Y, for the split into end plates and sides, and the payload plate and snap pegs from this project's [AMSAT kit](../amsat-cubesatsim/README.md).
- The [Teachers in Space](https://www.tis.org/) frame's Multiplate, for carrying an Arduino on a plate.

It's licensed like the rest of the project: MIT or Apache 2.0.
