# Outdoor ground station: Uno R3, Nano 33 BLE Sense and a 3000 mAh battery

The [classroom kit](../amsat-cubesatsim/README.md)'s payload, sealed in a box for outdoors. The box has the same columns and keyed snap pins inside, so the **same payload plate** snaps in, with the Uno R3 and the bare Nano 33 BLE Sense Rev2 on it exactly as in the kit. The 3000 mAh battery lies under the plate. A **1/4"-20 camera thread** in the middle of the back takes a tripod. The rules behind it are in the [design guide](../DESIGN-GUIDE.md).

**Status: design checks only.** Nothing in this folder has been printed, and nothing here is a water test. Don't trust the seal until the box has been tested: see "Test it" below.

## How it goes together

- **Plate:** the kit's payload plate snaps onto the box's columns the same way it snaps onto the AMSAT frame: one way round, three clicks. Two pins are tight, on opposite corners.
- **Battery:** the largest cell MakerFocus's listing allows (67 × 36 × 12 mm plus 2 mm each way) lies in a walled pocket on a deck under the plate. The plate sits 0.5 mm above it, so once the plate is on the cell can't move, and there are no extra parts to hold it. The wire leaves the pocket's top end and comes up through the notch in the plate's top edge.
- **Seal:** a gasket sits in a groove round the rim and is squeezed 27% when the lid is down (20–30% seals without crushing it). Print the gasket in TPU 95A, or use 2.5 mm silicone cord, about 380 mm.
- **Lid:** eight M3 screws into heat-set inserts in the rim, all outside the gasket. The lid's holes are narrow at the bottom, so each screw cuts through once and then stays in the lid: no loose screws once it's built. The CubeSat Classroom mark and lettering are raised on the lid.
- **Ports** are in the wall that faces down on a tripod, so rain runs off them: an M12 × 1.5 cable gland for a USB cable (3–6.5 mm), and an M12 × 1.5 breathable membrane vent, which stops condensation and pressure changes from pumping water past the seal.
- **Tripod:** a 1/4"-20 UNC thread printed straight into a sealed boss in the middle of the back, 9 mm deep. It stops 2 mm short of the inside, so it can't let water in. Tilt the tripod head so the lid faces out and the ports face down.

## What to print

| File | Plate | What's on it |
| --- | --- | --- |
| **[p1s/outdoor-test-coupons.3mf](p1s/outdoor-test-coupons.3mf)** (print first) | Test coupons | The tripod-thread coupon, and the snap pin with its fit ladder |
| **[p1s/outdoor-station.3mf](p1s/outdoor-station.3mf)** | 1. Box | The box (169 cm³) |
| | 2. Lid and payload plate | The lid (53 cm³) and the kit's payload plate |
| | 3. Gasket | The gasket, in TPU 95A (2.5 cm³) |

- **Material:** ASA or PETG for the box, lid and plate. PLA softens in summer sun. The fit ladder was tuned in PLA, so print the pin coupon again in your outdoor material and check which tab fits best.
- **Supports:** none. The box prints on its back. The rim's overhang has a 45° slope under it, and the deck under the battery is bridged between ribs 10 mm apart.
- **Tripod-thread coupon:** two sockets in a block as thick as the box's boss. The plain one is the box's thread. The one with a dot is 0.15 mm looser, for printers that print holes small. Screw your tripod's screw into each. If the plain one is too tight and the dotted one is right, tell us: the box's thread is one number (`thread_clear`), and the files can be regenerated for it.

Every part is also in [stl/](stl/), in print orientation.

## Buy

| Part | Notes |
| --- | --- |
| MakerFocus flat 3.7 V 3000 mAh LiPo, JST PH 2.0 | Protected cell. Check the plug's polarity before connecting it. |
| LiPo charger with a 5 V boost output, such as Adafruit's PowerBoost 1000C | The Uno and Nano need 5 V; the cell gives 3.0–4.2 V. It fits beside the battery under the plate, in a space about 26 × 60 × 24 mm. It has no printed mount yet: hold it with foam tape. |
| M12 × 1.5 cable gland, for 3–6.5 mm cable | Its hole is 12.3 mm, on a flat part of the wall. |
| M12 × 1.5 breathable membrane vent | Also called a protective vent. |
| 8 × M3 heat-set inserts (4.0 mm hole, up to 6 mm long) and 8 × M3 × 10 mm socket-head screws | A mentor sets the inserts with a soldering iron. |
| TPU 95A filament, or 2.5 mm silicone cord | For the gasket. |
| A short right-angle Micro-USB cable, or jumper wires to the Nano's VIN | A straight plug sticks out too far for the box. |

## Assemble

1. A mentor sets the eight heat-set inserts in the rim and checks the battery (see the [battery safety quest](../../quest/m1/battery-safety.md)).
2. Screw in the cable gland and the vent, from outside.
3. Put the battery in its pocket, wire towards the top end, and the boost board beside it. Plug the battery into the boost board, and plug the boost board's USB cable in.
4. Run the cable up through the plate's notch and snap the plate onto the columns (one way round, three clicks). Fit the Uno and the Nano as in the kit.
5. Press the gasket into its groove, all the way round, without stretching it.
6. Put the lid on and tighten the eight screws a little at a time, going round in a cross pattern, until the lid sits down evenly on the gasket.

## Test it

Before it goes outside, test the box empty: close it with a dry paper towel inside, then spray it all over with a garden hose for five minutes, or hold it under water in a bucket for a minute. Open it and check the towel. Write down the result as a hardware bench test. A wet towel means the gasket, the screws or a port need work: tell us.

## How it fits

| Check | Value |
| --- | --- |
| Box | 118 × 118 mm across the rim, 98 mm across the walls, 80 mm tall with the lid. Walls and back 3 mm. Inside, a 92 mm square with 10 mm corner chamfers: the plate has at least 1 mm all round. |
| Plate | On the columns 27 mm above the back: the deck (12.5 mm), the largest cell (14 mm) and 0.5 mm clearance |
| Gasket | 2.6 mm square in a 3.0 × 1.9 mm groove: squeezed 27% |
| Screws | 9.5 mm out from the inside edge, 1.5 mm of plastic between each insert and the gasket groove |
| Tripod thread | 1/4"-20 UNC, cut 0.3 mm larger on the diameter for printing, 9 mm long, 2 mm of plastic sealing it from the inside |
| Ports | M12 gland and vent at 48 mm up the wall that faces down, 25 mm either side of the middle |

The checks in [crates/amsat-kit/src/outdoor_checks.rs](../../crates/amsat-kit/src/outdoor_checks.rs) fail if:

- the walls or back are under 2.4 mm, the gasket is squeezed outside 20–30%, the groove is too close to an edge, or a screw sits inside the seal or breaks out of the rim
- the plate doesn't fit the box with 1 mm all round, or doesn't snap onto its columns the same way as in the kit
- the battery doesn't fit its pocket, can slide about, or isn't held down by the plate
- the boards or their parts hit the box or stand too tall for the lid
- a port runs onto a corner or the slope under the rim, its nut hits anything inside, or two ports are too close
- the tripod thread breaks into the box, or is too short to hold

The self-test (`cargo run --release -p amsat-kit -- self-test`) breaks the outdoor cases fourteen ways and checks that each one is caught. These are design checks, not measurements.

## Regenerate

From the repository root, `cargo run --release -p amsat-kit` builds all three cases. Box sizes, ports and the thread are in `Outdoor` in [outdoor.rs](../../crates/amsat-kit/src/outdoor.rs).
