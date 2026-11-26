# Reference board models

The mounts in this folder are designed around Arduino's own CAD for both boards. These files let you check a fit in any slicer or CAD tool. Don't print them.

| File | What it is | Source |
| --- | --- | --- |
| `nano-33-ble-sense-rev2-ABX00069.stl` | Arduino Nano 33 BLE Sense Rev2, bare board (no headers), with every part's 3D model | Arduino's Altium design, `NANO33BLE_SENSE.PcbDoc`, from the [ABX00069 CAD files](https://docs.arduino.cc/hardware/nano-33-ble-sense-rev2) |
| `uno-r3-envelope.stl` | Arduino Uno R3: the board plus a box for each part, at its catalogue height | Arduino's Eagle design, `UNO-TH_Rev3e.brd`, from the [A000066 CAD files](https://docs.arduino.cc/hardware/uno-rev3). Arduino ships no 3D models for the Uno, so the part heights in `UNO_PARTS` in [boards.rs](../../../crates/amsat-kit/src/boards.rs) are rounded-up catalogue values. |

Coordinates are in mm. The origin is the board's lower-left corner, on the bottom face of the PCB, with parts up (+z). On the Nano, the USB end is at x = 0.

What the CAD shows:

- **Nano 33 BLE Sense Rev2:** the board outline is 43.18 × 17.78 mm (0.7 × 1.7 in), and the layer stack-up adds up to 1.59 mm thick. The Micro-USB sticks out 1.5 mm past the end, which is why Arduino's spec pages give 45 × 18 mm. The underside has only pads (solder jumpers SJ1–SJ4, SWD test pads), so it sits flat. The tallest parts on top are the USB (2.8 mm), the NINA-B306 module (2.3 mm) and the reset button (2.1 mm). Both long edges have a strip about 2.8 mm wide with nothing but the castellated pads, and that's where the V-grooves grip. The optional pin headers (ABX00070) are left out.
- **Uno R3:** 68.58 × 53.34 mm. The mounting holes are at (13.97, 2.54), (15.24, 50.80), (66.04, 7.62) and (66.04, 35.56), the same as before.

## License

Arduino publishes these designs under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The two STL files are derived from them: converted to meshes, with the Nano's header models removed and the Uno reduced to part outlines. They keep that license, with credit to Arduino SA. Arduino® is a trademark of Arduino SA. This project isn't made or endorsed by Arduino.

## Rebuild

You need KiCad 7 or later and `cadquery-ocp`.

1. Get `ABX00069-cad-files.zip` from Arduino's docs page, or from [arduino/docs-content](https://github.com/arduino/docs-content) at `content/hardware/nano/boards/nano-33-ble-sense-rev2/downloads/`.
2. Import `NANO33BLE_SENSE.PcbDoc` with KiCad's Altium importer, for example `pcbnew.IO_MGR.Load(pcbnew.IO_MGR.ALTIUM_DESIGNER, path)`, and save it as a `.kicad_pcb`. The embedded STEP models are extracted to `ALTIUM_EMBEDDED_MODELS/`.
3. Delete the two `Header _Fem_1X8.step` model lines (the optional headers). Then run `KIPRJMOD=$PWD kicad-cli pcb export step` next to an empty `.kicad_pro`.
4. Mesh the STEP with OCP (`BRepMesh_IncrementalMesh`, 0.05 mm) and move the board's corner to the origin.
