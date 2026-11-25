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
TOLERANCE = 1e-6                # mm of floating-point noise to ignore; far below print resolution
PLA_STRAIN_LIMIT = 0.02         # keep repeated flexing well under PLA's ~3% yield
MIN_SNAP_OVERLAP = 0.2          # barb diameter minus hole diameter that still holds

# Board geometry comes from Arduino's own CAD files (CC BY-SA 4.0), see reference/README.md.
# Arduino Uno R3 (A000066, Eagle UNO-TH_Rev3e.brd): 68.58 x 53.34 mm, 1.6 mm PCB.
# Coordinates from the lower-left corner, USB on the left edge.
UNO = (68.58, 53.34)
UNO_HOLES = [(66.04, 35.56), (66.04, 7.62), (15.24, 50.80), (13.97, 2.54)]   # hole 0..3
UNO_ORIGIN = (-35.66, 32.0 - UNO[1])  # USB-B overhang ends 1.9 mm inside the plate edge
UNO_POST_H = 10.0
UNO_PIN = 2.9
UNO_PIN_ABOVE = -0.3            # pin tip below the board's top face, so clamps press flat
UNO_POST_TOP = 4.4              # the hole at (15.24, 50.8) sits beside the header pins
PCB_UNO = 1.6
UNO_RING = (1.6, 3.2)           # copper keep-out ring around each 3.2 mm hole (radius)
MIN_RING_CONTACT = 3.0          # mm^2 of each ring a clamp must press on
# Top-side parts (x0, y0, x1, y1, height above the board) from the Eagle layout. Heights are
# the parts' catalogue heights, rounded up.
UNO_PARTS = {
    "DC jack": (-1.87, 1.30, 13.68, 12.35, 11.0),
    "USB-B": (-6.47, 30.49, 9.53, 45.71, 11.0),
    "digital header 8-13": (17.45, 49.45, 43.00, 52.14, 8.5),
    "digital header 0-7": (44.37, 49.45, 64.84, 52.14, 8.5),
    "power header": (26.59, 1.19, 47.06, 3.88, 8.5),
    "analog header": (49.45, 1.19, 64.84, 3.88, 8.5),
    "ICSP (328P)": (62.28, 24.05, 67.51, 31.82, 8.5),
    "ICSP (16U2)": (14.40, 43.61, 22.17, 48.84, 8.5),
    "JP2": (16.94, 38.53, 22.17, 43.76, 8.5),
    "ATmega328P in socket": (27.73, 11.50, 64.97, 21.26, 8.0),
    "capacitor PC1": (22.22, 5.24, 28.82, 13.04, 7.7),
    "capacitor PC2": (15.11, 5.24, 21.71, 13.04, 7.7),
    "reset button": (1.35, 46.45, 11.35, 52.35, 5.0),
    "crystal Y1": (13.13, 23.80, 24.71, 28.52, 4.5),
    "diode D1": (18.19, 0.84, 24.99, 4.75, 2.5),
    "regulator U1": (3.54, 14.02, 11.96, 20.77, 1.8),
}

# Arduino Nano 33 BLE Sense Rev2, bare board (ABX00069, Altium NANO33BLE_SENSE.PcbDoc):
# 43.18 x 17.78 mm, 1.59 mm PCB, flat underside (pads only), castellated edges. The Micro-USB
# overhangs the end by 1.5 mm. The often-quoted 45 x 18 mm includes that overhang.
NANO_L, NANO_W, NANO_T = 43.18, 17.78, 1.59
# Top-side parts (along, across, along, across, height): measured from the CAD's 3D models.
NANO_PARTS = {
    "Micro-USB": (-1.48, 4.50, 4.13, 13.30, 2.8),
    "NINA-B306 module": (28.05, 3.90, 43.05, 13.90, 2.32),
    "reset button": (9.65, 6.65, 12.55, 11.15, 2.05),
    "sensors and passives": (0.0, 2.80, NANO_L, 14.90, 1.4),
}
NANO_USB_CENTRE = 8.90          # across the board
NANO_USB_ABOVE = 1.4            # receptacle centre above the top face
# The Nano stands on its long edge like a card in a card cage, parts facing +y (into the
# CubeSat), USB end towards -x. NX0 = USB end, NY0 = back face, NZ0 = lower edge. NANO_PARTS
# keeps the CAD's own axes; nano_card() turns them into this position.
NX0, NY0, NZ0 = -14.0, -40.8, 4.5
NANO_END_PLAY = 0.5             # along the board, between the end stops (0.2 printed a little tight)
V_MOUTH, V_APEX, V_DEPTH = 2.4, 1.4, 2.0     # V-groove widths and depth that grip each edge
V_WALL = 2.6                    # groove wall beside the board
MIN_SEAT = 1.0                  # how far each Nano edge must sit inside its groove
EDGE_CLEAR = 2.8                # the board's edge strips are free of parts this far in
NANO_STOP_REACH = 3.0           # end stops cover the corners only, clear of the USB plug
PLUG = (28.0, 7.0, 10.5)        # Micro-USB plug envelope: length, thickness, width
MIN_PLUG_ROOM = 20.0            # from the receptacle to the plate edge, for the plug

# Printed screws: a round single-start thread made by twisting an off-centre circle.
SCREW_R = 2.75                  # core circle
SCREW_E = 0.5                   # offset: thread crest 6.5 mm, root 4.5 mm diameter
PITCH = 2.5                     # 8 layers per turn at 0.3 mm
SCREW_CLEAR = 0.25              # radial clearance in the pillar
LOCK_INTERFERENCE = 0.1         # bottom of each bore grips the screw like a nylon locknut
LOCK_LEN = 3.0
ENGAGE = 10.0                   # thread length in the pillar
HEAD_D, HEAD_H = 10.0, 4.0
MIN_BORE_WALL = 1.2
PILLAR_OD = 10.0
CLAMP_T = 4.0
HEEL_GAP = 0.3                  # pillar tops sit this far below the clamp, so the screw
                                # pulls the clamp down onto the board, like a strap clamp
EDGE_GAP = 0.5                  # board edge to pillar

UNO_TOP_Z = PLATE_T + UNO_POST_H + PCB_UNO
NANO_SEAT = V_DEPTH * (V_MOUTH - NANO_T) / (V_MOUTH - V_APEX)   # edge depth in each groove
BASE_MOUTH_Z = NZ0 + NANO_SEAT
CAP_MOUTH_Z = NZ0 + NANO_W - NANO_SEAT
NANO_MID_Y = NY0 + NANO_T / 2


def uno_xy(ux, uy):
    return UNO_ORIGIN[0] + ux, UNO_ORIGIN[1] + uy


def uno_clamps():
    """Clamp footprints: (pillar xy, rectangles pressing the hole ring). Pillars sit just off
    the board edge; each tongue reaches the ring between neighbouring parts."""
    ux0, uy0 = UNO_ORIGIN
    right = ux0 + UNO[0] + EDGE_GAP + PILLAR_OD / 2
    top = uy0 + UNO[1] + EDGE_GAP + PILLAR_OD / 2
    bottom = uy0 - EDGE_GAP - PILLAR_OD / 2
    h = [uno_xy(*p) for p in UNO_HOLES]
    return {
        "uno-clamp-0": ((right, h[0][1]), (uno_xy(63.0, 33.5), uno_xy(UNO[0], 37.6))),
        "uno-clamp-1": ((right, h[1][1]), (uno_xy(63.0, 5.5), uno_xy(UNO[0], 9.7))),
        "uno-clamp-2": ((uno_xy(14.8, 0)[0], top), (uno_xy(12.6, 49.2), uno_xy(17.0, UNO[1]))),
        "hole-3 leg": ((uno_xy(16.0, 0)[0], bottom), (uno_xy(14.0, 0.3), uno_xy(18.0, 4.3))),
    }


FAR_PILLAR = (34.5, -36.5)


def pillar_positions():
    c = uno_clamps()
    return [c[k][0] for k in ("uno-clamp-0", "uno-clamp-1", "uno-clamp-2", "hole-3 leg")] + [FAR_PILLAR]


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


def barb_rooms(z_top=None):
    """Space each snap barb needs where it comes up through the plate."""
    h = 0.2 + PEG_LIP + PEG_LEAD + 1.0
    return [cyl(PEG_BARB + 2.0, h if z_top is None else z_top, x, y, PLATE_T) for x, y in plate_holes()]


# ---------- printed screws ----------

def thread(r, z0, z1, x=0.0, y=0.0):
    """Round single-start right-hand thread between z0 and z1, in phase at any z so that bore
    sections of different sizes line up."""
    h = z1 - z0
    start = 360.0 * z0 / PITCH
    section = cad.CrossSection.circle(r, FACETS).translate((SCREW_E, 0)).rotate(start)
    return section.extrude(h, max(2, int(h / 0.15)), 360.0 * h / PITCH).translate((x, y, z0))


def screw_bore(x, y, z_top):
    """Threaded hole down from a pillar top: clearance first, then the locking band."""
    z_bottom = z_top - ENGAGE - 1.0
    lock = z_bottom + 1.0 + LOCK_LEN
    return (thread(SCREW_R + SCREW_CLEAR, lock, z_top + 1.0, x, y)
            + thread(SCREW_R - LOCK_INTERFERENCE, z_bottom, lock + EPS, x, y)
            + cyl(2 * (SCREW_R - SCREW_E) - 0.4, ENGAGE + 1.0, x, y, z_bottom))


def screw():
    """Head down, as printed. Slot in the head for a flat screwdriver, hex flats for pliers or a spanner."""
    head = cad.CrossSection.circle(HEAD_D / 2, 6).extrude(HEAD_H) ^ cyl(HEAD_D * 1.08, HEAD_H)
    head -= box(-HEAD_D, -0.8, -1, HEAD_D, 0.8, 1.2)
    shank_len = CLAMP_T + ENGAGE - 0.5
    shank = thread(SCREW_R, HEAD_H - EPS, HEAD_H + shank_len)
    tip = cyl(2 * (SCREW_R + SCREW_E) + 2, 1.2, z=HEAD_H + shank_len - 1.2) - cyl(
        2 * (SCREW_R + SCREW_E) + 0.2, 1.2 + EPS, z=HEAD_H + shank_len - 1.2 - EPS / 2, d2=2 * (SCREW_R - SCREW_E))
    return head + shank - tip


def clearance_hole(x, y, z0, h):
    return cyl(2 * (SCREW_R + SCREW_E) + 0.6, h, x, y, z0)


# ---------- payload plate ----------

def nano_card():
    """The Nano in place, as a solid: PCB plus envelopes of its top-side parts."""
    def place(lx0, ly0, lx1, ly1, lz0, lz1):
        # Turning the board so its parts face +y with the USB towards -x puts the CAD's
        # across-the-board axis upside down: z = NZ0 + NANO_W - across.
        return box(NX0 + lx0, NY0 + lz0, NZ0 + NANO_W - ly1, NX0 + lx1, NY0 + lz1, NZ0 + NANO_W - ly0)
    body = place(0, 0, NANO_L, NANO_W, 0, NANO_T)
    for x0, y0, x1, y1, h in NANO_PARTS.values():
        body += place(x0, y0, x1, y1, NANO_T - EPS, NANO_T + h)
    return body


def nano_plug():
    usb_x = NX0 + NANO_PARTS["Micro-USB"][0]
    y = NY0 + NANO_T + NANO_USB_ABOVE
    z = NZ0 + NANO_W - NANO_USB_CENTRE
    return box(usb_x - PLUG[0], y - PLUG[1] / 2, z - PLUG[2] / 2, usb_x, y + PLUG[1] / 2, z + PLUG[2] / 2)


def _groove(pts, x0, x1, mouth_z):
    """Extrude a (y, z) profile along +x."""
    section = cad.CrossSection([[(y, z) for y, z in pts]])
    solid = section.extrude(x1 - x0)                    # profile in XY, length along Z
    solid = solid.rotate((0, 90, 0))                    # Z -> X; X -> -Z
    solid = solid.rotate((90, 0, 0))                    # Y -> Z; Z -> -Y
    return solid.translate((x0, NANO_MID_Y, mouth_z))


def payload_plate():
    top = PLATE_T
    part = chamfered_square(PLATE, PLATE_CHAMFER, PLATE_T)

    # Uno R3: four posts with short locating pins that stop below the board's top face.
    pcb_bottom = top + UNO_POST_H
    pin_h = PCB_UNO + UNO_PIN_ABOVE
    for ux, uy in UNO_HOLES:
        x, y = uno_xy(ux, uy)
        part += cyl(7.0, UNO_POST_H - 2.5 + EPS, x, y, top - EPS)
        part += cyl(UNO_POST_TOP, 2.5 + EPS, x, y, pcb_bottom - 2.5 - EPS)
        part += cyl(UNO_PIN, pin_h - 0.4 + 2 * EPS, x, y, pcb_bottom - EPS)
        part += cyl(UNO_PIN, 0.4, x, y, pcb_bottom + pin_h - 0.4, d2=UNO_PIN - 0.6)

    # Pillars for the five printed screws.
    bores = []
    for name, ((x, y), _) in uno_clamps().items():
        z = (CAP_MOUTH_Z if name == "hole-3 leg" else UNO_TOP_Z) - HEEL_GAP
        part += cyl(PILLAR_OD, z - top + EPS, x, y, top - EPS)
        bores.append(screw_bore(x, y, z))
    x, y = FAR_PILLAR
    z = CAP_MOUTH_Z - HEEL_GAP
    part += cyl(PILLAR_OD, z - top + EPS, x, y, top - EPS)
    bores.append(screw_bore(x, y, z))

    # Nano card cage, lower half: V-groove rail, USB-end corner stop, far-end wall.
    y0, y1 = NY0 - V_WALL, NY0 + NANO_T + V_WALL
    x_end = NX0 + NANO_L + NANO_END_PLAY
    part += box(NX0 - 2.0, y0, top - EPS, x_end + 1.0, y1, BASE_MOUTH_Z)
    part += box(NX0 - 2.0, NY0 - 1.0, BASE_MOUTH_Z - EPS, NX0, NY0 + NANO_T + 1.0, NZ0 + NANO_STOP_REACH)
    part += box(x_end, y0, top - EPS, x_end + 2.0, y1, NZ0 + NANO_W - 2.0)
    part -= _groove([(-V_MOUTH / 2, 0), (-V_APEX / 2, -V_DEPTH), (V_APEX / 2, -V_DEPTH), (V_MOUTH / 2, 0),
                     (V_MOUTH / 2, 1), (-V_MOUTH / 2, 1)], NX0 - EPS, x_end, BASE_MOUTH_Z)

    for hole in plate_holes():
        part -= cyl(SNAP_HOLE, PLATE_T + 2, hole[0], hole[1], -1)
    for room in barb_rooms():
        part -= room                      # pillars and rails near a barb get a pocket above it
    for bore in bores:
        part -= bore
    return part


def uno_clamp(name):
    """Flat bar: rests on its pillar and presses one hole ring. Printed flat."""
    (px, py), ((x0, y0), (x1, y1)) = uno_clamps()[name]
    z0 = UNO_TOP_Z
    if name == "uno-clamp-2":   # narrow between the reset button and the header, wider off the board
        bar = box(x0, y0, z0, x1, py, z0 + CLAMP_T)
    else:
        bar = box(x0, y0, z0, px, y1, z0 + CLAMP_T)
    bar += cyl(HEAD_D, CLAMP_T, px, py, z0)
    return bar - clearance_hole(px, py, z0 - 1, CLAMP_T + 2)


def lower_clamp():
    """Cap rail over the Nano's upper edge, joined to the leg that presses Uno hole 3.
    One flat top face, so it prints upside down without supports."""
    zs = CAP_MOUTH_Z
    zt = zs + V_DEPTH + 2.0
    (hx, hy), ((lx0, ly0), (lx1, ly1)) = uno_clamps()["hole-3 leg"]
    fx, fy = FAR_PILLAR
    rail = box(NX0 - 2.0, NY0 - V_WALL, zs, fx, NY0 + NANO_T + V_WALL, zt)
    rail += cyl(HEAD_D, zt - zs, fx, fy, zs) + box(fx - 3.0, fy - 3.0, zs, fx + 0.1, NY0 + NANO_T, zt)
    arm = box(hx - 4.0, NY0, zs, hx + 4.5, hy, zt) + cyl(HEAD_D, zt - zs, hx, hy, zs)
    arm += box(lx0, hy, zs, lx1, ly1, zt)
    leg = box(lx0, ly0, UNO_TOP_Z, lx1, ly1, zs + EPS)
    stop = box(NX0 - 2.0, NY0 - 1.0, NZ0 + NANO_W - NANO_STOP_REACH, NX0, NY0 + NANO_T + 1.0, zs + EPS)
    part = rail + arm + leg + stop
    part -= _groove([(-V_MOUTH / 2, -1), (V_MOUTH / 2, -1), (V_MOUTH / 2, 0),
                     (V_APEX / 2, V_DEPTH), (-V_APEX / 2, V_DEPTH), (-V_MOUTH / 2, 0)],
                    NX0 - EPS, NX0 + NANO_L + NANO_END_PLAY + EPS, zs)
    for x, y in ((hx, hy), (fx, fy)):
        part -= clearance_hole(x, y, zs - 1, zt - zs + 2)
    return part


# Fit plate: the payload plate's mounts on a light isogrid panel, styled like a spacecraft
# structure for photos and school displays. Raised lettering from label.json (see label.py).
FIT_BASE_T = 2.4                # rib height; the board mounts sit on top of it
FIT_SKIN = 0.6                  # floor of each pocket, two 0.3 mm layers
FIT_PAD = 3.0                   # solid deck around every board feature
FIT_RIM = 5.0
FIT_BEVEL = 0.8                 # 45 degree bevel on the outer top edge
ISO_PITCH = 16.0                # isogrid triangle side
ISO_RIB = 2.0                   # about three lines with a 0.6 mm nozzle
ISO_NODE = 5.0                  # boss where three ribs meet
ISO_MIN_POCKET = 30.0           # mm^2: smaller slivers next to the deck are filled in
ISO_ROUND = 1.0                 # pocket corner radius
LETTER_H = 0.6                  # raised lettering: swap filament at this layer for two colours
NAMEPLATE = (-24.5, -12.0, 24.5, 20.5)
AXES_PLATE = (1.0, -35.0, 28.0, -20.0)
AXES_ORIGIN = (4.5, -31.8)
PLATE_BORDER = 1.0


def _label():
    import json
    out = cad.CrossSection()
    for line in json.loads((HERE / "label.json").read_text()):
        out += cad.CrossSection(line["polygons"], cad.FillRule.EvenOdd)
    return out


def _arrow(x0, y0, x1, y1, w=1.2, head=3.0):
    """Flat arrow from (x0, y0) to (x1, y1)."""
    import math
    length = math.hypot(x1 - x0, y1 - y0)
    shaft = cad.CrossSection.square((length - head, w)).translate((0, -w / 2))
    tip = cad.CrossSection([[(length - head, -head * 0.55), (length, 0), (length - head, head * 0.55)]])
    return (shaft + tip).rotate(math.degrees(math.atan2(y1 - y0, x1 - x0))).translate((x0, y0))


def _rect(x0, y0, x1, y1):
    return cad.CrossSection.square((x1 - x0, y1 - y0)).translate((x0, y0))


def fit_plate():
    """Quick fit test and display piece: the payload plate's board features, unchanged and in
    the same places, on an isogrid panel with a nameplate and a spacecraft axis marker."""
    import math
    top = PLATE_T
    features = payload_plate() - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    outline = chamfered_square(PLATE, PLATE_CHAMFER, 1).slice(0.5)

    # Panel with a bevelled top edge (the outline is convex, so a hull makes the bevel).
    inner = outline.offset(-FIT_BEVEL, cad.JoinType.Miter)
    panel = cad.Manifold.hull(outline.extrude(FIT_BASE_T - FIT_BEVEL)
                              + inner.extrude(EPS).translate((0, 0, FIT_BASE_T - EPS)))

    # Solid deck: rim, pads around the mounts, nameplate and axis plate.
    solid = outline - outline.offset(-FIT_RIM, cad.JoinType.Miter)
    solid += features.slice(top + 0.1).offset(FIT_PAD, cad.JoinType.Round, circular_segments=FACETS)
    for r in (NAMEPLATE, AXES_PLATE):
        solid += _rect(*r).offset(1.5, cad.JoinType.Round, circular_segments=FACETS).offset(-1.5, cad.JoinType.Round, circular_segments=FACETS)

    # Isogrid ribs in three directions, with nodes where they cross.
    ribs = cad.CrossSection()
    h = ISO_PITCH * math.sqrt(3) / 2
    for angle in (0, 60, 120):
        for k in range(-12, 13):
            line = cad.CrossSection.square((3 * PLATE, ISO_RIB)).translate((-1.5 * PLATE, k * h - ISO_RIB / 2))
            ribs += line.rotate(angle)
    for j in range(-8, 9):
        for i in range(-8, 9):
            x, y = i * ISO_PITCH + (j % 2) * ISO_PITCH / 2, j * h
            ribs += cad.CrossSection.circle(ISO_NODE / 2, 24).translate((x, y))
    pockets = (outline - solid - ribs).offset(-ISO_ROUND, cad.JoinType.Round, circular_segments=16)
    pockets = pockets.offset(ISO_ROUND, cad.JoinType.Round, circular_segments=16).simplify(0.02)
    pockets = cad.CrossSection([p for p in pockets.to_polygons() if abs(cad.CrossSection([p]).area()) > ISO_MIN_POCKET])
    panel -= pockets.extrude(FIT_BASE_T).translate((0, 0, FIT_SKIN))

    # Raised nameplate border, lettering and axis arrows.
    raised = _label()
    for r in (NAMEPLATE, AXES_PLATE):
        frame = _rect(*r).offset(1.5, cad.JoinType.Round, circular_segments=FACETS).offset(-1.5, cad.JoinType.Round, circular_segments=FACETS)
        raised += frame - frame.offset(-PLATE_BORDER, cad.JoinType.Miter)
    ox, oy = AXES_ORIGIN
    raised += _arrow(ox, oy, ox + 13.0, oy) + _arrow(ox, oy, ox, oy + 6.5)
    raised += cad.CrossSection.circle(1.4, 24).translate((ox, oy))
    panel += raised.simplify(0.01).extrude(LETTER_H + EPS).translate((0, 0, FIT_BASE_T - EPS))

    return (panel + features.translate((0, 0, FIT_BASE_T - top)))


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


def peg_strain():
    """Bending strain of one peg half as the barb passes through the hole."""
    half = (PEG_SHANK - PEG_SLOT) / 2
    arm = PLATE_T + 0.2 + PEG_LIP + PEG_LEAD + SLOT_INTO_BODY
    return 1.5 * half * (PEG_BARB - SNAP_HOLE) / 2 / arm ** 2


def snap_overlap():
    """How much wider the barb is than the hole it snaps through (diameter)."""
    return PEG_BARB - SNAP_HOLE


def bore_wall():
    return PILLAR_OD / 2 - (SCREW_R + SCREW_E + SCREW_CLEAR)


def at_least(value, minimum):
    """value >= minimum, ignoring floating-point noise: 0.7 - 0.4 is 0.29999999999999993."""
    return value >= minimum - TOLERANCE


def require(ok, message):
    if not ok:
        raise ValueError(message)


def overlap(a, b):
    return (a ^ b).volume()


def uno_parts():
    return {name: box(*uno_xy(x0, y0), UNO_TOP_Z - EPS, *uno_xy(x1, y1), UNO_TOP_Z + h)
            for name, (x0, y0, x1, y1, h) in UNO_PARTS.items()}


def outline_prism(z0, z1):
    return chamfered_square(PLATE, PLATE_CHAMFER, z1 - z0).translate((0, 0, z0))


def verify(bottom, plate, clamps):
    """Assemble the parts in frame coordinates and check fit, clearances and retention."""
    lower = clamps["lower-clamp"]
    bars = [clamps[k] for k in ("uno-clamp-0", "uno-clamp-1", "uno-clamp-2")]

    # Simple design rules first.
    require(UNO_PIN_ABOVE < 0, "the Uno's locating pins must stop below its top face so the clamps press flat")
    require(at_least(NANO_SEAT, MIN_SEAT), f"the Nano's edges sit {NANO_SEAT:.2f} mm into the V-grooves, "
            f"less than {MIN_SEAT} mm")
    lowest = min(y0 for _, y0, _, _, _ in NANO_PARTS.values() if y0 > 0)
    highest = max(y1 for _, _, _, y1, _ in NANO_PARTS.values())
    require(NANO_SEAT + 0.3 < min(lowest, NANO_W - highest),
            "the V-groove walls reach the parts along the Nano's edges")

    # Snap columns.
    require(at_least(snap_overlap(), MIN_SNAP_OVERLAP),
            f"snap barb ({PEG_BARB} mm) overlaps the {SNAP_HOLE} mm hole by {snap_overlap():.2f} mm, "
            f"less than {MIN_SNAP_OVERLAP} mm, so the peg would not latch")
    require(SNAP_HOLE > PEG_SHANK, f"snap hole {SNAP_HOLE} mm does not clear the {PEG_SHANK} mm peg shank")
    require(PEG_SLOT > snap_overlap(), "the peg slot is too narrow for the barb to squeeze through the hole")
    require(peg_strain() < PLA_STRAIN_LIMIT, f"snap peg strain {peg_strain():.1%} is too high for PLA")

    # Printed screws.
    require(at_least(bore_wall(), MIN_BORE_WALL),
            f"pillar wall around the screw is {bore_wall():.2f} mm, less than {MIN_BORE_WALL} mm")
    require(ENGAGE >= 1.5 * 2 * (SCREW_R + SCREW_E), "screw engages less than 1.5 diameters")
    require(0 < LOCK_INTERFERENCE < SCREW_E, "the locking band must be tighter than the screw, but not strip it")
    require(PITCH / 0.3 >= 6, "thread pitch is too fine to print at 0.3 mm layers")

    # Plate on the columns.
    seated = plate.rotate((180, 0, 0)).translate((0, 0, FACE_INNER_Z - COLUMN_LENGTH))
    for (x, y), (px, py) in zip(STACK_HOLES, plate_holes()):
        require(np.allclose((px, -py), (x, y)), "plate holes do not line up with the columns")
    require(overlap(seated, bottom) < 1e-3, "payload plate collides with the AMSAT end piece")
    require(seated.bounding_box()[5] <= FACE_INNER_Z - SNAP_TABS_REACH,
            "payload plate does not clear the AMSAT snap tabs")

    # Board side of the plate, in plate coordinates.
    top = PLATE_T
    features = plate - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    everything = features + lower
    for bar in bars:
        everything += bar
    for room in barb_rooms():
        require(overlap(everything, room) < 1e-3, "a part sits where a snap barb comes through")
    require(overlap(everything, outline_prism(top, 80) ^ everything) >= everything.volume() - 1e-2,
            "a part reaches outside the plate outline, into the frame's corner rails")

    # Uno R3: board, underside, parts, clamps.
    ux, uy = UNO_ORIGIN
    pcb_bottom = top + UNO_POST_H
    uno = box(ux, uy, pcb_bottom, ux + UNO[0], uy + UNO[1], UNO_TOP_Z)
    posts = cad.Manifold()
    for hx, hy in UNO_HOLES:
        uno -= cyl(3.2, PCB_UNO + 2, ux + hx, uy + hy, pcb_bottom - 1)
        posts += cyl(7.0, UNO_POST_H + PCB_UNO + 2, ux + hx, uy + hy, top - 1)
    require(overlap(everything, uno) < 1e-3, "a part passes through the Uno board")
    underside = box(ux, uy, pcb_bottom - 2.0, ux + UNO[0], uy + UNO[1], pcb_bottom) - posts
    require(overlap(everything, underside) < 1e-3, "something under the Uno would hit its solder joints")
    for name, body in uno_parts().items():
        require(overlap(everything, body) < 1e-3, f"a clamp or pillar hits the Uno's {name}")
    above = box(ux - 7, uy, UNO_TOP_Z + 0.01, ux + UNO[0], uy + UNO[1], 80)
    require(overlap(features, above) < 1e-3, "something on the plate blocks lowering the Uno onto its posts")
    contact = box(-PLATE, -PLATE, UNO_TOP_Z, PLATE, PLATE, UNO_TOP_Z + 0.05)
    clamp_feet = (lower + bars[0] + bars[1] + bars[2]) ^ contact
    for i, (hx, hy) in enumerate(UNO_HOLES):
        x, y = uno_xy(hx, hy)
        ring = cyl(2 * UNO_RING[1], 0.05, x, y, UNO_TOP_Z) - cyl(2 * UNO_RING[0], 1, x, y, UNO_TOP_Z - 0.5)
        area = overlap(clamp_feet, ring) / 0.05
        require(area >= MIN_RING_CONTACT, f"Uno hole {i}: the clamp presses {area:.1f} mm^2 of the ring, "
                f"less than {MIN_RING_CONTACT} mm^2")

    # Nano 33 BLE Sense card: no collisions, captured in every direction, room for the plug.
    card = nano_card()
    pcb = box(NX0, NY0 + 0.02, NZ0 + 0.02, NX0 + NANO_L, NY0 + NANO_T - 0.02, NZ0 + NANO_W - 0.02)
    parts = card - box(NX0 - 1, NY0 - 1, NZ0 - 1, NX0 + NANO_L + 1, NY0 + NANO_T, NZ0 + NANO_W + 1)
    require(overlap(everything, parts) < 1e-3, "a groove wall, stop or clamp hits a part on the Nano")
    require(overlap(everything, pcb) < 1e-3, "the Nano's board does not fit between its V-grooves and stops")
    for label, (dx, dy, dz) in {"up": (0, 0, 0.3), "down": (0, 0, -0.3), "towards the Uno": (0, 0.3, 0),
                                "towards the wall": (0, -0.3, 0),
                                "towards the USB end": (-NANO_END_PLAY - 0.2, 0, 0),
                                "towards the far end": (NANO_END_PLAY + 0.2, 0, 0)}.items():
        moved = pcb.translate((dx, dy, dz))
        require(overlap(everything, moved) > 1e-3, f"the Nano could move {label}: nothing holds it")
        for half, solid in (("base", features), ("cap", lower)):
            if dz == 0 and dx == 0:
                require(overlap(solid, moved) > 1e-3, f"the {half} groove does not hold the Nano sideways")
    lift = (box(NX0 + 0.1, NY0 + 0.02, NZ0 + 0.01, NX0 + NANO_L + 0.1, NY0 + NANO_T - 0.02, 90)
            + parts.translate((0.1, 0, 0)) + box(NX0 + 0.1, NY0, NZ0 + EDGE_CLEAR, NX0 + NANO_L, NY0 + NANO_T + 2.8, 90)
            ) - box(-PLATE, -PLATE, -1, PLATE, PLATE, NZ0 + 0.01)
    lift = lift + box(NX0 + 0.1, NY0 + 0.02, NZ0 + NANO_W, NX0 + NANO_L + 0.1, NY0 + NANO_T + 2.8, 90)
    require(overlap(features, lift) < 1e-3, "with the cap off, the Nano can't be lowered into its groove")
    require(overlap(everything, nano_plug()) < 1e-3, "something blocks the Nano's USB plug")
    room = NX0 + NANO_PARTS["Micro-USB"][0] + PLATE / 2
    require(at_least(room, MIN_PLUG_ROOM), f"only {room:.1f} mm from the Nano's USB to the plate edge, "
            f"less than {MIN_PLUG_ROOM} mm for the plug")

    # Every pillar has a threaded hole under its clamp's screw hole.
    for name, ((x, y), _) in uno_clamps().items():
        z = (CAP_MOUTH_Z if name == "hole-3 leg" else UNO_TOP_Z) - HEEL_GAP
        bore = cyl(2 * (SCREW_R + SCREW_E) - 0.2, 4.0, x, y, z - 4.0)
        require(overlap(plate, bore) < bore.volume() * 0.9, f"{name}: the pillar has no threaded hole")


def export(name, solid):
    mesh = check(name, solid)
    mesh.apply_translation((0, 0, -mesh.bounds[0, 2]))
    folder = HERE / "stl"
    folder.mkdir(exist_ok=True)
    mesh.export(folder / f"{name}.stl")
    size = [round(float(v), 1) for v in mesh.extents]
    print(f"{name}: {size} mm, watertight, {len(mesh.faces)} triangles")
    return mesh


def all_clamps():
    clamps = {k: uno_clamp(k) for k in ("uno-clamp-0", "uno-clamp-1", "uno-clamp-2")}
    clamps["lower-clamp"] = lower_clamp()
    return clamps


BROKEN_DESIGNS = {
    "snap hole wider than the barb": {"SNAP_HOLE": 5.8},
    "snap hole leaves too little overlap": {"SNAP_HOLE": 5.6},
    "plate inside the snap tabs": {"COLUMN_LENGTH": 10.0},
    "thin pillar wall around the screw": {"PILLAR_OD": 9.0},
    "Uno pins standing above the board": {"UNO_PIN_ABOVE": 0.5},
    "V-grooves too shallow to grip the Nano": {"V_DEPTH": 0.4},
    "Nano too close to the plate edge for its USB plug": {"NX0": -26.0},
    "Uno moved 2 mm towards the top edge": {"UNO_ORIGIN": (-35.66, -19.34)},
    "Nano parts against the groove wall": {"V_DEPTH": 4.0},
}

# Designs exactly at a limit must pass: the limits are inclusive.
BOUNDARY_DESIGNS = {
    "pillar wall exactly at the 1.2 mm minimum": {"PILLAR_OD": 2 * (SCREW_R + SCREW_E + SCREW_CLEAR + MIN_BORE_WALL)},
}


def _derived():
    """Recompute module values that depend on parameters a test changes."""
    global UNO_TOP_Z, NANO_SEAT, BASE_MOUTH_Z, CAP_MOUTH_Z, NANO_MID_Y
    UNO_TOP_Z = PLATE_T + UNO_POST_H + PCB_UNO
    NANO_SEAT = V_DEPTH * (V_MOUTH - NANO_T) / (V_MOUTH - V_APEX)
    BASE_MOUTH_Z = NZ0 + NANO_SEAT
    CAP_MOUTH_Z = NZ0 + NANO_W - NANO_SEAT
    NANO_MID_Y = NY0 + NANO_T / 2


def _run(change):
    saved = {name: globals()[name] for name in change}
    globals().update(change)
    _derived()
    try:
        verify(bottom_with_columns(), payload_plate(), all_clamps())
    finally:
        globals().update(saved)
        _derived()


def self_test():
    """Check that verify() rejects each broken design and accepts designs exactly at a limit."""
    assert at_least(0.7 - 0.4, 0.3) and not at_least(0.29, 0.3)
    for label, change in BOUNDARY_DESIGNS.items():
        try:
            _run(change)
            print(f"accepted {label}")
        except ValueError as error:
            raise SystemExit(f"verify() rejected a design at the limit, {label}: {error}") from error
    missed = []
    for label, change in BROKEN_DESIGNS.items():
        try:
            _run(change)
            missed.append(label)
        except ValueError as error:
            print(f"rejected {label}: {error}")
    if missed:
        raise SystemExit(f"verify() accepted broken designs: {', '.join(missed)}")


if __name__ == "__main__":
    import sys

    if "--self-test" in sys.argv:
        self_test()
        sys.exit()
    bottom = bottom_with_columns()
    plate = payload_plate()
    clamps = all_clamps()
    verify(bottom, plate, clamps)
    print(f"checks passed: snap overlap {snap_overlap():.2f} mm, peg strain {peg_strain():.1%}, "
          f"pillar wall {bore_wall():.2f} mm, Nano edge in each groove {NANO_SEAT:.2f} mm")
    # The end piece prints face down, so its columns point up.
    export("bottom-with-columns", bottom.rotate((180, 0, 0)))
    export("payload-plate", plate)
    for name in ("uno-clamp-0", "uno-clamp-1", "uno-clamp-2"):
        export(name, clamps[name])
    export("lower-clamp", clamps["lower-clamp"].rotate((180, 0, 0)))      # flat top on the bed
    export("screw", screw())
    fit = export("fit-plate", fit_plate())
    print(f"fit-plate: {fit.volume / 1000:.1f} cm3 against {check('payload-plate', plate).volume / 1000:.1f} cm3 "
          "for the full payload plate")
    peg_part, tab_part = snap_test()
    export("snap-test-peg", peg_part)
    export("snap-test-tab", tab_part)
