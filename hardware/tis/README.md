# Teachers in Space 1U frame

[Teachers in Space](https://www.tis.org/) (TIS) trains teachers to build Arduino CubeSats and fly them on balloons, gliders and rockets. Their 1U frame is the one used in TIS workshops and classrooms.

## Get the files from TIS

- **Print files:** [1U CubeSat Frame 3D Printer Files](https://www.tis.org/s/1U2.zip) on the TIS [CubeSat Parts & Files](https://www.tis.org/partsandfiles) page
- **Printed kit with hardware:** [1U CubeSat Frame & Hardware](https://www.tis.org/store/p/1u-cubesat-frame-hardware), from TIS
- **Arduino sketches and sensor list:** also on [Parts & Files](https://www.tis.org/partsandfiles)
- **Questions:** noah@tis.org

TIS doesn't publish a license for these files, so this repository links to them instead of copying them. If TIS adds an open license, the STLs can be added here with credit.

## How it compares with the AMSAT frame

Measured from the TIS zip (April 2025 upload) and the AMSAT CubeSatSim v2.0 STLs.

| | TIS frame | AMSAT CubeSatSim v2.0 |
| --- | --- | --- |
| Parts | `1U Base` ×2, `1U Side` ×2, `Multiplate` ×2 | top/bottom ×2, side, side with camera hole |
| End piece | 99.5 × 99.5 × 14.75 mm | 100 × 100 × 15 mm |
| Side | 113.5 × 100 × 8.5 mm (flat panel) | 113.5 × 100 × 33.5 mm (U channel) |
| Board plate | `Multiplate`, 82 × 82 × 2 mm, Uno holes and 8 mm wire holes | none, AMSAT's PCBs bolt in directly |
| Plate mounting | Standoffs on a 65.5 mm square pattern (3.7 mm holes) | Four screws into the bottom piece |
| Fasteners | Standoffs, screws and nuts in the kit | Holds by itself; [our version](../amsat-cubesatsim/README.md) is fully printed |
| License | Not stated | GPL-3.0 |

Both frames use the same 1U envelope and the same split into two end pieces and two sides. The TIS `Multiplate` is the model for this project's [printed payload plate](../amsat-cubesatsim/README.md).
