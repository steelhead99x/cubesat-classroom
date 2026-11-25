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
KU2Y, and is licensed GPL-3.0 like the original. Modified by CubeSat Classroom
on 2026-11-25: standoff columns added to the AMSAT top/bottom piece, and new
payload parts. See LICENSE and README.md.

Run with --self-test to check that verify() rejects deliberately broken designs.
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
FACE_T = 3.0
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
UNO_POST_H = 10.0               # also sets the clip arm length, see clip_strain()
UNO_PIN = 2.9
UNO_POST_TOP = 4.4              # the hole at (15.24, 50.8) sits beside the header pins
PCB_UNO = 1.6
CLIP_T = 1.8                    # three lines with a 0.6 mm nozzle
CLIP_W = 8.0
CLIP_GAP = 0.3
CLIP_LIP = 0.7
PLA_STRAIN_LIMIT = 0.02         # keep repeated flexing well under PLA's ~3% yield
MIN_SNAP_OVERLAP = 0.2          # barb diameter minus hole diameter that still holds
MIN_CLIP_REACH = 0.3            # how far a clip lip must reach over the Uno's edge
TOLERANCE = 1e-6                # mm of floating-point noise to ignore; far below print resolution

# Arduino Nano 33 BLE Sense: 45 x 18 mm PCB (Arduino's figure; the older 43.18 x 17.78
# is the classic Nano and left too little room), pin rows 15.24 mm apart.
NANO = (45.0, 18.0)
NANO_CENTER = (-2.0, -26.5)     # USB end towards -x
NANO_SIDE_CLEARANCE = 0.8       # per side, as the board drops past the strap posts (0.6 mm nozzle)
JUMPER_HOUSING = 2.54           # female Dupont housing on each header pin
NANO_ROW = 15.24
NANO_RAIL_H = 20.0              # room for header pins and female jumper wires below
PCB_NANO = 1.6
STRAP_ARCH = 3.0                # clearance over the parts between the pin rows
STRAP_FOOT_ABOVE_PCB = 1.0      # feet bear on the soldered header pin tips
STRAP_T = 3.0
STRAP_W = 7.0
STRAP_POST = 13.5               # posts at +/- this from the Nano centre line


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
        column = cyl(COLUMN_OD, COLUMN_LENGTH + FACE_T, x, y, z0)   # fills the 3.2 mm hole flush
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
        part += cyl(7.0, UNO_POST_H - 2.5 + EPS, x, y, top - EPS)       # wide base
        part += cyl(UNO_POST_TOP, 2.5 + EPS, x, y, pcb_bottom - 2.5 - EPS)  # clears solder joints
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


FIT_BASE_T = 2.0                # thinner, open base for the fit plate
FIT_PAD = 3.0                   # solid base around every board feature
FIT_BAR = 5.0                   # width of the ring and the bars joining the pads


def fit_plate():
    """Quick fit test: the payload plate's board features, unchanged and in the same
    places, on a thin open frame. No snap holes. Print it with the Nano strap."""
    top = PLATE_T
    features = payload_plate() - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    pads = features.slice(top + 0.1).offset(FIT_PAD, cad.JoinType.Round, circular_segments=FACETS)
    h = PLATE / 2
    outline = chamfered_square(PLATE, PLATE_CHAMFER, 1).slice(0.5)
    ring = outline - outline.offset(-FIT_BAR, cad.JoinType.Miter)
    nx, ny = NANO_CENTER
    _, uy = uno_in_plate(0, UNO[1] / 2)
    _, lower_posts = uno_in_plate(0, (UNO_HOLES[0][1] + UNO_HOLES[2][1]) / 2)
    b = FIT_BAR / 2
    bars = [cad.CrossSection.square((PLATE, FIT_BAR)).translate((-h, y - b)) for y in (ny, uy, lower_posts)]
    bars.append(cad.CrossSection.square((FIT_BAR, PLATE)).translate((nx - b, -h)))
    base = pads + ring
    for bar in bars:
        base += bar
    base = (base ^ outline).simplify(0.05).extrude(FIT_BASE_T + EPS).translate((0, 0, top - FIT_BASE_T))
    return (base + features).translate((0, 0, FIT_BASE_T - top))


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


def clip_strain():
    """Bending strain of an Uno clip as the board pushes it out by the lip depth."""
    arm = PLATE_T + UNO_POST_H + PCB_UNO + 0.1 - PLATE_T     # plate surface to the lip
    return 1.5 * CLIP_T * CLIP_LIP / arm ** 2


def peg_strain():
    """Bending strain of one peg half as the barb passes through the hole."""
    half = (PEG_SHANK - PEG_SLOT) / 2
    arm = PLATE_T + 0.2 + PEG_LIP + PEG_LEAD + SLOT_INTO_BODY
    return 1.5 * half * (PEG_BARB - SNAP_HOLE) / 2 / arm ** 2


def snap_overlap():
    """How much wider the barb is than the hole it snaps through (diameter)."""
    return PEG_BARB - SNAP_HOLE


def clip_reach():
    """How far each clip lip reaches over the Uno's edge, after the gap to the board."""
    return CLIP_LIP - CLIP_GAP


def at_least(value, minimum):
    """value >= minimum, ignoring floating-point noise: 0.7 - 0.4 is 0.29999999999999993."""
    return value >= minimum - TOLERANCE


def require(ok, message):
    if not ok:
        raise ValueError(message)


def overlap(a, b):
    return (a ^ b).volume()


def verify(bottom, plate, strap):
    """Assemble the parts in frame coordinates and check fit, clearances and strain."""
    # Retention first: a part that no longer latches also shows a low strain.
    require(at_least(snap_overlap(), MIN_SNAP_OVERLAP),
            f"snap barb ({PEG_BARB} mm) overlaps the {SNAP_HOLE} mm hole by {snap_overlap():.2f} mm, "
            f"less than {MIN_SNAP_OVERLAP} mm, so the peg would not latch")
    require(SNAP_HOLE > PEG_SHANK, f"snap hole {SNAP_HOLE} mm does not clear the {PEG_SHANK} mm peg shank")
    require(PEG_SLOT > snap_overlap(), "the peg slot is too narrow for the barb to squeeze through the hole")
    require(at_least(clip_reach(), MIN_CLIP_REACH),
            f"Uno clip lip reaches {clip_reach():.2f} mm over the board edge, less than {MIN_CLIP_REACH} mm, "
            "so the clip would not hold the board")
    require(clip_strain() < PLA_STRAIN_LIMIT, f"Uno clip strain {clip_strain():.1%} is too high for PLA")
    require(peg_strain() < PLA_STRAIN_LIMIT, f"snap peg strain {peg_strain():.1%} is too high for PLA")

    # Plate on the columns: holes line up, nothing collides, plate clears the snap tabs.
    seated = plate.rotate((180, 0, 0)).translate((0, 0, FACE_INNER_Z - COLUMN_LENGTH))
    for (x, y), (px, py) in zip(STACK_HOLES, plate_holes()):
        require(np.allclose((px, -py), (x, y)), "plate holes do not line up with the columns")
    require(overlap(seated, bottom) < 1e-3, "payload plate collides with the AMSAT end piece")
    require(seated.bounding_box()[5] <= FACE_INNER_Z - SNAP_TABS_REACH,
            "payload plate does not clear the AMSAT snap tabs")

    # Board side of the plate, in plate coordinates.
    top = PLATE_T
    features = plate - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    barb_h = 0.2 + PEG_LIP + PEG_LEAD
    for x, y in plate_holes():
        room = cyl(PEG_BARB + 2.0, barb_h + 1.0, x, y, top)
        require(overlap(features, room) < 1e-3, "a plate feature sits where a snap barb comes through")

    ux, uy = UNO_ORIGIN
    pcb_bottom = top + UNO_POST_H
    uno = box(ux, uy, pcb_bottom, ux + UNO[0], uy + UNO[1], pcb_bottom + PCB_UNO)
    posts = cad.Manifold()
    for hx, hy in UNO_HOLES:
        uno -= cyl(3.2, PCB_UNO + 2, ux + hx, uy + hy, pcb_bottom - 1)
        posts += cyl(7.0, UNO_POST_H + PCB_UNO + 2, ux + hx, uy + hy, top - 1)
    require(overlap(features, uno) < 1e-3, "a plate feature passes through the Uno board")
    underside = box(ux, uy, pcb_bottom - 2.0, ux + UNO[0], uy + UNO[1], pcb_bottom) - posts
    require(overlap(features, underside) < 1e-3, "something under the Uno would hit its solder joints")

    nx, ny = NANO_CENTER
    rail_top = top + NANO_RAIL_H
    nano = box(nx - NANO[0] / 2, ny - NANO[1] / 2, rail_top, nx + NANO[0] / 2, ny + NANO[1] / 2, rail_top + PCB_NANO)
    require(overlap(features, nano) < 1e-3, "a plate feature passes through the Nano board")
    for side in (-1, 1):                      # header strips and pins hang below the PCB
        row = ny + side * NANO_ROW / 2
        pins = box(nx - 19.1, row - 1.3, rail_top - 8.5, nx + 19.1, row + 1.3, rail_top)
        require(overlap(features, pins) < 1e-3, "a Nano rail hits the header pins")
        housings = box(nx - 19.1, row - JUMPER_HOUSING / 2, top + 1.0, nx + 19.1, row + JUMPER_HOUSING / 2, rail_top)
        require(overlap(features, housings) < 1e-3, "a plate feature blocks the jumper wires on the Nano's pins")
    # The board is lowered onto the rails from above, past the strap posts and their barbs.
    c = NANO_SIDE_CLEARANCE
    path = box(nx - NANO[0] / 2, ny - NANO[1] / 2 - c, rail_top, nx + NANO[0] / 2, ny + NANO[1] / 2 + c, rail_top + 40)
    require(overlap(features, path) < 1e-3,
            f"the Nano can't drop onto its rails: less than {c} mm per side between it and the strap posts")

    # Strap on its posts, feet down, resting on the pin rows above the Nano.
    strap_bottom = rail_top + PCB_NANO + STRAP_ARCH
    seated_strap = strap.rotate((180, 0, 0)).translate((nx, ny, strap_bottom + STRAP_T))
    require(overlap(seated_strap, plate) < 1e-3, "the strap collides with its posts or the plate")
    require(overlap(seated_strap, nano) < 1e-3, "the strap feet press into the Nano board")


def export(name, solid):
    mesh = check(name, solid)
    mesh.apply_translation((0, 0, -mesh.bounds[0, 2]))
    folder = HERE / "stl"
    folder.mkdir(exist_ok=True)
    mesh.export(folder / f"{name}.stl")
    size = [round(float(v), 1) for v in mesh.extents]
    print(f"{name}: {size} mm, watertight, {len(mesh.faces)} triangles")
    return mesh


BROKEN_DESIGNS = {
    "snap hole wider than the barb": {"SNAP_HOLE": 5.8},
    "snap hole leaves too little overlap": {"SNAP_HOLE": 5.6},
    "clip gap wider than the lip": {"CLIP_GAP": 0.8},
    "plate inside the snap tabs": {"COLUMN_LENGTH": 10.0},
    "stiff Uno clip (2.4 mm arm, 1.0 mm lip, 6 mm posts)": {"CLIP_T": 2.4, "CLIP_LIP": 1.0, "UNO_POST_H": 6.0},
    "Nano rails on the pin rows": {"NANO_ROW": 8.0},
    "strap feet pressing into the Nano": {"STRAP_FOOT_ABOVE_PCB": -0.5},
    "Nano over a snap column": {"NANO_CENTER": (25.0, -28.0)},
    "strap posts too close to drop the Nano in (first print)": {"STRAP_POST": 12.0},
}


# Designs exactly at a limit must pass: the limits are inclusive.
BOUNDARY_DESIGNS = {
    "clip reach exactly at the 0.30 mm minimum (0.7 mm lip, 0.4 mm gap)": {"CLIP_LIP": 0.7, "CLIP_GAP": 0.4},
}


def self_test():
    """Check that verify() rejects each broken design and accepts designs exactly at a limit."""
    assert at_least(0.7 - 0.4, 0.3) and not at_least(0.29, 0.3)
    saved = {name: globals()[name] for change in BOUNDARY_DESIGNS.values() for name in change}
    for label, change in BOUNDARY_DESIGNS.items():
        globals().update(change)
        try:
            verify(bottom_with_columns(), payload_plate(), nano_strap())
            print(f"accepted {label}")
        except ValueError as error:
            raise SystemExit(f"verify() rejected a design at the limit, {label}: {error}") from error
        finally:
            globals().update(saved)
    saved = {name: globals()[name] for change in BROKEN_DESIGNS.values() for name in change}
    missed = []
    for label, change in BROKEN_DESIGNS.items():
        globals().update(change)
        try:
            verify(bottom_with_columns(), payload_plate(), nano_strap())
            missed.append(label)
        except ValueError as error:
            print(f"rejected {label}: {error}")
        finally:
            globals().update(saved)
    if missed:
        raise SystemExit(f"verify() accepted broken designs: {', '.join(missed)}")


if __name__ == "__main__":
    import sys

    if "--self-test" in sys.argv:
        self_test()
        sys.exit()
    bottom = bottom_with_columns()
    plate = payload_plate()
    strap = nano_strap()
    verify(bottom, plate, strap)
    print(f"checks passed: snap overlap {snap_overlap():.2f} mm, clip reach {clip_reach():.2f} mm, "
          f"clip strain {clip_strain():.1%}, peg strain {peg_strain():.1%}")
    # The end piece prints face down, so its columns point up.
    export("bottom-with-columns", bottom.rotate((180, 0, 0)))
    export("payload-plate", plate)
    export("nano-strap", strap)
    fit = export("fit-plate", fit_plate())
    print(f"fit-plate: {fit.volume / 1000:.1f} cm3 against {check('payload-plate', plate).volume / 1000:.1f} cm3 "
          "for the full payload plate")
    peg_part, tab_part = snap_test()
    export("snap-test-peg", peg_part)
    export("snap-test-tab", tab_part)
