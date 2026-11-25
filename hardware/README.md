# Hardware

Printed 1U CubeSat frames (100 × 100 × 113.5 mm) for a Bambu Lab P1S or a similar printer.

| Design | What it is | Start here |
| --- | --- | --- |
| **Rev C (Rust)** | New frame built to the CubeSat Design Specification: 8.5 mm-plus rounded rails, recessed ends, 100% printed, Uno R3 and Nano 33 BLE Sense. CAD, checks and firmware all in Rust. | [Build guide](rev-c/README.md) |
| **AMSAT CubeSatSim frame + Arduino payload** | AMSAT's v2.0 frame with a printed plate for an Uno R3 and a Nano 33 BLE Sense. 100% printed, no supports. | [Build guide](amsat-cubesatsim/README.md) |
| Teachers in Space frame | The frame TIS uses in its Arduino CubeSat workshops. Files come from TIS. | [Links and comparison](tis/README.md) |
| Rev A bench box | Closed test box and the M3 screw-fit coupon | [Print guide](p1s/README.md) |
| Rev B rail-fit set | Printed frame with four corner rails and two end plates. A step toward a printed flight structure. Needs supports. | [Below](#rev-b-rail-fit-set) |

If you're new, build the AMSAT frame with the Arduino payload.

Every structural part, cover, mount, retainer and fit tool the project designs must have a version that prints on a P1S or similar printer. Electronics, wiring, cells, sensors and RF parts are chosen separately.

## Rev B rail-fit set

![Rev B printed rail-fit frame and end plates](rev-b-cad-preview.png)

One printed frame with four continuous 9 mm corner rails, plus two end plates that drop onto slip-fit locators without screws (0.35 mm clearance per side). Nothing holds the plates in yet, so hold them while handling. Positive retainers and payload mounts still need design work. See the [dimensioned drawing](rev-b-drawing.svg).

| Part | Size (mm) | How to print |
| --- | --- | --- |
| [Rail corner coupon](stl/rev-b-rail-corner-coupon.stl) | 26 × 26 × 20 | Print first, rail end down. Check the corner width and the recessed walls. |
| [Rail frame](stl/rev-b-rail-frame.stl) | 100 × 100 × 113.5 | Upright as exported. Turn on build-plate supports: the side walls start 5.4 mm above the bed. Add a brim if the rail feet lift. |
| [Base plate](stl/rev-b-base-plate.stl), [camera plate](stl/rev-b-camera-plate.stl) | 93.3 × 93.3 × 4.4 | Flat face down, locator tabs up. Both should seat without force and stay inside the rails. |

Start with PLA, then repeat the coupon in ASA using the filament maker's profile. Measure the rail width and straightness at both ends and the middle, and note warping and support scars. The [Rev B quests](../quest/README.md#m0--start-now) list what to record. Flight evidence is tracked in the [ISS qualification review](../docs/iss-deployment-review.md#printed-structure-qualification).

## Regenerate the files

The Rev A and Rev B meshes come from [generate.py](generate.py), which also checks that each part is watertight, has the expected size, and that the lids and plates don't collide with their frames. [p1s/prepare.py](p1s/prepare.py) packs the Rev A parts into centered P1S 3MF plates, and [drawing.py](drawing.py) redraws the Rev B drawing.

```sh
python3 -m venv /tmp/cubesat-cad
/tmp/cubesat-cad/bin/pip install -r hardware/requirements.txt
/tmp/cubesat-cad/bin/python hardware/generate.py
/tmp/cubesat-cad/bin/python hardware/p1s/prepare.py
/tmp/cubesat-cad/bin/python hardware/drawing.py
```

The AMSAT kit has its own [generate.py](amsat-cubesatsim/generate.py) and [prepare.py](amsat-cubesatsim/prepare.py). CI runs all of them and fails if the committed files differ from the output.

Printer reference: the P1S has a 256 × 256 × 256 mm build volume and prints PLA and ASA ([Bambu Lab P1S quick start](https://cdn1.bambulab.com/documentation/quick-start-59b0cefdc0fc4/P1S/English%20version-Quick%20Start%20Guide%20for%20P1S.pdf)). For the limits of printed parts in flight, see [NASA CubeSat 101](https://science.nasa.gov/wp-content/uploads/2023/06/nasa_csli_cubesat_101_508.pdf).
