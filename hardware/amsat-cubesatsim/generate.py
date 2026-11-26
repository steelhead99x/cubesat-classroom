"""Fully printed payload kit for the AMSAT CubeSatSim v2.0 frame. Dimensions in mm.

Builds, from the unmodified AMSAT frame in upstream/:
  bottom-with-columns  AMSAT top/bottom piece plus four standoff columns where the AMSAT
                       board stack screws to the frame; three end in snap pins, one in a
                       locating pin
  payload-plate        holds an Arduino Uno R3 (four latch clips) and a Nano 33 BLE Sense
                       (card cage); snaps down onto the columns
  lower-clamp          caps the Nano: hooks under a lip at one end, snaps onto a pin at the other
  latch-clip           holds one Uno edge (four), printed lying flat
  lock-key             locks a latch clip shut (four)
  fit-plate            the payload plate as an open truss, for fit tests and displays
  coupon-*             one of each joint on its own, about 5 g each, to test a printer first

No screws, nuts, springs or metal: everything is printed, and every part is in print
orientation with one large flat face, so a slicer's auto-orient keeps it that way.

Design rules (snap-fit practice for FDM printing):
  - Every part that flexes bends in its own print plane, along its printed strands, never
    across its layers. The Uno clips are separate parts for this reason.
  - Design strain at most 1.2%: 15% of PLA's roughly 8% elongation at break.
  - Nothing that carries load is thinner than 2 mm; clearances are 0.4 mm.
  - Every snap has a 45 degree face, so it prints without supports and pulls its joint tight
    whatever the print error, so nothing rattles.
  - The Uno clips have a secondary lock (a key), like an aerospace connector's position
    assurance clip.

This folder is a derivative of the AMSAT CubeSatSim frame by Alan Johnston,
KU2Y, and is licensed GPL-3.0 like the original. Modified by CubeSat Classroom
on 2026-11-25: standoff columns added to the AMSAT top/bottom piece, and new
payload parts. See LICENSE and README.md.

Run with --self-test to check that verify() rejects deliberately broken designs.
"""

import math
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

COLUMN_OD = 10.0                # its top is the shoulder the plate sits on
COLUMN_LENGTH = 15.0            # face to plate; clears the 12 mm snap tabs

PLATE = 88.0                    # AMSAT boards are 90.5 mm square
PLATE_CHAMFER = 8.0
PLATE_T = 3.0
TOLERANCE = 1e-6                # mm of floating-point noise to ignore; far below print resolution

# Snap-fit design rules.
SNAP_STRAIN_LIMIT = 0.012       # 15% of PLA's ~8% elongation at break
MIN_WALL = 2.0                  # nothing that flexes or carries load is thinner
PLA_E = 3500.0                  # MPa, for spring forces
FRICTION = 0.3                  # PLA on PLA
PAYLOAD_MASS = 0.10             # kg on the plate: plate, boards, clamp, clips, keys (estimate)
DESIGN_G = 20.0                 # quasi-static design load, a common figure for small-satellite launch
RETENTION_MARGIN = 1.5

# Snap pin and flex fingers. A pin ends in a head with a 45 degree underside. Two curved
# fingers in the part around the hole carry a tooth at their tips; pushing the part down, the
# head spreads the teeth, which then spring in under the head. The 45 degree face pulls the part
# down onto the pin's shoulder, so the joint stays tight however the print came out.
PIN_R = 3.0                     # shaft through the part
PIN_HEAD_R = 3.9
PIN_BITE = 0.45                 # the head's 45 degree face starts this far below the part's top
PIN_CAP = 0.8
PIN_LEAD = 1.5                  # 45 degree lead-in on top
LOCATE_CLEAR = 0.4              # plain locating pin in a plain hole
FINGER_CLEAR = 0.4              # head to fingers, while it passes
FINGER_W = 2.0                  # finger width, in the part's plane
FINGER_SLIT = 1.4               # room outside each finger to flex into
FINGER_SPAN = 165.0             # degrees of arc per finger
FINGER_GAP = 7.0                # degrees between a finger's tip and the next finger's root
TOOTH_R = 3.1                   # tooth inner radius, relaxed
TOOTH_SPAN = 28.0               # degrees
SNAP_COLUMNS = [0, 1, 2]        # stack holes with snap pins; the one by the Nano gets a locating pin

# Arduino Uno R3 (A000066, Eagle UNO-TH_Rev3e.brd): 68.58 x 53.34 mm, 1.6 mm PCB.
# Coordinates from the lower-left corner, USB on the left edge.
UNO = (68.58, 53.34)
UNO_HOLES = [(66.04, 35.56), (66.04, 7.62), (15.24, 50.80), (13.97, 2.54)]   # hole 0..3
UNO_ORIGIN = (-35.66, 32.0 - UNO[1])  # USB-B overhang ends 1.9 mm inside the plate edge
UNO_POST_H = 16.0               # tall enough for latch arms that bend gently
UNO_PIN = 2.9
UNO_PIN_ABOVE = -0.3            # pin tip below the board's top face
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
# CubeSat), USB end towards -x. NX0 = USB end, NY0 = back face, NZ0 = lower edge.
NX0, NY0, NZ0 = -14.0, -40.8, 4.5
NANO_END_PLAY = 0.5             # along the board, between the end stops (0.2 printed a little tight)
V_MOUTH, V_APEX, V_DEPTH = 2.4, 1.4, 2.0     # V-groove widths and depth that grip each edge
V_WALL = 2.6                    # groove wall beside the board
MIN_SEAT = 1.0                  # how far each Nano edge must sit inside its groove
EDGE_CLEAR = 2.8                # the board's edge strips are free of parts this far in
NANO_STOP_REACH = 3.0           # end stops cover the corners only, clear of the USB plug
PLUG = (28.0, 7.0, 10.5)        # Micro-USB plug envelope: length, thickness, width
MIN_PLUG_ROOM = 20.0            # from the receptacle to the plate edge, for the plug
NANO_PRELOAD = 0.15             # the Nano's top edge lifts its clamp this much

# Nano clamp: hooks under a lip past the Nano's far end, snaps onto a pin at the USB end.
PILLAR_OD = 10.0
CLAMP_T = 4.0
NANO_ANCHOR = (-7.0, -31.0)    # snap pillar, between the Uno's bottom edge and the Nano
CLAMP_PAD_R = 8.5               # clamp round the anchor's fingers
HOOK_T = 4.0                    # hook block past the far end wall
HOOK_LIP = 2.0                  # how far the lip reaches back over the clamp's tab
HOOK_BITE = 0.1                 # lip presses the tab down this much in service
HOOK_GAP = 0.4                  # tab end to the hook block

# Uno latch clips, each a separate part printed lying flat so its arm bends along its strands.
# A clip's foot slides out into a tunnel in its backstop; its arm stands beside the board edge
# with an angled hook over the board, so the arm's spring presses the board down onto its posts.
# Then a lock key drops in behind the arm, so it can't open. Latches sit only where Arduino's
# layout leaves the board edge clear (see UNO_PARTS).
LATCHES = {                     # name: (board edge, centre along that edge, gusset depth)
    "left": ("left", 21.0, 0.0),        # between the DC jack and the USB-B; no room for gussets
    "right": ("right", 16.0, 1.5),      # between hole 1 and the ICSP header
    "top": ("top", 22.5, 1.5),          # past the post at hole 2, over the 8-13 header's clear strip
}
# No latch on the bottom edge: every clear stretch there is on a column's flex fingers, a post
# or the Nano clamp. Three latches and four pinned posts hold the board: lifting its free edge
# would mean pushing its top edge down into the posts.
LATCH_W = 6.0                   # the clip's thickness as printed
LATCH_T = 2.0                   # arm thickness, in the clip's plane
LATCH_GAP = 0.3                 # board edge to arm, relaxed
LATCH_REACH = 0.7               # how far the hook reaches over the board
MIN_LATCH_REACH = 0.6
LATCH_ANGLE = 20.0              # hook underside: inward spring becomes a downward force
LATCH_PRELOAD = 0.15            # arm stays bent outward this much with the board seated
LATCH_LIP_T = 1.0               # hook thickness at its tip
FOOT_T = 2.6                    # clip foot, plate to tunnel ceiling
FOOT_IN = 3.0                   # foot reaches this far in under the board edge
ROOT_FILLET = 1.0
TUNNEL_CLEAR = 0.25
KEY_T, KEY_W = 1.8, 6.0         # lock key thickness (across the slot) and width
KEY_CLEAR = 0.1                 # each side of the key in its slot
KEY_SLOT = 1.4                  # split between the key's two detent prongs
KEY_PRONG = 11.0                # prong length
KEY_BUMP = 0.3                  # detent bump on each prong, 45 degree flanks
KEY_NOTCH_Z = 2.0               # detent height above the foot
KEY_TAB = 4.5                   # key above the latch, with a hole for a pull ribbon
CHEEK_T = 1.25
BACK_T = 3.6                    # backstop the key bears on; the foot's tunnel runs into it
TUNNEL_END = 1.0                # backstop left behind the tunnel's end

UNO_TOP_Z = PLATE_T + UNO_POST_H + PCB_UNO
FOOT_TOP = PLATE_T + FOOT_T
NANO_SEAT = V_DEPTH * (V_MOUTH - NANO_T) / (V_MOUTH - V_APEX)   # edge depth in each groove
BASE_MOUTH_Z = NZ0 + NANO_SEAT
CAP_MOUTH_Z = NZ0 + NANO_W - NANO_SEAT
NANO_MID_Y = NY0 + NANO_T / 2

# Branding: the CubeSat Classroom mark (docs/mark.svg) and lettering (label.json), raised.
LETTER_H = 0.6                  # swap filament at this layer for two colours
MARK_SIZE = 17.0                # mm across the mark's 120-unit drawing
MARK_AT = (0.0, -7.0)


def uno_xy(ux, uy):
    return UNO_ORIGIN[0] + ux, UNO_ORIGIN[1] + uy


def cyl(d, h, x=0.0, y=0.0, z=0.0, d2=None):
    return cad.Manifold.cylinder(h, d / 2, (d2 if d2 is not None else d) / 2,
                                 circular_segments=FACETS).translate((x, y, z))


def box(x0, y0, z0, x1, y1, z1):
    if x1 <= x0 or y1 <= y0 or z1 <= z0:
        return cad.Manifold()
    return cad.Manifold.cube((x1 - x0, y1 - y0, z1 - z0)).translate((x0, y0, z0))


def chamfered_square(size, chamfer, height):
    h = size / 2
    c = chamfer
    pts = [(-h + c, -h), (h - c, -h), (h, -h + c), (h, h - c),
           (h - c, h), (-h + c, h), (-h, h - c), (-h, -h + c)]
    return cad.CrossSection([pts]).extrude(height)


def _poly(points):
    """Polygon with counter-clockwise winding, whichever way the points were listed."""
    area = sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1) in zip(points, points[1:] + points[:1]))
    return cad.CrossSection([points if area > 0 else points[::-1]])


def _uz_prism(points, width):
    """Extrude a (u, z) profile across v = -width/2 .. width/2."""
    return _poly(points).extrude(width).rotate((90, 0, 0)).translate((0, width / 2, 0))


def _arc_band(r0, r1, a0, a1, n=None):
    n = n or max(4, int(abs(a1 - a0) / 4))
    outer = [(r1 * math.cos(math.radians(a0 + (a1 - a0) * i / n)), r1 * math.sin(math.radians(a0 + (a1 - a0) * i / n)))
             for i in range(n + 1)]
    inner = [(r0 * math.cos(math.radians(a1 - (a1 - a0) * i / n)), r0 * math.sin(math.radians(a1 - (a1 - a0) * i / n)))
             for i in range(n + 1)]
    return _poly(outer + inner)


def load(name):
    mesh = trimesh.load_mesh(HERE / "upstream" / name)
    return cad.Manifold(cad.Mesh(vert_properties=np.asarray(mesh.vertices, np.float32),
                                 tri_verts=np.asarray(mesh.faces, np.uint32)))


def plate_holes():
    """Stack holes seen from the board side of the plate (plate printed board side up)."""
    return [(x, -y) for x, y in STACK_HOLES]


def to_frame(solid):
    """Plate coordinates (plate underside at z = 0, board side up) to frame coordinates."""
    return solid.rotate((180, 0, 0)).translate((0, 0, FACE_INNER_Z - COLUMN_LENGTH))


# ---------- snap pins and flex fingers ----------

def pin(x, y, z_shoulder, part_t, post=False):
    """Snap pin standing on a shoulder at z_shoulder, for a part part_t thick. The head's 45
    degree underside starts PIN_BITE below the part's top face, so the teeth meet it there."""
    z_c = z_shoulder + part_t - PIN_BITE
    flank = PIN_HEAD_R - PIN_R
    shaft = cyl(2 * PIN_R, z_c - z_shoulder + 2 * EPS, x, y, z_shoulder - EPS)
    head = cyl(2 * PIN_R, flank, x, y, z_c, d2=2 * PIN_HEAD_R)
    head += cyl(2 * PIN_HEAD_R, PIN_CAP + 2 * EPS, x, y, z_c + flank - EPS)
    head += cyl(2 * PIN_HEAD_R, PIN_LEAD, x, y, z_c + flank + PIN_CAP, d2=2 * (PIN_HEAD_R - PIN_LEAD))
    return shaft + head


def pin_top(z_shoulder, part_t):
    return z_shoulder + part_t - PIN_BITE + (PIN_HEAD_R - PIN_R) + PIN_CAP + PIN_LEAD


def locating_pin(x, y, z_shoulder, part_t):
    top = z_shoulder + part_t + 1.0
    return cyl(2 * PIN_R, top - 0.6 - z_shoulder + EPS, x, y, z_shoulder - EPS) + \
        cyl(2 * PIN_R, 0.6, x, y, top - 0.6, d2=2 * PIN_R - 1.2)


def contact_r():
    """Where the teeth meet the head's 45 degree face: at the part's top face."""
    return PIN_R + PIN_BITE


def finger_radii():
    r_clear = PIN_HEAD_R + FINGER_CLEAR
    return r_clear, r_clear + FINGER_W, r_clear + FINGER_W + FINGER_SLIT


def finger_length():
    r_clear, r_out, _ = finger_radii()
    return (r_clear + r_out) / 2 * math.radians(FINGER_SPAN)


def finger_pass():
    """How far each tooth springs out while the head passes."""
    return PIN_HEAD_R + 0.1 - TOOTH_R


def finger_strain():
    return 1.5 * FINGER_W * finger_pass() / finger_length() ** 2


def finger_stiffness(part_t):
    """Radial spring rate at a finger's tip, N/mm, as a straight cantilever."""
    return 3 * PLA_E * (part_t * FINGER_W ** 3 / 12) / finger_length() ** 3


def pin_retention(part_t):
    """Pull needed to lift a part off one snap pin, N: both teeth pushed out to the head's
    full radius along its 45 degree face, against friction."""
    radial = finger_stiffness(part_t) * (PIN_HEAD_R - contact_r())
    return 2 * radial * math.tan(math.radians(45) + math.atan(FRICTION))


def finger_cut(cx, cy, a0, tooth_r=TOOTH_R):
    """Region cut from a part around a snap pin: the head's clearance, the slits that free two
    curved fingers, and the gaps at their tips, less the teeth. tooth_r lets the checks place
    the teeth where they sit with the pin in, or spread to let the head through."""
    r_clear, r_out, r_slit = finger_radii()
    cut = cad.CrossSection.circle(r_clear, FACETS)
    teeth = cad.CrossSection()
    for k in (0, 1):
        root = a0 + 180 * k
        tip = root + FINGER_SPAN
        cut += _arc_band(r_out, r_slit, root, tip + FINGER_GAP)
        cut += _arc_band(r_clear - EPS, r_slit, tip, tip + FINGER_GAP)
        teeth += _arc_band(tooth_r, r_clear + EPS, tip - TOOTH_SPAN, tip)
    cut = cut.offset(-0.5, cad.JoinType.Round, circular_segments=16).offset(0.5, cad.JoinType.Round, circular_segments=16)
    return (cut - teeth).translate((cx, cy))


def finger_zone(cx, cy, z0, h):
    """Room the fingers need to flex in: nothing else may stand on them."""
    return cyl(2 * finger_radii()[2] + 1.0, h, cx, cy, z0)


# Finger orientation at each snap column: roots away from the nearest features.
FINGER_A0 = {0: 100.0, 1: 260.0, 2: 0.0}
ANCHOR_A0 = 270.0


# ---------- Uno latch clips and keys ----------

def latch_arm():
    return UNO_TOP_Z - FOOT_TOP


def latch_strain():
    """Bending strain at the arm root as the hook rides over the board edge."""
    return 1.5 * LATCH_T * LATCH_REACH / latch_arm() ** 2


def key_strain():
    w = (KEY_W - KEY_SLOT) / 2
    return 1.5 * w * KEY_BUMP / KEY_PRONG ** 2


def latch_preload():
    """Downward force on the board from one latch, newtons (no friction)."""
    stiffness = 3 * PLA_E * (LATCH_W * LATCH_T ** 3 / 12) / latch_arm() ** 3
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


def _key_slot():
    """u of the arm's back (seated), the key's inner face, and the backstop's front."""
    back = LATCH_GAP + LATCH_T + LATCH_PRELOAD
    return back, back + KEY_CLEAR, back + 2 * KEY_CLEAR + KEY_T


def latch_top_z():
    tip = -LATCH_REACH
    under_tip = UNO_TOP_Z + (-LATCH_PRELOAD - tip) * math.tan(math.radians(LATCH_ANGLE))
    return under_tip + LATCH_LIP_T + (LATCH_GAP - tip)


def _backstop_w():
    return KEY_W + 2 * 0.15 + 2 * CHEEK_T


def foot_end():
    _, _, front = _key_slot()
    return front + BACK_T - TUNNEL_END - TUNNEL_CLEAR


def _tunnel():
    """Room in the backstop for the clip's foot. Its flat ceiling, a short bridge, is what the
    foot bears up against when the hook presses the board down, so it sets the hook's height;
    the clearance is under the foot."""
    w = LATCH_W / 2 + TUNNEL_CLEAR
    _, _, front = _key_slot()
    return box(front - EPS, -w, PLATE_T - EPS, front + BACK_T - TUNNEL_END, w, FOOT_TOP)


def latch_clip_local(deflect=0.0):
    """One clip in latch-local coordinates, seated. deflect bends the arm outward
    (approximated as a shift of the arm and hook), for the seated and snap-in positions."""
    t = math.tan(math.radians(LATCH_ANGLE))
    tip = -LATCH_REACH
    z_tip_under = UNO_TOP_Z + (-LATCH_PRELOAD - tip) * t
    z_root_under = UNO_TOP_Z - (LATCH_GAP + LATCH_PRELOAD) * t
    z_top = latch_top_z()
    arm = [(LATCH_GAP, FOOT_TOP - EPS), (LATCH_GAP + LATCH_T, FOOT_TOP - EPS), (LATCH_GAP + LATCH_T, z_top),
           (LATCH_GAP, z_top)]
    hook = [(LATCH_GAP + EPS, z_root_under), (tip, z_tip_under), (tip, z_tip_under + LATCH_LIP_T),
            (LATCH_GAP + EPS, z_top)]
    moving = (_poly(arm) + _poly(hook)).translate((deflect, 0))
    foot = _poly([(LATCH_GAP - FOOT_IN, PLATE_T + TUNNEL_CLEAR), (foot_end(), PLATE_T + TUNNEL_CLEAR),
                  (foot_end(), FOOT_TOP), (LATCH_GAP - FOOT_IN, FOOT_TOP)])
    fillet = _poly([(LATCH_GAP + EPS, FOOT_TOP - EPS), (LATCH_GAP - ROOT_FILLET, FOOT_TOP - EPS),
                    (LATCH_GAP + EPS, FOOT_TOP + ROOT_FILLET)])     # on the arm's tension side
    section = moving + foot + fillet
    return section.extrude(LATCH_W).rotate((90, 0, 0)).translate((0, LATCH_W / 2, 0))


def latch_clip(name, deflect=0.0):
    return _place(latch_clip_local(deflect), name)


def latch_clip_flat():
    """A clip as printed: lying on its side, so its arm and hook bend in the print plane."""
    return latch_clip_local().rotate((-90, 0, 0))


def latch_fixed(name):
    """The plate's side of a latch: cheeks that guide the key, a backstop it bears on with the
    foot's tunnel, and gussets behind it."""
    top = PLATE_T
    z_top = latch_top_z()
    back, _, front = _key_slot()
    cheek_u = LATCH_GAP + LATCH_T + LATCH_REACH + 0.2       # clear of the arm while it snaps
    bw = _backstop_w()
    fixed = box(front, -bw / 2, top - EPS, front + BACK_T, bw / 2, z_top)
    for side in (-1, 1):
        y0, y1 = sorted((side * (KEY_W / 2 + 0.15), side * bw / 2))
        cheek = box(cheek_u, y0, top - EPS, front + EPS, y1, z_top)
        notch = cad.Manifold.cube((front - cheek_u + 2, 0.9 / math.sqrt(2), 0.9 / math.sqrt(2)), True)
        notch = notch.rotate((45, 0, 0)).translate(((cheek_u + front) / 2, side * (KEY_W / 2 + 0.15),
                                                     FOOT_TOP + KEY_NOTCH_Z))
        fixed += cheek - notch
    fixed -= _tunnel()
    gusset = LATCHES[name][2]
    if gusset:
        for v in (-bw / 2 + 1.0, bw / 2 - 1.0):
            g = _uz_prism([(front + BACK_T - EPS, top - EPS), (front + BACK_T + gusset, top - EPS),
                           (front + BACK_T - EPS, z_top - 1.0)], 1.6)
            fixed += g.translate((0, v, 0))
    return _place(fixed, name)


def key_flat():
    """Lock key as printed: lying flat, prongs down the page. x across, y along, z thickness."""
    length = latch_top_z() - FOOT_TOP + KEY_TAB
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
    """The key seated in its latch, bottom on the clip's foot."""
    _, inner, _ = _key_slot()
    k = key_flat()        # x = v, y = z above the foot, z = u from the key's inner face
    k = k.transform([[0, 0, 1, inner], [1, 0, 0, 0], [0, 1, 0, FOOT_TOP]])
    return _place(k, name)


# ---------- Nano card cage ----------

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


def x_end():
    return NX0 + NANO_L + NANO_END_PLAY


def clamp_z():
    """Clamp underside and top as built, resting on the anchor pillar's shoulder."""
    zs = CAP_MOUTH_Z - NANO_PRELOAD
    return zs, zs + CLAMP_T


def hook_x0():
    return x_end() + 2.0


def tab_end():
    return hook_x0() - HOOK_GAP


def hook_block():
    """Block past the Nano's far end wall with a lip that the clamp's tab tucks under. The lip's
    45 degree underside presses the tab down HOOK_BITE in service, and prints without supports."""
    _, zt = clamp_z()
    y0, y1 = NY0 - V_WALL, NY0 + NANO_T + V_WALL
    x0 = hook_x0()
    z_ch = zt + NANO_PRELOAD - HOOK_BITE - HOOK_GAP     # lip underside at the block face
    top = z_ch + HOOK_LIP + 1.2
    profile = _poly([(x0 + HOOK_T, PLATE_T - EPS), (x0 + HOOK_T, top), (x0 - HOOK_LIP, top),
                     (x0 - HOOK_LIP, z_ch + HOOK_LIP), (x0, z_ch), (x0, PLATE_T - EPS)])
    return profile.extrude(y1 - y0).rotate((90, 0, 0)).translate((0, y1, 0))


# ---------- branding ----------

def _label(lines=None):
    import json
    out = cad.CrossSection()
    for line in json.loads((HERE / "label.json").read_text()):
        if lines is None or line["text"] in lines:
            out += cad.CrossSection(line["polygons"], cad.FillRule.EvenOdd)
    return out


def mark(size=MARK_SIZE, at=MARK_AT):
    """The CubeSat Classroom mark from docs/mark.svg: a cube with an orbit ring passing in front
    of it and behind it. Flat shapes in plate coordinates, for raised relief."""
    def svg(points):
        s = size / 120.0
        return [(at[0] + (x - 60) * s, at[1] - (y - 60) * s) for x, y in points]

    faces = cad.CrossSection()
    for pts in ([(60.00, 28.85), (85.38, 43.50), (60.00, 58.15), (34.62, 43.50)],
                [(33.02, 46.27), (58.40, 60.92), (58.40, 90.23), (33.02, 75.58)],
                [(61.60, 60.92), (86.98, 46.27), (86.98, 75.58), (61.60, 90.23)]):
        faces += _poly(svg(pts))

    def ring(t_from, t_to, width):
        """The orbit ellipse (rx 53, ry 15, turned -15 degrees in the drawing) as a stroke."""
        pts = []
        for i in range(49):
            t = math.radians(t_from + (t_to - t_from) * i / 48)
            ex, ey = 53 * math.cos(t), 15 * math.sin(t)
            r = math.radians(-15)
            pts.append((60 + ex * math.cos(r) - ey * math.sin(r), 60 + ex * math.sin(r) + ey * math.cos(r)))
        dots = [cad.CrossSection.circle(width / 2 * size / 120, 12).translate(p) for p in svg(pts)]
        return cad.CrossSection.batch_boolean([(d0 + d1).hull() for d0, d1 in zip(dots, dots[1:])], cad.OpType.Add)
    hexagon = _poly(svg([(60.00, 27.00), (88.58, 43.50), (88.58, 76.50), (60.00, 93.00), (31.42, 76.50),
                         (31.42, 43.50)])).offset(2.5 * size / 120, cad.JoinType.Round, circular_segments=12)
    front = ring(0, 180, 4.6)            # the half that passes in front of the cube (svg y down)
    back = ring(180, 360, 4.6) - hexagon
    gap = ring(0, 180, 9.5)
    return (faces - gap) + front + back


def branding():
    """Mark and the words CUBESAT CLASSROOM, for the plates' open middle under the Uno."""
    return mark() + _label(["CUBESAT", "CLASSROOM"])


# ---------- the parts ----------

def bottom_with_columns():
    """AMSAT end piece with standoff columns growing from the face's inner side (frame
    coordinates). Three end in snap pins and one in a locating pin, for the payload plate."""
    part = load("cubesatsim_frame_top_bottom.v2.0.stl")
    for i, (x, y) in enumerate(plate_holes()):
        column = cyl(COLUMN_OD, COLUMN_LENGTH + FACE_T / 2, x, y, -COLUMN_LENGTH - FACE_T / 2)
        column += pin(x, y, 0.0, PLATE_T) if i in SNAP_COLUMNS else locating_pin(x, y, 0.0, PLATE_T)
        part += to_frame(column)
    return part


def plate_cuts(z0, t, tooth_r=TOOTH_R):
    """Holes through a plate (from z0, t thick) for the columns."""
    cuts = cad.Manifold()
    for i, (x, y) in enumerate(plate_holes()):
        if i in SNAP_COLUMNS:
            cuts += finger_cut(x, y, FINGER_A0[i], tooth_r).extrude(t + 2).translate((0, 0, z0 - 1))
        else:
            cuts += cyl(2 * (PIN_R + LOCATE_CLEAR), t + 2, x, y, z0 - 1)
    return cuts


def board_features(tooth_r=TOOTH_R):
    """Everything standing on the plate's board side, without the plate itself."""
    top = PLATE_T
    part = cad.Manifold()

    # Uno R3: four posts with short locating pins that stop below the board's top face.
    pcb_bottom = top + UNO_POST_H
    pin_h = PCB_UNO + UNO_PIN_ABOVE
    for ux, uy in UNO_HOLES:
        x, y = uno_xy(ux, uy)
        part += cyl(7.0, UNO_POST_H - 2.5 + EPS, x, y, top - EPS)
        part += cyl(UNO_POST_TOP, 2.5 + EPS, x, y, pcb_bottom - 2.5 - EPS)
        part += cyl(UNO_PIN, pin_h - 0.4 + 2 * EPS, x, y, pcb_bottom - EPS)
        part += cyl(UNO_PIN, 0.4, x, y, pcb_bottom + pin_h - 0.4, d2=UNO_PIN - 0.6)

    # Uno latch backstops and cheeks; the clips themselves are separate parts.
    for name in LATCHES:
        part += latch_fixed(name)

    # Nano card cage, lower half: V-groove rail, USB-end corner stop, far-end wall.
    y0, y1 = NY0 - V_WALL, NY0 + NANO_T + V_WALL
    part += box(NX0 - 2.0, y0, top - EPS, x_end() + 1.0, y1, BASE_MOUTH_Z)
    part += box(NX0 - 2.0, NY0 - 1.0, BASE_MOUTH_Z - EPS, NX0, NY0 + NANO_T + 1.0, NZ0 + NANO_STOP_REACH)
    part += box(x_end(), y0, top - EPS, x_end() + 2.0, y1, NZ0 + NANO_W - 2.0)
    part -= _groove([(-V_MOUTH / 2, 0), (-V_APEX / 2, -V_DEPTH), (V_APEX / 2, -V_DEPTH), (V_MOUTH / 2, 0),
                     (V_MOUTH / 2, 1), (-V_MOUTH / 2, 1)], NX0 - EPS, x_end(), BASE_MOUTH_Z)

    # Nano clamp: hook block at the far end, snap pillar at the USB end.
    part += hook_block()
    zs, _ = clamp_z()
    ax, ay = NANO_ANCHOR
    part += cyl(PILLAR_OD, zs - top + EPS, ax, ay, top - EPS) + pin(ax, ay, zs, CLAMP_T)  # pillar top is the shoulder
    return part


def payload_plate(tooth_r=TOOTH_R):
    plate = chamfered_square(PLATE, PLATE_CHAMFER, PLATE_T) - plate_cuts(0.0, PLATE_T, tooth_r)
    relief = branding().extrude(LETTER_H + EPS).translate((0, 0, PLATE_T - EPS))
    return plate + board_features() + relief


def lower_clamp(tooth_r=TOOTH_R):
    """Cap rail over the Nano's upper edge. Its far end tucks under the hook block's lip, and
    its near end snaps onto the anchor pillar's pin. One flat top face, so it prints upside down
    without supports. It doesn't touch the Uno, so either board comes out on its own.
    Built resting on the pillar's shoulder; the Nano's edge lifts it NANO_PRELOAD in service."""
    zs, zt = clamp_z()
    ax, ay = NANO_ANCHOR
    rail = box(NX0 - 2.2, NY0 - V_WALL, zs, tab_end(), NY0 + NANO_T + V_WALL, zt)
    pad = cyl(2 * CLAMP_PAD_R, zt - zs, ax, ay, zs)
    part = (rail + pad) ^ chamfered_square(PLATE - 1.0, PLATE_CHAMFER, 100).translate((0, 0, -10))
    stop = box(NX0 - 2.0, NY0 - 1.0, NZ0 + NANO_W - NANO_STOP_REACH, NX0, NY0 + NANO_T + 1.0, zs + EPS)
    part += stop
    part -= _groove([(-V_MOUTH / 2, -1), (V_MOUTH / 2, -1), (V_MOUTH / 2, 0),
                     (V_APEX / 2, V_DEPTH), (-V_APEX / 2, V_DEPTH), (-V_MOUTH / 2, 0)],
                    NX0 - EPS, tab_end() + EPS, zs)
    part -= finger_cut(ax, ay, ANCHOR_A0, tooth_r).extrude(CLAMP_T + 2).translate((0, 0, zs - 1))
    return part


# Fit plate: the payload plate as an open truss, for a quick fit test with far less plastic.
FIT_RIM = 3.0
FIT_PAD = 1.6
TRUSS_PITCH = 34.0
TRUSS_RIB = 2.4
TRUSS_MIN_POCKET = 12.0


def fit_plate():
    """The payload plate's mounts, joints and branding, unchanged and in the same places, on an
    open triangular truss instead of a solid deck."""
    outline = chamfered_square(PLATE, PLATE_CHAMFER, 1).slice(0.5)
    solid = outline - outline.offset(-FIT_RIM, cad.JoinType.Miter)
    features = board_features()
    solid += features.slice(PLATE_T + 0.1).offset(FIT_PAD, cad.JoinType.Round, circular_segments=FACETS)
    for x, y in plate_holes():
        solid += cad.CrossSection.circle(finger_radii()[2] + 1.6, FACETS).translate((x, y))
    solid += branding().offset(1.0, cad.JoinType.Round, circular_segments=16)
    ribs = cad.CrossSection()
    h = TRUSS_PITCH * math.sqrt(3) / 2
    for angle in (0, 60, 120):
        for k in range(-6, 7):
            ribs += cad.CrossSection.square((3 * PLATE, TRUSS_RIB)).translate((-1.5 * PLATE, k * h - TRUSS_RIB / 2)).rotate(angle)
    pockets = (outline - solid - ribs).offset(-1.0, cad.JoinType.Round, circular_segments=16)
    pockets = pockets.offset(1.0, cad.JoinType.Round, circular_segments=16).simplify(0.02)
    pockets = cad.CrossSection([p for p in pockets.to_polygons() if abs(cad.CrossSection([p]).area()) > TRUSS_MIN_POCKET])
    return payload_plate() - pockets.extrude(PLATE_T + 2).translate((0, 0, -1))


# Test coupons: one of each joint on its own, to try a printer before the big parts.
def coupon_pin_base():
    """A column stub with a snap pin, on a small base."""
    base = box(-12, -12, 0, 12, 12, 2.4)
    stub = cyl(COLUMN_OD, 3.0 + EPS, 0, 0, 2.4 - EPS)
    return base + stub + pin(0, 0, 5.4, PLATE_T)


def coupon_pin_tab():
    """A square of plate with the flex fingers, to snap onto the pin."""
    tab = box(-12, -12, 0, 12, 12, PLATE_T)
    return tab - finger_cut(0, 0, 0.0).extrude(PLATE_T + 2).translate((0, 0, -1))


COUPON_POSTS = [(-7.0, -5.0), (-7.0, 5.0)]     # in latch-local u, v: posts under the test board


def coupon_latch_base():
    """One latch's backstop, with two short Uno posts carrying a strip of test board."""
    base = box(-12.0, -9.0, 0, 9.5, 9.0, PLATE_T)
    fixed = latch_fixed("right").translate((-(UNO_ORIGIN[0] + UNO[0]), -(UNO_ORIGIN[1] + LATCHES["right"][1]), 0))
    posts = cad.Manifold()
    pcb_bottom = PLATE_T + UNO_POST_H
    for u, v in COUPON_POSTS:
        posts += cyl(7.0, UNO_POST_H + EPS, u, v, PLATE_T - EPS)
        posts += cyl(UNO_PIN, PCB_UNO + UNO_PIN_ABOVE, u, v, pcb_bottom - EPS)
    return base + fixed + posts


def coupon_board():
    """A strip of 1.6 mm board with its edge where the Uno's would be."""
    strip = box(-12.0, -8.0, 0, 0.0, 8.0, PCB_UNO)
    for u, v in COUPON_POSTS:
        strip -= cyl(3.2, PCB_UNO + 2, u, v, -1)
    return strip


# ---------- checks ----------

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


def verify(bottom, plate, clamp):
    """Assemble the parts and check fit, clearances and retention."""
    keys = [key_in_place(name) for name in LATCHES]
    clips = [latch_clip(name) for name in LATCHES]

    # Snap-fit design rules first.
    require(at_least(SNAP_STRAIN_LIMIT, finger_strain()),
            f"the flex fingers bend {finger_strain():.2%} as a pin head passes, over {SNAP_STRAIN_LIMIT:.1%}")
    require(at_least(SNAP_STRAIN_LIMIT, latch_strain()),
            f"the latch arms bend {latch_strain():.2%} snapping over the Uno, over {SNAP_STRAIN_LIMIT:.1%}")
    require(at_least(SNAP_STRAIN_LIMIT, key_strain()),
            f"the key prongs bend {key_strain():.2%}, over {SNAP_STRAIN_LIMIT:.1%}")
    for label, value in (("finger", FINGER_W), ("latch arm", LATCH_T), ("key prong", (KEY_W - KEY_SLOT) / 2),
                         ("clip foot", FOOT_T - TUNNEL_CLEAR)):
        require(at_least(value, MIN_WALL), f"the {label} is {value:.2f} mm thick, under the {MIN_WALL} mm minimum")
    preload = contact_r() - TOOTH_R
    require(preload >= 0.2, f"the teeth meet a pin head {preload:.2f} mm out from rest; under 0.2 mm, "
            "print error could leave the joint loose")
    require(PIN_HEAD_R - contact_r() >= 0.3, "the teeth sit too near the edge of the pin head to hold")
    need = PAYLOAD_MASS * 9.81 * DESIGN_G * RETENTION_MARGIN
    have = len(SNAP_COLUMNS) * pin_retention(PLATE_T)
    require(have >= need, f"the plate's snap pins hold {have:.1f} N, less than {need:.1f} N "
            f"(payload at {DESIGN_G:.0f} g, times {RETENTION_MARGIN})")
    _, r_out, r_slit = finger_radii()
    root = 180 - FINGER_SPAN - FINGER_GAP
    require(root >= 6.0, f"only {root:.1f} degrees of root left for each finger")
    require(at_least(LATCH_REACH, MIN_LATCH_REACH),
            f"Uno latch hooks reach {LATCH_REACH} mm over the board, less than {MIN_LATCH_REACH} mm")
    free = 2 * KEY_CLEAR
    require(free < (LATCH_REACH - LATCH_PRELOAD) / 2,
            f"with its key in, a latch can still open {free:.2f} mm, more than half its "
            f"{LATCH_REACH - LATCH_PRELOAD:.2f} mm grip, so the key doesn't lock it")
    require(0 < LATCH_PRELOAD < LATCH_REACH / 2, "latch preload must be positive and well inside the hook's reach")
    require(KEY_SLOT >= 2 * KEY_BUMP + 0.2, "the key's slot is too narrow for its prongs to flex past the notches")
    require(foot_end() - _key_slot()[2] >= 2.0, "the clip's foot reaches less than 2 mm into its tunnel")
    require(at_least(NANO_SEAT, MIN_SEAT), f"the Nano's edges sit {NANO_SEAT:.2f} mm into the V-grooves, "
            f"less than {MIN_SEAT} mm")
    lowest = min(y0 for _, y0, _, _, _ in NANO_PARTS.values() if y0 > 0)
    highest = max(y1 for _, _, _, y1, _ in NANO_PARTS.values())
    require(NANO_SEAT + 0.3 < min(lowest, NANO_W - highest),
            "the V-groove walls reach the parts along the Nano's edges")

    # Plate on the columns, in plate coordinates.
    columns = bottom.translate((0, 0, -(FACE_INNER_Z - COLUMN_LENGTH))).rotate((180, 0, 0))
    seated = payload_plate(contact_r() + 0.02)
    require(overlap(seated, columns) < 1e-3, "the payload plate collides with the columns or the end piece")
    require(overlap(plate, columns) > 1e-3, "the flex fingers don't press on the pin heads: the plate could rattle")
    require(overlap(seated.translate((0, 0, 0.3)), columns) > 1e-3, "nothing holds the plate down on its columns")
    spread = payload_plate(PIN_HEAD_R + 0.1)
    for dz in (0.5, 1.5, 2.5, 3.5, 4.5):
        require(overlap(spread.translate((0, 0, dz)), columns) < 1e-3,
                "the plate can't be pushed down past the pin heads with its fingers spread")
    require(to_frame(plate).bounding_box()[5] <= FACE_INNER_Z - SNAP_TABS_REACH,
            "payload plate does not clear the AMSAT snap tabs")

    top = PLATE_T
    features = board_features() - box(-PLATE, -PLATE, -1, PLATE, PLATE, top)
    lifted_clamp = clamp.translate((0, 0, NANO_PRELOAD))      # lifted by the Nano's edge, in service
    clamp_seated = lower_clamp(contact_r() + 0.02).translate((0, 0, NANO_PRELOAD))
    everything = features + clamp_seated
    for solid in keys + clips:
        everything += solid
    for i, (x, y) in enumerate(plate_holes()):
        if i in SNAP_COLUMNS:
            require(overlap(everything, finger_zone(x, y, top, 3.0)) < 1e-3,
                    "a part stands on a plate's flex fingers, so they can't flex")
            head = cyl(2 * PIN_HEAD_R + 0.6, pin_top(0.0, PLATE_T) - top + 0.3, x, y, top)
            require(overlap(everything, head) < 1e-3, "a part sits where a column's pin head comes through")
    require(overlap(everything, outline_prism(top, 80) ^ everything) >= everything.volume() - 1e-2,
            "a part reaches outside the plate outline, into the frame's corner rails")

    # Clamp: snaps onto the anchor, captured by the lip.
    ax, ay = NANO_ANCHOR
    zs, zt = clamp_z()
    resting = lower_clamp(contact_r() + 0.02)                 # on its shoulder, as at the anchor
    require(overlap(resting, features) < 1e-3, "the Nano clamp collides with the plate's parts")
    require(overlap(clamp, features) > 1e-3, "the clamp's fingers don't press on the anchor pin")
    require(overlap(resting.translate((0, 0, 0.3)), features - hook_block()) > 1e-3, "nothing holds the clamp down")
    hook = hook_block()
    require(overlap(lifted_clamp, hook) > 1e-4, "the hook's lip doesn't press on the clamp's tab")
    require(overlap(lower_clamp(contact_r() + 0.02), hook) < 1e-3, "the clamp's tab doesn't fit under the lip")
    tab = box(tab_end() - HOOK_LIP + 0.4, NY0, zs, tab_end(), NY0 + NANO_T, zt).translate((0, 0, NANO_PRELOAD + 0.3))
    require(overlap(tab, hook) > 1e-3, "the clamp's tab could lift out from under the lip")
    spread_clamp = lower_clamp(PIN_HEAD_R + 0.1).translate((0, 0, NANO_PRELOAD))
    for dz in (0.5, 1.5, 2.5, 3.5):
        moved = spread_clamp.translate((0, 0, dz)) - box(hook_x0() - 6, -PLATE, -1, PLATE, PLATE, 90)
        require(overlap(moved, features) < 1e-3, "the clamp can't be pressed down onto the anchor pin")
    require(pin_retention(CLAMP_T) >= 0.03 * 9.81 * DESIGN_G * RETENTION_MARGIN,
            "the clamp's snap pin can't hold the clamp and Nano at the design load")

    # Uno R3: board, underside and parts, with every clip bent to its seated position.
    ux, uy = UNO_ORIGIN
    pcb_bottom = top + UNO_POST_H
    seated_clips = [latch_clip(name, LATCH_PRELOAD + 0.01) for name in LATCHES]
    in_service = features + clamp_seated
    for solid in seated_clips + keys:
        in_service += solid
    for clip in clips:
        require(overlap(clip, features) < 1e-3, "a latch clip doesn't fit its backstop and tunnel")
    uno = box(ux, uy, pcb_bottom, ux + UNO[0], uy + UNO[1], UNO_TOP_Z)
    posts = cad.Manifold()
    for hx, hy in UNO_HOLES:
        uno -= cyl(3.2, PCB_UNO + 2, ux + hx, uy + hy, pcb_bottom - 1)
        posts += cyl(7.0, UNO_POST_H + PCB_UNO + 2, ux + hx, uy + hy, top - 1)
    require(overlap(in_service, uno) < 1e-3, "a clip, the clamp or a part passes through the Uno board")
    underside = box(ux, uy, pcb_bottom - 2.0, ux + UNO[0], uy + UNO[1], pcb_bottom) - posts
    require(overlap(in_service, underside) < 1e-3, "something under the Uno would hit its solder joints")
    for name, body in uno_parts().items():
        require(overlap(in_service, body) < 1e-3, f"a clip, key, clamp or pillar hits the Uno's {name}")
    above_board = box(ux, uy, UNO_TOP_Z, ux + UNO[0], uy + UNO[1], UNO_TOP_Z + 3.0)
    for name, clip in zip(LATCHES, seated_clips):
        held = overlap(clip, above_board) / LATCH_W
        require(held > 0.2, f"the {name} clip's hook doesn't reach over the Uno")
        for key in keys:
            require(overlap(key, latch_clip(name)) < 1e-3, f"a lock key doesn't fit behind the {name} clip")
        opened = latch_clip(name, LATCH_PRELOAD + 2 * KEY_CLEAR + 0.05)
        require(sum(overlap(opened, k) for k in keys) > 1e-3,
                f"the {name} clip can open past its key: the key doesn't lock it")
        require(overlap(latch_clip(name).translate((0, 0, 0.1)), features) > 1e-3,
                f"the {name} clip's foot isn't held down by its tunnel")
    snapping = features + clamp_seated
    for name in LATCHES:
        snapping += latch_clip(name, LATCH_REACH + 0.05)
    path = box(ux, uy, UNO_TOP_Z + 0.01, ux + UNO[0], uy + UNO[1], 80) + box(ux - 7, uy + 30, UNO_TOP_Z + 0.01, ux, uy + 46, 80)
    require(overlap(snapping, path) < 1e-3, "the Uno can't be pushed down past its latch clips")

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
        for half, solid in (("base", features), ("cap", clamp_seated)):
            if dz == 0 and dx == 0:
                require(overlap(solid, moved) > 1e-3, f"the {half} groove does not hold the Nano sideways")
    lift = (box(NX0 + 0.1, NY0 + 0.02, NZ0 + 0.01, NX0 + NANO_L + 0.1, NY0 + NANO_T - 0.02, 90)
            + parts.translate((0.1, 0, 0)) + box(NX0 + 0.1, NY0, NZ0 + EDGE_CLEAR, NX0 + NANO_L, NY0 + NANO_T + 2.8, 90)
            ) - box(-PLATE, -PLATE, -1, PLATE, PLATE, NZ0 + 0.01)
    lift = lift + box(NX0 + 0.1, NY0 + 0.02, NZ0 + NANO_W, NX0 + NANO_L + 0.1, NY0 + NANO_T + 2.8, 90)
    require(overlap(features, lift) < 1e-3, "with the clamp off, the Nano can't be lowered into its groove")
    require(overlap(everything, nano_plug()) < 1e-3, "something blocks the Nano's USB plug")
    room = NX0 + NANO_PARTS["Micro-USB"][0] + PLATE / 2
    require(at_least(room, MIN_PLUG_ROOM), f"only {room:.1f} mm from the Nano's USB to the plate edge, "
            f"less than {MIN_PLUG_ROOM} mm for the plug")

    # Coupons: the same joints fit on their own.
    base, tab = coupon_pin_base(), coupon_pin_tab()
    seated_tab = (box(-12, -12, 0, 12, 12, PLATE_T)
                  - finger_cut(0, 0, 0.0, contact_r() + 0.02).extrude(PLATE_T + 2).translate((0, 0, -1)))
    require(overlap(base, seated_tab.translate((0, 0, 5.4))) < 1e-3, "the snap-pin coupon doesn't fit together")
    require(overlap(base, tab.translate((0, 0, 5.4))) > 1e-3, "the snap-pin coupon's fingers don't press its pin")


def export(name, solid):
    mesh = check(name, solid)
    mesh.apply_translation((0, 0, -mesh.bounds[0, 2]))
    folder = HERE / "stl"
    folder.mkdir(exist_ok=True)
    mesh.export(folder / f"{name}.stl")
    size = [round(float(v), 1) for v in mesh.extents]
    print(f"{name}: {size} mm, {mesh.volume / 1000:.1f} cm3, watertight, {len(mesh.faces)} triangles")
    return mesh


BROKEN_DESIGNS = {
    "plate inside the snap tabs": {"COLUMN_LENGTH": 10.0},
    "flex fingers too thick to bend": {"FINGER_W": 3.0},
    "fingers too short, so they would crack": {"FINGER_SPAN": 120.0},
    "pin head too small to hold the plate": {"PIN_HEAD_R": 3.5},
    "no preload: the plate could rattle": {"PIN_BITE": 0.25},
    "latch hooks too short to hold the Uno": {"LATCH_REACH": 0.4},
    "stiff latch arm that would crack": {"LATCH_T": 3.0},
    "Uno posts too short for gentle latch arms": {"UNO_POST_H": 10.0},
    "lock key too loose to stop a clip opening": {"KEY_CLEAR": 0.3},
    "left latch moved into the USB-B connector": {"LATCHES": {**LATCHES, "left": ("left", 38.0, 1.5)}},
    "V-grooves too shallow to grip the Nano": {"V_DEPTH": 0.4},
    "Nano parts against the groove wall": {"V_DEPTH": 4.0},
    "Nano too close to the plate edge for its USB plug": {"NX0": -26.0},
    "clamp pillar moved into the Nano's USB plug": {"NANO_ANCHOR": (-17.0, -31.0)},
    "Uno moved 3 mm towards the Nano": {"UNO_ORIGIN": (-35.66, -24.34)},
    "lip reaching over the Nano, so it can't go in": {"HOOK_LIP": 3.0},
    "snap pin on the column beside the Nano card cage": {"SNAP_COLUMNS": [0, 1, 2, 3], "FINGER_A0": {0: 100.0, 1: 260.0, 2: 0.0, 3: 0.0}},
}

# Designs exactly at a limit must pass: the limits are inclusive.
BOUNDARY_DESIGNS = {
    "flex fingers exactly at the 1.2% strain limit": {
        "FINGER_SPAN": math.degrees(math.sqrt(1.5 * FINGER_W * (PIN_HEAD_R + 0.1 - TOOTH_R) / SNAP_STRAIN_LIMIT)
                                    / ((2 * (PIN_HEAD_R + FINGER_CLEAR) + FINGER_W) / 2))},
}


def _derived():
    """Recompute module values that depend on parameters a test changes."""
    global UNO_TOP_Z, FOOT_TOP, NANO_SEAT, BASE_MOUTH_Z, CAP_MOUTH_Z, NANO_MID_Y
    UNO_TOP_Z = PLATE_T + UNO_POST_H + PCB_UNO
    FOOT_TOP = PLATE_T + FOOT_T
    NANO_SEAT = V_DEPTH * (V_MOUTH - NANO_T) / (V_MOUTH - V_APEX)
    BASE_MOUTH_Z = NZ0 + NANO_SEAT
    CAP_MOUTH_Z = NZ0 + NANO_W - NANO_SEAT
    NANO_MID_Y = NY0 + NANO_T / 2


def _run(change):
    saved = {name: globals()[name] for name in change}
    globals().update(change)
    _derived()
    try:
        verify(bottom_with_columns(), payload_plate(), lower_clamp())
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
    clamp = lower_clamp()
    verify(bottom, plate, clamp)
    print(f"checks passed: Nano edge in each groove {NANO_SEAT:.2f} mm")
    print(f"snap pins: finger strain {finger_strain():.2%} (limit {SNAP_STRAIN_LIMIT:.1%}), preload "
          f"{contact_r() - TOOTH_R:.2f} mm, {pin_retention(PLATE_T):.1f} N to pull the plate off each pin, "
          f"{len(SNAP_COLUMNS) * pin_retention(PLATE_T):.0f} N in all "
          f"(payload at {DESIGN_G:.0f} g: {PAYLOAD_MASS * 9.81 * DESIGN_G:.0f} N)")
    print(f"Uno latch clips: arm strain {latch_strain():.2%}, hook reach {LATCH_REACH} mm, "
          f"opening with key in {2 * KEY_CLEAR:.2f} mm, preload about {latch_preload():.1f} N each, "
          f"key prong strain {key_strain():.2%}")
    # The end piece prints face down, so its columns point up.
    export("bottom-with-columns", bottom.rotate((180, 0, 0)))
    export("payload-plate", plate)
    export("lower-clamp", clamp.rotate((180, 0, 0)))      # flat top on the bed
    export("latch-clip", latch_clip_flat())
    export("lock-key", key_flat())
    fit = export("fit-plate", fit_plate())
    export("coupon-pin-base", coupon_pin_base())
    export("coupon-pin-tab", coupon_pin_tab())
    export("coupon-latch-base", coupon_latch_base())
    export("coupon-board", coupon_board())
