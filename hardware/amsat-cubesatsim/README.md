# AMSAT CubeSatSim frame with a printed Arduino payload

A real 1U CubeSat frame (100 × 100 × 113.5 mm) that holds an **Arduino Uno R3** and an **Arduino Nano 33 BLE Sense**. Every part is 3D printed: no screws, nuts or metal standoffs.

The frame is the [AMSAT CubeSat Simulator](https://cubesatsim.org) v2.0 frame, unchanged except for one piece. The payload plate, strap and snap columns are new parts designed for this kit.

![Payload plate layout](payload-plate-layout.png)

## Credit and license

- **Frame:** AMSAT CubeSatSim v2.0 by Alan Johnston, KU2Y, AMSAT VP Educational Relations. Source: [alanbjohnston/CubeSatSim `hardware/frame/v2.0`](https://github.com/alanbjohnston/CubeSatSim/tree/master/hardware/frame/v2.0), commit `1f6a01c4` (2024-10-07), with the [Onshape CAD](https://cad.onshape.com/documents/ba47819ff100597297d1d967/w/f19409ec7cd5459afb429cb9/e/a0e0cc37dee6fb879c4bcc2b) linked from there.
- **License:** the CubeSatSim repository is GPL-3.0, so everything in this folder is GPL-3.0 too ([LICENSE](LICENSE)). The rest of CubeSat Classroom keeps its MIT or Apache-2.0 terms.
- **Changes:** `bottom-with-columns` is the AMSAT top/bottom piece with four standoff columns added where the AMSAT board stack normally screws in. The files in [upstream/](upstream/) are AMSAT's, byte for byte (SHA-256 below).
- This is an independent build. It isn't made or endorsed by AMSAT.

## What to print

Use the P1S profile for your nozzle (0.20mm Standard for 0.4 mm, 0.30mm Standard for 0.6 mm), PLA, and **supports off**. Each 3MF opens with every part already in its print orientation.

| Plate | Parts | 0.6 mm nozzle |
| --- | --- | --- |
| [01-snap-test](p1s/01-snap-test.3mf) | Short snap column and a tab with the snap hole | 5 min, 2 g |
| [02-sides](p1s/02-sides.3mf) | AMSAT side and side with camera hole | 1 h 40 min, 58 g |
| [03-top-and-bottom](p1s/03-top-and-bottom.3mf) | AMSAT top (unchanged) and bottom with snap columns | 1 h 18 min, 48 g |
| [04-payload-plate](p1s/04-payload-plate.3mf) | Payload plate and Nano strap | 50 min, 33 g |

The whole kit is about 3 h 50 min and 140 g of PLA. The times are slicer estimates.

**Print the snap test first.** Push the peg into the tab. It should click in with a firm push and not fall out. If it's too tight or too loose, change `SNAP_HOLE` in [generate.py](generate.py) by 0.1 mm and regenerate (see below). The payload plate and the Nano strap use the same snap. The first test print on a P1S with a 0.6 mm nozzle snapped well at the default size.

## Assemble

1. **Uno R3:** set the board on the four pins with the USB port toward the clip side between the jacks. Press down until both edge clips click over the board.
2. **Nano 33 BLE Sense** (with headers soldered, pins down): lay it on the two rails with the USB end away from the end stop, then push it against the stop. Press the strap over the two posts until it clicks. The strap's feet rest on the header pin tips, clear of the parts in the middle.
3. **Plate into the frame:** turn the plate over and press it onto the four columns of the bottom piece until all four snaps click.
4. **Frame:** snap the two sides onto the bottom, then the top, the same way as AMSAT's frame. AMSAT notes the frame holds together on its own. Skip their optional screws to keep it 100% printed.

The Nano's pins hang 20 mm above the plate, so there's room for female jumper wires underneath. The Uno's headers face up for wiring from above.

## How it fits

| Check | Value |
| --- | --- |
| Plate | 88 × 88 × 3 mm with 8 mm corner chamfers (AMSAT's boards are 90.5 mm) |
| Plate position | 15 mm inside the bottom face, below AMSAT's 12 mm snap tabs |
| Snap peg | 5.0 mm shank, 5.7 mm barb, 1.4 mm split, into a 5.2 mm hole |
| Uno R3 | 4 locating pins (2.9 mm) on 10 mm posts, 2 edge clips (1.8 mm arm, 0.7 mm lip) |
| Nano 33 BLE Sense | PCB on rails at 20 mm, end stop, strap with feet on the pin rows |
| Above the plate | 30 mm to the top of the strap, leaving room for a battery and sensors |
| Bending strain | Uno clips 1.4%, snap pegs 0.8%, under a 2% limit for PLA |

[generate.py](generate.py) assembles the parts in frame coordinates and fails if:

- the plate holes don't line up with the columns, or the plate hits the AMSAT piece or its snap tabs
- a plate feature passes through either board, sits under the Uno's solder joints, or blocks a snap barb
- a Nano rail hits the header pins, or the strap presses into the Nano
- a clip or peg would bend more than PLA can take over and over

Every part was sliced with a P1S-style 0.6 mm profile to confirm it prints without supports.

Not yet checked on a real print: the Uno clip grip and the Nano strap pressure. The Nano's PCB thickness and header height vary by batch, so expect to tune `PCB_NANO` or `STRAP_FOOT_ABOVE_PCB`. Report what you measure with the [P1S case quests](../../quest/README.md#bambu-p1s-case-test-track).

## Regenerate

```sh
python3 -m venv /tmp/cubesat-cad
/tmp/cubesat-cad/bin/pip install -r hardware/requirements.txt
/tmp/cubesat-cad/bin/python hardware/amsat-cubesatsim/generate.py
/tmp/cubesat-cad/bin/python hardware/amsat-cubesatsim/prepare.py
```

To redraw the layout image, install `matplotlib` too and run `python layout.py` from this folder.

Upstream checksums (SHA-256):

```
34c9737cd7a26b5df9b34df2227ffc6aecfdbc02061d5a5a1234c1b2ddb30784  upstream/cubesatsim_frame_side.v2.0.stl
df1d094d1110f4a5d77c2b580937f8cb6df850ceacb43847701be3b9036afa04  upstream/cubesatsim_frame_side_camera.v2.0.stl
8f6ffc3367aca32192878f5a98caab0e6d6c74b85a1ea21323669c52d9d81d21  upstream/cubesatsim_frame_top_bottom.v2.0.stl
```
