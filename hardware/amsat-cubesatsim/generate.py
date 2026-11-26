"""Fully printed payload kit for the AMSAT CubeSatSim v2.0 frame. Dimensions in mm.

Builds, from the unmodified AMSAT frame in upstream/:
  bottom-with-columns  AMSAT top/bottom piece plus four hollow standoff columns,
                       placed where the AMSAT board stack screws to the frame
  payload-plate        holds an Arduino Uno R3 (snap latches) and a Nano 33 BLE Sense
                       (card cage), with quarter-turn receptacles for the frame
  lower-clamp          caps the Nano; two quarter-turn studs hold it down
  lock-key             locks a Uno latch shut (four)
  qt-stud-plate/-clamp quarter-turn studs; qt-washer is the spring under each head
  fit-plate            the payload plate's mounts on an isogrid display panel
  load-test-*          one plate fastener on its own, to hang weights from

No metal screws, nuts or springs: everything is printed. Every part is in print orientation and
needs no supports.

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

# Quarter-turn fasteners (the classic push-and-turn panel fastener of aircraft and spacecraft;
# a generic design, described here without brand names). A stud with a screwdriver slot has a
# spiral cam slot across its tip. Push it through, past a cross-bar in the receptacle, and turn
# it 90 degrees clockwise: the cam draws it in against a printed spring washer, then drops the
# bar into a detent. To undo it you have to push in and turn back, so vibration can't loosen it.
QT_D = 6.0                      # stud shank
QT_BORE = 6.4                   # hole the stud passes through
QT_BAR_W, QT_BAR_H = 2.0, 1.6   # receptacle cross-bar, across and along the stud
QT_BAR_Z = 0.7                  # bar's near face, below the receptacle's entry face
QT_CLEAR = 0.3                  # slot clearance around the bar
QT_LIFT = 0.55                  # cam draw-in over the first 75 degrees
QT_DET = 0.2                    # detent: extra push needed to turn back out
QT_TIP = 1.6                    # stud material behind the bar when locked
QT_HEAD_D, QT_HEAD_H = 10.0, 2.5
# Spring washer, printed flat: two spiral arms join a hub (under the stud head) to a ring
# standing on three feet. Pushing the hub down bends the arms along their layers.
WASHER_OD, WASHER_ID = 18.0, 6.8
WASHER_T = 1.5                  # arm thickness, five 0.3 mm layers
WASHER_ARM = 1.6                # arm width
WASHER_SLIT = 0.8
WASHER_SWEEP = 150.0            # degrees each arm wraps
WASHER_FOOT = 1.2
WASHER_H = WASHER_T + WASHER_FOOT
NANO_PRELOAD = 0.15             # the Nano's top edge lifts its clamp this much: the washers press it down
PAYLOAD_MASS = 0.10             # kg on the plate: plate, boards, clamp, keys (estimate)
DESIGN_G = 20.0                 # quasi-static design load, a common figure for small-satellite launch

COLUMN_OD = 10.0                # hollow: the plate's studs pass up through them
COLUMN_LENGTH = 15.0            # face to plate; clears the 12 mm snap tabs

PLATE = 88.0                    # AMSAT boards are 90.5 mm square
PLATE_CHAMFER = 8.0
PLATE_T = 3.0
TOLERANCE = 1e-6                # mm of floating-point noise to ignore; far below print resolution
PLA_STRAIN_LIMIT = 0.02         # keep repeated flexing well under PLA's ~3% yield

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

PILLAR_OD = 10.0
CLAMP_T = 4.0                   # Nano clamp thickness
# Uno snap latches with lock keys. Each latch is a cantilever arm beside a board edge with a
# hook over the board. Its hook underside is angled, so the arm's spring presses the board
# down onto its posts. After it clicks in, a printed key drops in behind the arm and clips
# into place: then the arm can't open, like the secondary lock on an aerospace connector.
# Latches sit only where Arduino's layout leaves the board edge clear (see UNO_PARTS).
LATCHES = {                     # name: (board edge, centre along that edge, gusset depth)
    "left": ("left", 21.0, 1.5),        # between the DC jack and the USB-B
    "right": ("right", 16.0, 1.5),      # between hole 1 and the ICSP header
    "top": ("top", 14.5, 1.5),          # between the reset button and the 8-13 header
    "bottom": ("bottom", 6.0, 0.0),     # beside the DC jack; no gussets, a plate stud is near
}
LATCH_W = 8.0
LATCH_T = 1.8                   # arm thickness: three lines with a 0.6 mm nozzle
LATCH_GAP = 0.3                 # board edge to arm, relaxed
LATCH_REACH = 0.8               # how far the hook reaches over the board
LATCH_ROOT_DEPTH = 1.5          # arm starts in a pocket this deep in the plate: a longer arm bends gentler
LATCH_STRAIN_LIMIT = 0.015      # stricter than PLA_STRAIN_LIMIT: the arm prints upright, so it bends across layers
MIN_LATCH_REACH = 0.6
LATCH_ANGLE = 20.0              # hook underside: inward spring becomes a downward force
LATCH_PRELOAD = 0.15            # arm stays bent outward this much with the board seated
LATCH_LIP_T = 1.0               # hook thickness at its tip
KEY_T, KEY_W = 1.8, 6.0         # lock key thickness (across the slot) and width
KEY_CLEAR = 0.1                 # each side of the key in its slot
KEY_SLOT = 1.4                  # split between the key's two detent prongs
KEY_PRONG = 9.0                 # prong length
KEY_BUMP = 0.35                 # detent bump on each prong, 45 degree flanks
KEY_NOTCH_Z = 2.0               # detent height above the plate
KEY_TAB = 4.5                   # key above the latch, with a hole for a pull ribbon
CHEEK_T = 1.25
BACK_T = 2.4                    # backstop wall the key bears on
EDGE_GAP = 0.5                  # board edge to pillar

UNO_TOP_Z = PLATE_T + UNO_POST_H + PCB_UNO
NANO_SEAT = V_DEPTH * (V_MOUTH - NANO_T) / (V_MOUTH - V_APEX)   # edge depth in each groove
BASE_MOUTH_Z = NZ0 + NANO_SEAT
CAP_MOUTH_Z = NZ0 + NANO_W - NANO_SEAT
NANO_MID_Y = NY0 + NANO_T / 2


def uno_xy(ux, uy):
    return UNO_ORIGIN[0] + ux, UNO_ORIGIN[1] + uy


FAR_PILLAR = (35.0, -30.0)
NANO_ANCHOR = (-10.0, -30.0)


def nano_anchor():
    """Pillar for the lower clamp's arm, between the Uno's bottom edge and the Nano."""
    return NANO_ANCHOR


def pillar_positions():
    return [nano_anchor(), FAR_PILLAR]


def latch_arm():
    return UNO_TOP_Z - (PLATE_T - LATCH_ROOT_DEPTH)


def latch_pocket(name):
    """Room in the plate for the lower end of the arm to flex."""
    floor = PLATE_T - LATCH_ROOT_DEPTH
    u1 = LATCH_GAP + LATCH_T + LATCH_REACH + 0.2
    return _place(box(LATCH_GAP - 1.2, -LATCH_W / 2 - 0.4, floor, u1, LATCH_W / 2 + 0.4, PLATE_T + EPS), name)


def latch_strain():
    """Bending strain at the arm root as the hook rides over the board edge."""
    return 1.5 * LATCH_T * LATCH_REACH / latch_arm() ** 2


def key_strain():
    w = (KEY_W - KEY_SLOT) / 2
    return 1.5 * w * KEY_BUMP / KEY_PRONG ** 2


def latch_preload():
    """Downward force on the board from one latch, newtons (PLA E = 3.5 GPa, no friction)."""
    import math
    stiffness = 3 * 3500 * (LATCH_W * LATCH_T ** 3 / 12) / latch_arm() ** 3
    return stiffness * LATCH_PRELOAD / math.tan(math.radians(LATCH_ANGLE))


def _latch_frame(name):
    """(angle, origin): latch-local u points out from the board edge, v along it."""
    edge, along, _ = LATCHES[name]
    ux0, uy0 = UNO_ORIGIN
    return {"right": (0, (ux0 + UNO[0], uy0 + along)), "top": (90, (ux0 + along, uy0 + UNO[1])),
            "left": (180, (ux0, uy0 + along)), "bottom": (270, (ux0 + along, uy0))}[edge]


def _place(solid, name):
    angle, (x, y) = _latch_frame(name)
    return solid.rotate((0, 0, angle)).translate((x, y, 0))


def _poly(points):
    """Polygon with counter-clockwise winding, whichever way the points were listed."""
    area = sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1) in zip(points, points[1:] + points[:1]))
    return cad.CrossSection([points if area > 0 else points[::-1]])


def _uz_prism(points, width):
    """Extrude a (u, z) profile across v = -width/2 .. width/2."""
    return _poly(points).extrude(width).rotate((90, 0, 0)).translate((0, width / 2, 0))


def _key_slot():
    """u of the arm's back (seated), the key's inner face, and the backstop's front."""
    back = LATCH_GAP + LATCH_T + LATCH_PRELOAD
    return back, back + KEY_CLEAR, back + 2 * KEY_CLEAR + KEY_T


def latch_top_z():
    import math
    tip = -LATCH_REACH
    under_tip = UNO_TOP_Z + (-LATCH_PRELOAD - tip) * math.tan(math.radians(LATCH_ANGLE))
    return under_tip + LATCH_LIP_T + (LATCH_GAP - tip)


def latch(name, deflect=0.0):
    """One latch in plate coordinates. deflect bends the arm outward (approximated as a shift
    of the hook), for checking the seated position and the snap-in path."""
    import math
    top = PLATE_T
    t = math.tan(math.radians(LATCH_ANGLE))
    tip = -LATCH_REACH                                      # hook tip, relaxed
    z_tip_under = UNO_TOP_Z + (-LATCH_PRELOAD - tip) * t
    z_root_under = UNO_TOP_Z - (LATCH_GAP + LATCH_PRELOAD) * t
    z_top = latch_top_z()
    floor = top - LATCH_ROOT_DEPTH
    arm = box(LATCH_GAP, -LATCH_W / 2, floor - EPS, LATCH_GAP + LATCH_T, LATCH_W / 2, z_top)
    hook = _uz_prism([(LATCH_GAP + EPS, z_root_under), (tip, z_tip_under), (tip, z_tip_under + LATCH_LIP_T),
                      (LATCH_GAP + EPS, z_top)], LATCH_W)
    root = _uz_prism([(LATCH_GAP + EPS, floor - EPS), (LATCH_GAP - 1.0, floor - EPS), (LATCH_GAP + EPS, floor + 1.0)],
                     LATCH_W)                               # small root fillet on the board side
    moving = arm + hook
    if deflect:
        moving = moving.translate((deflect, 0, 0))
    _, _, front = _key_slot()
    cheek_u = LATCH_GAP + LATCH_T + LATCH_REACH + 0.2       # clear of the arm while it snaps
    bw = KEY_W + 2 * 0.15 + 2 * CHEEK_T
    fixed = box(front, -bw / 2, top - EPS, front + BACK_T, bw / 2, z_top)
    for side in (-1, 1):
        y0, y1 = sorted((side * (KEY_W / 2 + 0.15), side * bw / 2))
        cheek = box(cheek_u, y0, top - EPS, front + EPS, y1, z_top)
        notch = cad.Manifold.cube((front - cheek_u + 2, 0.9 / math.sqrt(2), 0.9 / math.sqrt(2)), True)
        notch = notch.rotate((45, 0, 0)).translate(((cheek_u + front) / 2, side * (KEY_W / 2 + 0.15),
                                                     top + KEY_NOTCH_Z))
        fixed += cheek - notch
    gusset = LATCHES[name][2]
    if gusset:
        for v in (-bw / 2 + 0.8, bw / 2 - 0.8):
            g = _uz_prism([(front + BACK_T - EPS, top - EPS), (front + BACK_T + gusset, top - EPS),
                           (front + BACK_T - EPS, z_top - 1.0)], 1.6)
            fixed += g.translate((0, v, 0))
    return _place(moving + root + fixed, name)


def key_flat():
    """Lock key as printed: lying flat, prongs down the page. x across, y along, z thickness."""
    length = latch_top_z() - PLATE_T + KEY_TAB
    w, b = KEY_W / 2, KEY_BUMP
    body = cad.CrossSection.square((KEY_W, length - KEY_TAB + 0.6)).translate((-w, 0))
    body -= cad.CrossSection.square((KEY_SLOT, KEY_PRONG)).translate((-KEY_SLOT / 2, -EPS))
    for side in (-1, 1):
        body += _poly([(side * w, KEY_NOTCH_Z - b), (side * (w + b), KEY_NOTCH_Z), (side * w, KEY_NOTCH_Z + b)])
        body -= _poly([(side * (w + 1), -1), (side * (w - 0.5), -1), (side * (w + 1), 0.5)])
    tab = cad.CrossSection.square((KEY_W + 2.6, KEY_TAB)).translate((-w - 1.3, length - KEY_TAB))
    tab = tab.offset(-0.8, cad.JoinType.Round, circular_segments=24).offset(0.8, cad.JoinType.Round, circular_segments=24)
    key = (body + tab) - cad.CrossSection.circle(1.3, 24).translate((0, length - KEY_TAB / 2))
    return key.extrude(KEY_T)


def key_in_place(name):
    """The key seated in its latch, bottom on the plate."""
    _, inner, _ = _key_slot()
    k = key_flat()        # x = v, y = z above the plate, z = u from the key's inner face
    k = k.transform([[0, 0, 1, inner], [1, 0, 0, 0], [0, 1, 0, PLATE_T]])
    return _place(k, name)


def cyl(d, h, x=0.0, y=0.0, z=0.0, d2=None):
    return cad.Manifold.cylinder(h, d / 2, (d2 if d2 is not None else d) / 2,
                                 circular_segments=FACETS).translate((x, y, z))


def box(x0, y0, z0, x1, y1, z1):
    return cad.Manifold.cube((x1 - x0, y1 - y0, z1 - z0)).translate((x0, y0, z0))


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
    """AMSAT end piece with hollow columns growing from the face's inner side (frame
    coordinates). The plate's quarter-turn studs go up through them from outside."""
    part = load("cubesatsim_frame_top_bottom.v2.0.stl")
    for x, y in STACK_HOLES:
        z0 = FACE_INNER_Z - COLUMN_LENGTH
        part += cyl(COLUMN_OD, COLUMN_LENGTH + FACE_T, x, y, z0)
    for x, y in STACK_HOLES:
        part -= cyl(QT_BORE, COLUMN_LENGTH + FACE_T + 2, x, y, FACE_INNER_Z - COLUMN_LENGTH - 1)
    return part


def plate_holes():
    """Stack holes seen from the board side of the plate (plate printed board side up)."""
    return [(x, -y) for x, y in STACK_HOLES]


def tip_rooms():
    """Room on the plate's board side for each plate stud's tip, at its deepest (the detent crest)."""
    h = qt_stud_length(plate_stud_a0()) - plate_stud_a0() + QT_BAR_Z - PLATE_T + 1.0
    return [cyl(QT_BORE + 1.0, max(h, 0.5), x, y, PLATE_T) for x, y in plate_holes()]


# ---------- quarter-turn fasteners ----------

def qt_drop(phi):
    """How far the cam has drawn the stud in at phi degrees of turn."""
    if phi <= 75:
        return QT_LIFT * phi / 75
    if phi <= 82:
        return QT_LIFT + QT_DET * (phi - 75) / 7
    return QT_LIFT + QT_DET * (90 - phi) / 8


def qt_stud_length(a0):
    """Shank length so that QT_TIP of stud stays behind the bar when it's locked."""
    return a0 + QT_BAR_H + QT_TIP


def qt_bar_box(a, theta, length=None):
    """The bar as it sits relative to the stud: at depth a from the head, turned theta."""
    w = (QT_BAR_W + QT_CLEAR) / 2
    L = length or QT_D + 2
    return box(-L, -w, a - QT_CLEAR, L, w, a + QT_BAR_H).rotate((0, 0, theta))


def qt_stud(a0):
    """Stud as printed: head down on the bed, shank up. z = 0 is the underside of the head, and
    a0 is the bar's distance from it before turning (washer at rest)."""
    length = qt_stud_length(a0)
    head = cyl(QT_HEAD_D, QT_HEAD_H, z=-QT_HEAD_H) - box(-QT_HEAD_D, -0.8, -QT_HEAD_H - 1, QT_HEAD_D, 0.8, -QT_HEAD_H + 1.2)
    shank = cyl(QT_D, length + EPS, z=-EPS) - (cyl(QT_D + 2, 0.6, z=length - 0.6) - cyl(QT_D + 0.1, 0.6 + EPS, z=length - 0.6 - EPS / 2, d2=QT_D - 1.0))
    slot = qt_bar_box(a0, 0).translate((0, 0, 0)) + box(-QT_D, -(QT_BAR_W + QT_CLEAR) / 2, a0 - QT_CLEAR,
                                                         QT_D, (QT_BAR_W + QT_CLEAR) / 2, length + 1)
    steps = list(range(0, 91, 5)) + [78, 82, 86]
    steps = sorted(set(steps))
    prev = None
    for phi in steps:
        b = qt_bar_box(a0 - qt_drop(phi), -phi)
        slot += b if prev is None else cad.Manifold.hull(prev + b)
        prev = b
    return head + shank - slot


def qt_bar(x, y, z_face, downward):
    """Receptacle cross-bar in a hole of diameter QT_BORE, its near face QT_BAR_Z inside the entry
    face at z_face. downward: the stud comes in from above (entry face on top)."""
    z0 = z_face - QT_BAR_Z - QT_BAR_H if downward else z_face + QT_BAR_Z
    return box(-QT_BORE / 2 - 0.5, -QT_BAR_W / 2, z0, QT_BORE / 2 + 0.5, QT_BAR_W / 2, z0 + QT_BAR_H).translate((x, y, 0))


def _arc_band(r0, r1, a0, a1, n=48):
    import math
    outer = [(r1 * math.cos(math.radians(a0 + (a1 - a0) * i / n)), r1 * math.sin(math.radians(a0 + (a1 - a0) * i / n)))
             for i in range(n + 1)]
    inner = [(r0 * math.cos(math.radians(a1 - (a1 - a0) * i / n)), r0 * math.sin(math.radians(a1 - (a1 - a0) * i / n)))
             for i in range(n + 1)]
    return _poly(outer + inner)


def washer_radii():
    hub = WASHER_ID / 2 + 1.0
    arm0 = hub + WASHER_SLIT
    arm1 = arm0 + WASHER_ARM
    ring0 = arm1 + WASHER_SLIT
    return hub, arm0, arm1, ring0


def washer_arm_length():
    import math
    _, arm0, arm1, _ = washer_radii()
    return (arm0 + arm1) / 2 * math.radians(WASHER_SWEEP)


def qt_washer():
    """Spring washer as printed: flat, feet up. Fit it feet down under the stud head."""
    hub, arm0, arm1, ring0 = washer_radii()
    flat = _arc_band(WASHER_ID / 2, hub, 0, 360) + _arc_band(ring0, WASHER_OD / 2, 0, 360)
    for k in (0, 180):
        flat += _arc_band(arm0, arm1, k, k + WASHER_SWEEP)
        flat += _arc_band(hub - 0.1, arm0 + 0.1, k, k + 12)                       # spoke from the hub
        flat += _arc_band(arm1 - 0.1, ring0 + 0.1, k + WASHER_SWEEP - 12, k + WASHER_SWEEP)   # spoke to the ring
    washer = flat.simplify(0.02).extrude(WASHER_T)
    for k in (90, 210, 330):
        washer += _arc_band(ring0 + 0.3, WASHER_OD / 2 - 0.3, k - 6, k + 6, 8).extrude(WASHER_FOOT + EPS).translate((0, 0, WASHER_T - EPS))
    return washer


def washer_strain(deflection=None):
    """Arm bending strain, guided-end beam, at the detent crest by default."""
    d = QT_LIFT + QT_DET + NANO_PRELOAD if deflection is None else deflection
    return 3 * WASHER_T * d / washer_arm_length() ** 2


def washer_force(deflection):
    """Newtons, both arms, PLA E = 3.5 GPa."""
    L = washer_arm_length()
    return 2 * 12 * 3500 * (WASHER_ARM * WASHER_T ** 3 / 12) / L ** 3 * deflection


def plate_stud_a0():
    """Head underside to the plate bar, washer at rest: washer, face, column, then into the plate."""
    return WASHER_H + FACE_T + COLUMN_LENGTH + QT_BAR_Z


def clamp_stud_a0():
    """Head underside to a pillar's bar: washer, clamp, then into the pillar."""
    return WASHER_H + CLAMP_T + QT_BAR_Z


def placed_stud(a0, x, y, z_head, phi=0.0, downward=True):
    """A stud in place: head underside at z_head before turning, turned phi and drawn in."""
    stud = qt_stud(a0).rotate((0, 0, phi))
    d = qt_drop(phi)
    if downward:
        return stud.mirror((0, 0, 1)).translate((x, y, z_head - d))
    return stud.translate((x, y, z_head + d))


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

    # Uno snap latches, each rooted in a pocket so its arm is longer.
    for name in LATCHES:
        part -= latch_pocket(name)
        part += latch(name)

    # Pillars for the Nano clamp's two quarter-turn studs, each with a cross-bar inside.
    bores, bars = [], []
    z_pillar = CAP_MOUTH_Z - NANO_PRELOAD
    for x, y in pillar_positions():
        part += cyl(PILLAR_OD, z_pillar - top + EPS, x, y, top - EPS)
        depth = qt_stud_length(clamp_stud_a0()) - (WASHER_H + CLAMP_T) + 1.0
        bores.append(cyl(QT_BORE, depth + 1, x, y, z_pillar - depth))
        bars.append(qt_bar(x, y, z_pillar, downward=True))

    # Nano card cage, lower half: V-groove rail, USB-end corner stop, far-end wall.
    y0, y1 = NY0 - V_WALL, NY0 + NANO_T + V_WALL
    x_end = NX0 + NANO_L + NANO_END_PLAY
    part += box(NX0 - 2.0, y0, top - EPS, x_end + 1.0, y1, BASE_MOUTH_Z)
    part += box(NX0 - 2.0, NY0 - 1.0, BASE_MOUTH_Z - EPS, NX0, NY0 + NANO_T + 1.0, NZ0 + NANO_STOP_REACH)
    part += box(x_end, y0, top - EPS, x_end + 2.0, y1, NZ0 + NANO_W - 2.0)
    part -= _groove([(-V_MOUTH / 2, 0), (-V_APEX / 2, -V_DEPTH), (V_APEX / 2, -V_DEPTH), (V_MOUTH / 2, 0),
                     (V_MOUTH / 2, 1), (-V_MOUTH / 2, 1)], NX0 - EPS, x_end, BASE_MOUTH_Z)

    for x, y in plate_holes():
        part -= cyl(QT_BORE, PLATE_T + 2, x, y, -1)
    for room in tip_rooms():
        part -= room                      # pillars and rails near a stud tip get a pocket
    for bore in bores:
        part -= bore
    for x, y in plate_holes():
        part += qt_bar(x, y, 0.0, downward=False)       # studs come up from the frame side
    for bar in bars:
        part += bar
    return part


def lower_clamp():
    """Cap rail over the Nano's upper edge, held down at both ends by quarter-turn studs. One
    flat top face, so it prints upside down without supports. It doesn't touch the Uno, so
    either board comes out on its own. Built resting on its pillars; the Nano's edge lifts it
    NANO_PRELOAD in service."""
    zs = CAP_MOUTH_Z - NANO_PRELOAD
    zt = zs + CLAMP_T
    hx, hy = nano_anchor()
    fx, fy = FAR_PILLAR
    pad = WASHER_OD + 0.5
    rail = box(NX0 - 2.0, NY0 - V_WALL, zs, fx, NY0 + NANO_T + V_WALL, zt)
    rail += cyl(pad, zt - zs, fx, fy, zs) + box(fx - 4.0, NY0, zs, fx + 4.0, fy, zt)
    rail = rail ^ chamfered_square(PLATE - 1.0, PLATE_CHAMFER, 100).translate((0, 0, -10))
    arm = box(hx - 4.0, NY0, zs, hx + 4.0, hy, zt) + cyl(pad, zt - zs, hx, hy, zs)
    stop = box(NX0 - 2.0, NY0 - 1.0, NZ0 + NANO_W - NANO_STOP_REACH, NX0, NY0 + NANO_T + 1.0, zs + EPS)
    part = rail + arm + stop
    part -= _groove([(-V_MOUTH / 2, -1), (V_MOUTH / 2, -1), (V_MOUTH / 2, 0),
                     (V_APEX / 2, V_DEPTH), (-V_APEX / 2, V_DEPTH), (-V_MOUTH / 2, 0)],
                    NX0 - EPS, NX0 + NANO_L + NANO_END_PLAY + EPS, zs)
    for x, y in ((hx, hy), (fx, fy)):
        part -= cyl(QT_BORE, zt - zs + 2, x, y, zs - 1)
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
    for x, y in plate_holes():
        solid += cad.CrossSection.circle(7.0, FACETS).translate((x, y))
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

    # Latch pockets and the lower ends of the latch arms, as on the payload plate.
    dz = FIT_BASE_T - top
    below = box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    for name in LATCHES:
        panel -= latch_pocket(name).translate((0, 0, dz))
        panel += (latch(name) ^ below).translate((0, 0, dz))
    require(FIT_BASE_T - LATCH_ROOT_DEPTH >= 0.6, "the fit plate is too thin under the latch pockets")
    # Quarter-turn receptacles, so the fit plate mounts on the bottom piece like the real one.
    for x, y in plate_holes():
        panel -= cyl(QT_BORE, FIT_BASE_T + 2, x, y, -1)
        panel += qt_bar(x, y, 0.0, downward=False)
    require(QT_BAR_Z + QT_BAR_H <= FIT_BASE_T, "the receptacle bar doesn't fit in the fit plate")
    return panel + features.translate((0, 0, dz))


def load_test():
    """One plate fastener on its own, for a pull test: a column stub on a hanging tab (the
    bottom piece's face and column) and a plate tab with a receptacle bar and a weight loop."""
    stack = FACE_T + COLUMN_LENGTH
    base = box(-8, -8, 0, 24, 8, FACE_T) - cyl(5.0, FACE_T + 2, 18, 0, -1)
    post = base + cyl(COLUMN_OD, stack, 0, 0, 0)
    post -= cyl(QT_BORE, stack + 2, 0, 0, -1)
    tab = box(-8, -8, 0, 24, 8, PLATE_T) - cyl(QT_BORE, PLATE_T + 2, 0, 0, -1) - cyl(5.0, PLATE_T + 2, 18, 0, -1)
    tab += qt_bar(0, 0, 0.0, downward=False)
    return post, tab


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
    keys = [key_in_place(name) for name in LATCHES]

    # Simple design rules first.
    require(latch_strain() < LATCH_STRAIN_LIMIT,
            f"Uno latch arm bends {latch_strain():.1%} snapping in, over the {LATCH_STRAIN_LIMIT:.1%} "
            "limit for an arm printed upright")
    require(PLATE_T - LATCH_ROOT_DEPTH >= 1.2, "too little plate left under the latch pockets")
    require(at_least(LATCH_REACH, MIN_LATCH_REACH),
            f"Uno latch hooks reach {LATCH_REACH} mm over the board, less than {MIN_LATCH_REACH} mm")
    free = 2 * KEY_CLEAR
    require(free < (LATCH_REACH - LATCH_PRELOAD) / 2,
            f"with its key in, a latch can still open {free:.2f} mm, more than half its "
            f"{LATCH_REACH - LATCH_PRELOAD:.2f} mm grip, so the key doesn't lock it")
    require(0 < LATCH_PRELOAD < LATCH_REACH / 2, "latch preload must be positive and well inside the hook's reach")
    require(key_strain() < PLA_STRAIN_LIMIT, f"lock key prongs bend {key_strain():.1%}, too much for PLA")
    require(KEY_SLOT >= 2 * KEY_BUMP + 0.2, "the key's slot is too narrow for its prongs to flex past the notches")
    require(at_least(NANO_SEAT, MIN_SEAT), f"the Nano's edges sit {NANO_SEAT:.2f} mm into the V-grooves, "
            f"less than {MIN_SEAT} mm")
    lowest = min(y0 for _, y0, _, _, _ in NANO_PARTS.values() if y0 > 0)
    highest = max(y1 for _, _, _, y1, _ in NANO_PARTS.values())
    require(NANO_SEAT + 0.3 < min(lowest, NANO_W - highest),
            "the V-groove walls reach the parts along the Nano's edges")

    # Quarter-turn fasteners.
    crest = QT_LIFT + QT_DET
    require(0 < QT_DET < QT_LIFT, "each quarter-turn stud needs a detent smaller than its cam draw-in")
    require(at_least(PLA_STRAIN_LIMIT, washer_strain()),
            f"spring washer arms bend {washer_strain():.2%} at the detent crest, over {PLA_STRAIN_LIMIT:.0%}")
    require(WASHER_FOOT >= crest + NANO_PRELOAD + 0.2, "the washer's feet are too short for its hub to travel")
    need = PAYLOAD_MASS * 9.81 * DESIGN_G
    have = 4 * washer_force(QT_LIFT)
    require(have >= need, f"the four plate fasteners hold {have:.1f} N down, less than the {need:.1f} N "
            f"the payload weighs at {DESIGN_G:.0f} g, so the plate could rattle")
    require(QT_TIP >= 1.2, "too little stud left behind the bar to carry the load")
    # The bar is fused into the hole wall at both ends: fixed-fixed beam, central load, M = F L / 8.
    bar_stress = washer_force(crest + NANO_PRELOAD) * QT_BORE / 8 / (QT_BAR_W * QT_BAR_H ** 2 / 6)
    require(bar_stress < 20.0, f"receptacle bar bending stress {bar_stress:.0f} MPa is too high for PLA")

    # Plate on the columns.
    seated = plate.rotate((180, 0, 0)).translate((0, 0, FACE_INNER_Z - COLUMN_LENGTH))
    for (x, y), (px, py) in zip(STACK_HOLES, plate_holes()):
        require(np.allclose((px, -py), (x, y)), "plate holes do not line up with the columns")
    require(overlap(seated, bottom) < 1e-3, "payload plate collides with the AMSAT end piece")
    require(seated.bounding_box()[5] <= FACE_INNER_Z - SNAP_TABS_REACH,
            "payload plate does not clear the AMSAT snap tabs")
    frame = bottom + seated
    z_head = FACE_INNER_Z + FACE_T + WASHER_H
    a0 = plate_stud_a0()
    for x, y in STACK_HOLES:
        entry = placed_stud(a0, x, y, z_head, 0.0)
        require(overlap(entry, frame) < 1e-3, "a plate stud can't be pushed in past its receptacle bar")
        locked = placed_stud(a0, x, y, z_head - 0.02, 90.0)
        require(overlap(locked, frame) < 1e-3, "a locked plate stud collides with the plate or column")
        pulled = placed_stud(a0, x, y, z_head + 0.3, 90.0)
        require(overlap(pulled, frame) > 1e-3, "a locked plate stud isn't held by its bar")
        outside = cyl(WASHER_OD + 0.5, WASHER_H + QT_HEAD_H, x, y, FACE_INNER_Z + FACE_T + 0.01)
        require(overlap(outside, bottom) < 1e-3, "a plate stud's washer or head sits on a raised part of the face")

    # Board side of the plate, in plate coordinates.
    top = PLATE_T
    features = plate - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    lower = lower.translate((0, 0, NANO_PRELOAD))          # lifted by the Nano's edge, in service
    heads = cad.Manifold()
    for x, y in pillar_positions():
        heads += cyl(WASHER_OD, WASHER_H + QT_HEAD_H, x, y, CAP_MOUTH_Z + CLAMP_T)
    everything = features + lower + heads
    for key in keys:
        everything += key
    for room in tip_rooms():
        require(overlap(everything, room) < 1e-3, "a part sits where a plate stud's tip comes through")
    require(overlap(everything, outline_prism(top, 80) ^ everything) >= everything.volume() - 1e-2,
            "a part reaches outside the plate outline, into the frame's corner rails")

    # Uno R3: board, underside and parts, with every latch bent to its seated position.
    ux, uy = UNO_ORIGIN
    pcb_bottom = top + UNO_POST_H
    seated_latches = cad.Manifold()
    for name in LATCHES:
        seated_latches += latch(name, LATCH_PRELOAD + 0.01)
    fixed = features
    for name in LATCHES:
        fixed -= latch(name)
    in_service = fixed + seated_latches + lower + heads
    for key in keys:
        in_service += key
    uno = box(ux, uy, pcb_bottom, ux + UNO[0], uy + UNO[1], UNO_TOP_Z)
    posts = cad.Manifold()
    for hx, hy in UNO_HOLES:
        uno -= cyl(3.2, PCB_UNO + 2, ux + hx, uy + hy, pcb_bottom - 1)
        posts += cyl(7.0, UNO_POST_H + PCB_UNO + 2, ux + hx, uy + hy, top - 1)
    require(overlap(in_service, uno) < 1e-3, "a latch or part passes through the Uno board")
    underside = box(ux, uy, pcb_bottom - 2.0, ux + UNO[0], uy + UNO[1], pcb_bottom) - posts
    require(overlap(in_service, underside) < 1e-3, "something under the Uno would hit its solder joints")
    for name, body in uno_parts().items():
        require(overlap(in_service, body) < 1e-3, f"a latch, key or pillar hits the Uno's {name}")
    above_board = box(ux, uy, UNO_TOP_Z, ux + UNO[0], uy + UNO[1], UNO_TOP_Z + 3.0)
    for name in LATCHES:
        held = overlap(latch(name, LATCH_PRELOAD + 0.01), above_board) / LATCH_W
        require(held > 0.3, f"the {name} latch's hook doesn't reach over the Uno")
        for key in keys:
            require(overlap(key, latch(name)) < 1e-3, f"a lock key doesn't fit the {name} latch")
        opened = latch(name, LATCH_PRELOAD + 2 * KEY_CLEAR + 0.05)
        require(sum(overlap(opened, k) for k in keys) > 1e-3,
                f"the {name} latch can open past its key: the key doesn't lock it")
    snapping = fixed
    for name in LATCHES:
        snapping += latch(name, LATCH_REACH + 0.05)
    path = box(ux, uy, UNO_TOP_Z + 0.01, ux + UNO[0], uy + UNO[1], 80) + box(ux - 7, uy + 30, UNO_TOP_Z + 0.01, ux, uy + 46, 80)
    require(overlap(snapping, path) < 1e-3, "the Uno can't be pushed down past its latches")

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

    # Nano clamp studs: pass the bars going in, held by them when locked.
    design = clamps["lower-clamp"]
    a0 = clamp_stud_a0()
    z_head = CAP_MOUTH_Z - NANO_PRELOAD + CLAMP_T + WASHER_H
    for x, y in pillar_positions():
        stack = plate + design
        require(overlap(placed_stud(a0, x, y, z_head, 0.0), stack) < 1e-3,
                "a Nano clamp stud can't be pushed in past its bar")
        require(overlap(placed_stud(a0, x, y, z_head - 0.02, 90.0), stack) < 1e-3,
                "a locked Nano clamp stud collides with its pillar or the clamp")
        require(overlap(placed_stud(a0, x, y, z_head + 0.3, 90.0), stack) > 1e-3,
                "a locked Nano clamp stud isn't held by its bar")


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
    return {"lower-clamp": lower_clamp()}


BROKEN_DESIGNS = {
    "plate inside the snap tabs": {"COLUMN_LENGTH": 10.0},
    "spring washer arms too thick to flex": {"WASHER_T": 2.2},
    "no detent, so a stud could turn loose": {"QT_DET": 0.0},
    "too little cam draw-in to hold the plate down": {"QT_LIFT": 0.3},
    "stud tip too short behind the bar": {"QT_TIP": 0.6},
    "Nano clamp pillar moved into the Nano's USB plug": {"NANO_ANCHOR": (-18.0, -30.5)},
    "latch hooks too short to hold the Uno": {"LATCH_REACH": 0.4},
    "stiff latch arm that would crack": {"LATCH_T": 3.0},
    "lock key too loose to stop a latch opening": {"KEY_CLEAR": 0.3},
    "left latch moved into the USB-B connector": {"LATCHES": {**LATCHES, "left": ("left", 38.0, 1.5)}},
    "V-grooves too shallow to grip the Nano": {"V_DEPTH": 0.4},
    "Nano too close to the plate edge for its USB plug": {"NX0": -26.0},
    "Uno moved 3 mm towards the Nano": {"UNO_ORIGIN": (-35.66, -24.34)},
    "Nano parts against the groove wall": {"V_DEPTH": 4.0},
}

# Designs exactly at a limit must pass: the limits are inclusive.
BOUNDARY_DESIGNS = {
    "spring washer exactly at the 2% strain limit": {
        "WASHER_T": PLA_STRAIN_LIMIT * washer_arm_length() ** 2 / (3 * (QT_LIFT + QT_DET + NANO_PRELOAD))},
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
    print(f"checks passed: Nano edge in each groove {NANO_SEAT:.2f} mm")
    print(f"Uno latches: arm strain {latch_strain():.1%}, hook reach {LATCH_REACH} mm, "
          f"opening with key in {2 * KEY_CLEAR:.2f} mm, preload about {latch_preload():.0f} N each, "
          f"key prong strain {key_strain():.1%}")
    print(f"quarter-turn fasteners: washer strain {washer_strain():.2%} at the detent crest, "
          f"plate held down with {4 * washer_force(QT_LIFT):.0f} N "
          f"(payload at {DESIGN_G:.0f} g: {PAYLOAD_MASS * 9.81 * DESIGN_G:.0f} N), "
          f"Nano clamp {2 * washer_force(QT_LIFT + NANO_PRELOAD):.0f} N")
    # The end piece prints face down, so its columns point up.
    export("bottom-with-columns", bottom.rotate((180, 0, 0)))
    export("payload-plate", plate)
    export("lock-key", key_flat())
    export("lower-clamp", clamps["lower-clamp"].rotate((180, 0, 0)))      # flat top on the bed
    export("qt-stud-plate", qt_stud(plate_stud_a0()))
    export("qt-stud-clamp", qt_stud(clamp_stud_a0()))
    export("qt-washer", qt_washer())
    fit = export("fit-plate", fit_plate())
    print(f"fit-plate: {fit.volume / 1000:.1f} cm3 against {check('payload-plate', plate).volume / 1000:.1f} cm3 "
          "for the full payload plate")
    post, tab = load_test()
    export("load-test-post", post)
    export("load-test-tab", tab)
