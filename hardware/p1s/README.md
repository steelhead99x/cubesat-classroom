# Bambu P1S Rev A bench case

Follow the [P1S case test quests](../../quest/README.md#bambu-p1s-case-test-track) to validate the slice, print the parts, record fit, and retest improvements.

Use the **Bambu Lab P1S 0.4 mm nozzle** preset, the actual build plate and filament preset, and **0.20 mm Standard** as a starting process. Start with PLA. The [P1S specification](https://cdn1.bambulab.com/documentation/quick-start-59b0cefdc0fc4/P1S/English%20version-Quick%20Start%20Guide%20for%20P1S.pdf) lists a 256 × 256 × 256 mm build volume and support for PLA and ASA.

Each 3MF is a millimetre-scale model centered on its own plate. Import one at a time. Keep its exported orientation, check the dimensions, and slice locally with your profile. The package does not contain G-code, filament temperatures, or a printer connection.

| Print order | Model | STL fallback | Orientation and first check |
| --- | --- | --- | --- |
| 1 | [M3 fit coupon](01-m3-fit-coupon.3mf) | [STL](../stl/m3-fit-coupon.stl) | 48 × 14 × 10 mm. Flat on the bed. Starting at the notch, holes are 2.6, 2.8, 3.0, 3.2, 3.5, and 3.8 mm. Test the intended screw by hand. |
| 2 | [Case shell](02-case-shell.3mf) | [STL](../stl/bench-shell.stl) | 100 × 100 × 109.5 mm. Closed floor down, opening up. Inspect the four 2.8 mm blind pilots and the open cable slot in the layer preview. |
| 3 | [Camera lid](03-camera-lid.3mf) | [STL](../stl/camera-lid.stl) | 100 × 100 × 6 mm including tabs. Exterior face down, tabs up. Check four 3.5 mm clearance holes and the 18 mm camera opening. |

Start with supports disabled. In Bambu Studio, inspect layer 1, the pilot holes, and each tab, then review any overhang warnings before printing. For ASA repeat the coupon before committing to the full shell, use the filament supplier’s profile and follow its ventilation guidance.

Measure the assembled case against its nominal **100 × 100 × 113.5 mm** envelope. The lid should seat without force. Check for corner lift, cable clearance, and camera sightline. Check screw engagement against the 10 mm blind pilot depth. Record filament, nozzle, layer height, plate, and any measured changes so another crew can repeat the print.

This is a bench enclosure for test fitting. The [Rev B rail-fit kit](../README.md#rev-b-rail-fit-set) is a separate experiment. Neither has completed flight qualification.

Regenerate after rebuilding the original CAD meshes by running `python hardware/p1s/prepare.py` with the [CAD dependencies](../requirements.txt).
