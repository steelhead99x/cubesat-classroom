"""Fully printed payload kit for the AMSAT CubeSatSim v2.0 frame. Dimensions in mm.

Builds, from the unmodified AMSAT frame in upstream/:
  bottom-with-columns  AMSAT top/bottom piece plus four snap-peg standoff columns,
                       placed where the AMSAT board stack screws to the frame
  payload-plate        holds an Arduino Uno R3 and an Arduino Nano 33 BLE Sense
  nano-strap           snaps over the Nano to hold it down
  snap-test            a short column and a holed tab to test the snap fit first

No screws, nuts or metal standoffs. Everything is in print orientation, flat
side down, and needs no supports.

This folder is a derivative of the AMSAT CubeSatSim frame by Alan Johnston,
KU2Y, and is licensed GPL-3.0 like the original. See LICENSE and README.md.
"""

from pathlib import Path

import manifold3d as cad
import numpy as np
import trimesh

HERE = Path(__file__).resolve().parent
FACETS = 48
EPS = 0.05

# AMSAT end piece (frame coordinates: face exterior at z=+3, face inner side at z=0,
# frame interior towards -z). Stack mounting holes measured from the v2.0 STL.
STACK_HOLES = [(-32.99, 32.99), (-32.71, -25.01), (29.0, -38.99), (29.0, 31.51)]
FACE_INNER_Z = 0.0
SNAP_TABS_REACH = 12.0          # AMSAT snap tabs extend this far below the face

# Snap peg, shared by the columns, the strap posts and the test piece.
PEG_SHANK = 5.0
PEG_BARB = 5.7
PEG_TIP = 4.0
PEG_LIP = 0.8
PEG_LEAD = 1.0
PEG_SLOT = 1.4
SNAP_HOLE = 5.2
SLOT_INTO_BODY = 4.0

COLUMN_OD = 7.0
COLUMN_LENGTH = 15.0            # face to plate; clears the 12 mm snap tabs

PLATE = 88.0                    # AMSAT boards are 90.5 mm square
PLATE_CHAMFER = 8.0
PLATE_T = 3.0

# Arduino Uno R3: 68.58 x 53.34 mm, holes from the lower-left corner, USB on the left edge.
UNO = (68.58, 53.34)
UNO_HOLES = [(13.97, 2.54), (15.24, 50.80), (66.04, 7.62), (66.04, 35.56)]
UNO_ORIGIN = (-UNO[0] / 2, PLATE / 2 - 1.0 - UNO[1])
UNO_POST_H = 6.0
UNO_PIN = 2.9
PCB_UNO = 1.6
CLIP_T = 2.4
CLIP_W = 8.0
CLIP_GAP = 0.3
CLIP_LIP = 1.0

# Arduino Nano 33 BLE Sense: 43.18 x 17.78 mm PCB, pin rows 15.24 mm apart.
NANO = (43.18, 17.78)
NANO_CENTER = (-2.0, -28.0)     # USB end towards -x
NANO_ROW = 15.24
NANO_RAIL_H = 20.0              # room for header pins and female jumper wires below
PCB_NANO = 1.6
STRAP_ARCH = 3.0                # clearance over the parts between the pin rows
STRAP_FOOT_ABOVE_PCB = 1.0      # feet bear on the soldered header pin tips
STRAP_T = 3.0
STRAP_W = 7.0
STRAP_POST = 12.0               # posts at +/- this from the Nano centre line


def cyl(d, h, x=0.0, y=0.0, z=0.0, d2=None):
    return cad.Manifold.cylinder(h, d / 2, (d2 if d2 is not None else d) / 2,
                                 circular_segments=FACETS).translate((x, y, z))


def box(x0, y0, z0, x1, y1, z1):
    return cad.Manifold.cube((x1 - x0, y1 - y0, z1 - z0)).translate((x0, y0, z0))


def peg(z_base, through, slot_axis="x"):
    """Split snap peg standing on z_base: shank through `through` mm, then a barb."""
    shank = cyl(PEG_SHANK, through + 0.2 + EPS, z=z_base - EPS)
    z = z_base + through + 0.2
    barb = (cyl(PEG_BARB, PEG_LIP + EPS, z=z - EPS)
            + cyl(PEG_BARB, PEG_LEAD + EPS, z=z + PEG_LIP - EPS, d2=PEG_TIP))
    top = z + PEG_LIP + PEG_LEAD
    span = PEG_BARB + 2
    if slot_axis == "x":
        slot = box(-span / 2, -PEG_SLOT / 2, z_base - SLOT_INTO_BODY, span / 2, PEG_SLOT / 2, top + 1)
    else:
        slot = box(-PEG_SLOT / 2, -span / 2, z_base - SLOT_INTO_BODY, PEG_SLOT / 2, span / 2, top + 1)
    return shank + barb, slot


def chamfered_square(size, chamfer, height):
    h = size / 2
    c = chamfer
    pts = [(-h + c, -h), (h - c, -h), (h, -h + c), (h, h - c),
           (h - c, h), (-h + c, h), (-h, h - c), (-h, -h + c)]
    return cad.CrossSection([pts]).extrude(height)


def load(name):
    mesh = trimesh.load_mesh(HERE / "upstream" / name)
    return cad.Manifold(cad.Mesh(vert_properties=np.asarray(mesh.vertices, np.float32),
                                 tri_verts=np.asarray(mesh.faces, np.uint32)))


def bottom_with_columns():
    """AMSAT end piece with columns growing from the face's inner side (frame coordinates)."""
    part = load("cubesatsim_frame_top_bottom.v2.0.stl")
    slots = []
    for x, y in STACK_HOLES:
        z0 = FACE_INNER_Z - COLUMN_LENGTH
        column = cyl(COLUMN_OD, COLUMN_LENGTH + 1.5, x, y, z0)   # fills the 3.2 mm hole
        # Peg points away from the face (towards -z): build upward, then mirror in z.
        body, slot = peg(0.0, PLATE_T)
        body = body.mirror((0, 0, 1)).translate((x, y, z0))
        slot = slot.mirror((0, 0, 1)).translate((x, y, z0))
        part += column + body
        slots.append(slot)
    for slot in slots:
        part -= slot
    return part


def plate_holes():
    """Stack holes seen from the board side of the plate (plate printed board side up)."""
    return [(x, -y) for x, y in STACK_HOLES]


def uno_in_plate(ux, uy):
    return UNO_ORIGIN[0] + ux, UNO_ORIGIN[1] + uy


def payload_plate():
    top = PLATE_T
    part = chamfered_square(PLATE, PLATE_CHAMFER, PLATE_T)
    for x, y in plate_holes():
        part -= cyl(SNAP_HOLE, PLATE_T + 2, x, y, -1)

    # Uno R3: four posts with locating pins, and two edge clips that hook over the board.
    pcb_bottom = top + UNO_POST_H
    for ux, uy in UNO_HOLES:
        x, y = uno_in_plate(ux, uy)
        part += cyl(7.0, UNO_POST_H - 2.0 + EPS, x, y, top - EPS)       # wide base
        part += cyl(5.0, 2.0 + EPS, x, y, pcb_bottom - 2.0 - EPS)       # clears solder joints
        part += cyl(UNO_PIN, PCB_UNO + 0.9, x, y, pcb_bottom - EPS)
        part += cyl(UNO_PIN, 0.5 + EPS, x, y, pcb_bottom + PCB_UNO + 0.9 - EPS, d2=UNO_PIN - 0.8)
    catch = pcb_bottom + PCB_UNO + 0.1
    left_x = UNO_ORIGIN[0] - CLIP_GAP
    right_x = UNO_ORIGIN[0] + UNO[0] + CLIP_GAP
    clips = [(left_x - CLIP_T, left_x, uno_in_plate(0, 22.0)[1], +1),     # between DC jack and USB
             (right_x, right_x + CLIP_T, uno_in_plate(0, UNO[1] / 2)[1], -1)]
    for x0, x1, yc, inward in clips:
        y0, y1 = yc - CLIP_W / 2, yc + CLIP_W / 2
        part += box(x0, y0, top - EPS, x1, y1, catch + CLIP_LIP + 1.5)
        lip_x0, lip_x1 = (x1 - EPS, x1 + CLIP_LIP) if inward > 0 else (x0 - CLIP_LIP, x0 + EPS)
        lip = box(lip_x0, y0, catch, lip_x1, y1, catch + CLIP_LIP + EPS)
        # Lead-in ramp above the lip so the board pushes the clip outward as it goes in.
        ramp_pts = ([(x1, 0), (x1 + CLIP_LIP, 0), (x1, 1.5)] if inward > 0
                    else [(x0 - CLIP_LIP, 0), (x0, 0), (x0, 1.5)])
        ramp = cad.CrossSection([ramp_pts]).extrude(CLIP_W).rotate((90, 0, 0)).translate((0, y1, catch + CLIP_LIP - EPS))
        part += lip + ramp

    # Nano 33 BLE Sense: two rails between the pin rows, an end stop, and two strap posts.
    nx, ny = NANO_CENTER
    rail_top = top + NANO_RAIL_H
    for y0, y1 in ((2.0, 5.0), (-5.0, -2.0)):      # inside the pin rows at +/-7.62
        part += box(nx - 15, ny + y0, top - EPS, nx + 15, ny + y1, rail_top)
    stop_x = nx + NANO[0] / 2 + 0.3
    part += box(stop_x, ny - 3, top - EPS, stop_x + 2.5, ny + 3, rail_top + 1.0)
    pcb_top = rail_top + PCB_NANO
    strap_bottom = pcb_top + STRAP_ARCH
    for side in (-1, 1):
        x, y = nx, ny + side * STRAP_POST
        part += cyl(8.0, 5.0 + EPS, x, y, top - EPS)
        part += cyl(PEG_SHANK, strap_bottom - top - 5.0 + EPS, x, y, top + 5.0 - EPS)
        body, slot = peg(strap_bottom, STRAP_T, slot_axis="x")
        part = part + body.translate((x, y, 0)) - slot.translate((x, y, 0))
    return part


def nano_strap():
    """Printed feet up. Feet press on the header pin tips along both pin rows."""
    length = 2 * STRAP_POST + 10
    part = box(-STRAP_W / 2, -length / 2, 0, STRAP_W / 2, length / 2, STRAP_T)
    feet_h = STRAP_ARCH - STRAP_FOOT_ABOVE_PCB
    for side in (-1, 1):
        part -= cyl(SNAP_HOLE, STRAP_T + 2, 0, side * STRAP_POST, -1)
        y0 = side * (NANO_ROW / 2 - 1.0)
        y1 = side * (NANO[1] / 2 + 0.5)
        part += box(-STRAP_W / 2, min(y0, y1), STRAP_T - EPS, STRAP_W / 2, max(y0, y1), STRAP_T + feet_h)
    return part


def snap_test():
    """A short column with a peg, and a tab with the snap hole. Print both together."""
    base = box(-8, -8, 0, 8, 8, 2.0)
    column = cyl(COLUMN_OD, 6.0 + EPS, z=2.0 - EPS)
    body, slot = peg(8.0, PLATE_T)
    tab = box(-10, -8, 0, 10, 8, PLATE_T) - cyl(SNAP_HOLE, PLATE_T + 2, 0, 0, -1)
    return base + column + body - slot, tab


def check(name, solid):
    if solid.status() != cad.Error.NoError:
        raise ValueError(f"{name}: CAD error {solid.status()}")
    data = solid.to_mesh()
    mesh = trimesh.Trimesh(data.vert_properties[:, :3], data.tri_verts)
    if len(solid.decompose()) != 1:
        raise ValueError(f"{name}: expected one solid, got {len(solid.decompose())}")
    if not (mesh.is_watertight and mesh.is_winding_consistent and mesh.is_volume):
        raise ValueError(f"{name}: mesh is not a closed solid")
    return mesh


def verify(bottom, plate):
    """Seat the plate on the columns in frame coordinates and check for collisions."""
    seated = plate.rotate((180, 0, 0)).translate((0, 0, FACE_INNER_Z - COLUMN_LENGTH))
    columns_only = bottom ^ box(-46, -46, -COLUMN_LENGTH - 10, 46, 46, FACE_INNER_Z - SNAP_TABS_REACH - 0.5)
    overlap = (seated ^ (bottom - columns_only)).volume()
    if overlap > 1e-3:
        raise ValueError(f"payload plate collides with the AMSAT end piece ({overlap:.3f} mm3)")
    lo = np.array(seated.bounding_box()[:3])
    if lo[2] > FACE_INNER_Z - SNAP_TABS_REACH:
        raise ValueError("payload plate does not clear the AMSAT snap tabs")
    nx, ny = NANO_CENTER
    for x, y in plate_holes():
        ux0, uy0 = UNO_ORIGIN
        for ux, uy in UNO_HOLES:
            if np.hypot(ux0 + ux - x, uy0 + uy - y) < 7.0:
                raise ValueError("an Uno post sits on a snap column")
        if abs(x - nx) < NANO[0] / 2 + 3 and abs(y - ny) < STRAP_POST + 4.5:
            # Column barbs sit 1.5 mm above the plate; only the Nano rails and posts matter.
            if abs(x - nx) < 15 + 3.5 and abs(y - ny) < 5 + 3.5:
                raise ValueError("a snap column sits under the Nano rails")


def export(name, solid):
    mesh = check(name, solid)
    mesh.apply_translation((0, 0, -mesh.bounds[0, 2]))
    folder = HERE / "stl"
    folder.mkdir(exist_ok=True)
    mesh.export(folder / f"{name}.stl")
    size = [round(float(v), 1) for v in mesh.extents]
    print(f"{name}: {size} mm, watertight, {len(mesh.faces)} triangles")
    return mesh


if __name__ == "__main__":
    bottom = bottom_with_columns()
    plate = payload_plate()
    verify(bottom, plate)
    # The end piece prints face down, so its columns point up.
    export("bottom-with-columns", bottom.rotate((180, 0, 0)))
    export("payload-plate", plate)
    export("nano-strap", nano_strap())
    peg_part, tab_part = snap_test()
    export("snap-test-peg", peg_part)
    export("snap-test-tab", tab_part)
