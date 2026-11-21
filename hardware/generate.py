"""Generate the Rev A enclosure and Rev B printed rail-fit set. Dimensions in mm."""

from pathlib import Path

import manifold3d as cad
import numpy as np
import trimesh

WIDTH = 100.0
LENGTH = 113.5
CORNER_CLIP = 3.0
WALL = 3.0
FLOOR = 4.0
LID = 4.0
LIP_HEIGHT = 2.0
LIP_CLEARANCE_PER_SIDE = 0.35
BOSS_INSET = 6.0
BOSS_SIDE = 8.0
M3_CLEARANCE = 3.5
M3_PILOT = 2.8
PILOT_DEPTH = 10.0
CAMERA_APERTURE = 18.0
SERVICE_SLOT_WIDTH = 30.0
SERVICE_SLOT_DEPTH = 18.0
COUPON_DIAMETERS = (2.6, 2.8, 3.0, 3.2, 3.5, 3.8)
COUPON_PITCH = 8.0
FACETS = 64
EPS = 0.1

# Rev B bench geometry. The outside reference follows nominal 1U dimensions.
RAIL_WIDTH = 9.0
SIDE_RECESS = 3.0
END_RECESS = 3.0
FACE_THICKNESS = 2.4
FACE_CLEARANCE = 0.35
LOCATOR_LENGTH = 18.0
LOCATOR_DEPTH = 2.5
RAIL_COUPON_WIDTH = 26.0
RAIL_COUPON_LENGTH = 20.0


def profile(inset, height):
    b = WIDTH - inset
    a = inset
    c = CORNER_CLIP
    points = [(a+c, a), (b-c, a), (b, a+c), (b, b-c),
              (b-c, b), (a+c, b), (a, b-c), (a, a+c)]
    return cad.CrossSection([points]).extrude(height)


def bore(diameter, depth, x, y, z):
    return cad.Manifold.cylinder(depth, diameter / 2, circular_segments=FACETS).translate((x, y, z))


def positions():
    d = BOSS_INSET
    return [(x, y) for x in (d, WIDTH-d) for y in (d, WIDTH-d)]


def shell():
    height = LENGTH - LID
    body = profile(0, height) - profile(WALL, height).translate((0, 0, FLOOR))
    # Corner columns engage the wall and support a short thread at the open rim.
    for x, y in positions():
        boss = cad.Manifold.cube((BOSS_SIDE, BOSS_SIDE, height - FLOOR + EPS))
        body += boss.translate((x-BOSS_SIDE/2, y-BOSS_SIDE/2, FLOOR-EPS))
        body -= bore(M3_PILOT, PILOT_DEPTH + EPS, x, y, height-PILOT_DEPTH)
    # The cable slot opens at the top. The other faces remain available for fit studies.
    slot = cad.Manifold.cube((SERVICE_SLOT_WIDTH, WALL + 2*EPS, SERVICE_SLOT_DEPTH + EPS))
    return body - slot.translate(((WIDTH-SERVICE_SLOT_WIDTH)/2, -EPS, height-SERVICE_SLOT_DEPTH))


def lid():
    # Print exterior face down. Small mid-side alignment tabs avoid the corner bosses.
    top = profile(0, LID)
    tab_w, tab_d = 18.0, 3.0
    edge = WALL + LIP_CLEARANCE_PER_SIDE
    mid = (WIDTH - tab_w) / 2
    for x, y, dx, dy in [(mid, edge, tab_w, tab_d), (mid, WIDTH-edge-tab_d, tab_w, tab_d),
                         (edge, mid, tab_d, tab_w), (WIDTH-edge-tab_d, mid, tab_d, tab_w)]:
        top += cad.Manifold.cube((dx, dy, LIP_HEIGHT+EPS)).translate((x, y, LID-EPS))
    for x, y in positions():
        top -= bore(M3_CLEARANCE, LID+2*EPS, x, y, -EPS)
    return top - bore(CAMERA_APERTURE, LID+2*EPS, WIDTH/2, WIDTH/2, -EPS)


def coupon():
    width = COUPON_PITCH * len(COUPON_DIAMETERS)
    depth = 14.0
    height = PILOT_DEPTH
    part = cad.Manifold.cube((width, depth, height))
    for i, diameter in enumerate(COUPON_DIAMETERS):
        x = COUPON_PITCH * (i+0.5)
        part -= bore(diameter, height+2*EPS, x, depth/2, -EPS)
    # A vertical key at the 2.6 mm end makes the hole order unambiguous.
    return part - bore(4.0, height+2*EPS, 0, 0, -EPS)


def rail_frame():
    """Keep the four corner rails continuous, with four recessed side walls."""
    wall_start = END_RECESS + FACE_THICKNESS
    wall_height = LENGTH - 2 * wall_start
    outer_width = WIDTH - 2 * SIDE_RECESS
    inner_width = outer_width - 2 * WALL
    outer = cad.Manifold.cube((outer_width, outer_width, wall_height))
    cavity = cad.Manifold.cube((inner_width, inner_width, wall_height + 2*EPS))
    walls = (outer - cavity.translate((WALL, WALL, -EPS))).translate(
        (SIDE_RECESS, SIDE_RECESS, wall_start)
    )
    for x in (0, WIDTH - RAIL_WIDTH):
        for y in (0, WIDTH - RAIL_WIDTH):
            rail = cad.Manifold.cube((RAIL_WIDTH, RAIL_WIDTH, LENGTH))
            walls += rail.translate((x, y, 0))
    return walls


def rail_face(camera):
    """The removable plate rests on the end rim and locates inside the walls."""
    inset = SIDE_RECESS + FACE_CLEARANCE
    width = WIDTH - 2 * inset
    clip = 2 * (RAIL_WIDTH - SIDE_RECESS)
    corners = [(clip, 0), (width-clip, 0), (width, clip), (width, width-clip),
               (width-clip, width), (clip, width), (0, width-clip), (0, clip)]
    face = cad.CrossSection([corners]).extrude(FACE_THICKNESS)
    edge = WALL
    center = (width - LOCATOR_LENGTH) / 2
    for x, y, dx, dy in [
        (center, edge, LOCATOR_LENGTH, LOCATOR_DEPTH),
        (center, width-edge-LOCATOR_DEPTH, LOCATOR_LENGTH, LOCATOR_DEPTH),
        (edge, center, LOCATOR_DEPTH, LOCATOR_LENGTH),
        (width-edge-LOCATOR_DEPTH, center, LOCATOR_DEPTH, LOCATOR_LENGTH),
    ]:
        tab = cad.Manifold.cube((dx, dy, LIP_HEIGHT + EPS))
        face += tab.translate((x, y, FACE_THICKNESS - EPS))
    if camera:
        face -= bore(CAMERA_APERTURE, FACE_THICKNESS + 2*EPS, width/2, width/2, -EPS)
    return face


def verify_rail_set(frame, base, camera):
    inset = SIDE_RECESS + FACE_CLEARANCE
    upper = camera.rotate((180, 0, 0)).translate((inset, WIDTH-inset, LENGTH - END_RECESS))
    lower = base.translate((inset, inset, END_RECESS))
    for face in (upper, lower):
        if not np.isclose((frame ^ face).volume(), 0, atol=1e-5):
            raise ValueError("rail-fit plate collides with the frame")
    if not np.allclose((frame + upper + lower).bounding_box(), (0, 0, 0, WIDTH, WIDTH, LENGTH)):
        raise ValueError("rail-fit assembly exceeds the nominal 1U envelope")


def export(name, solid, expected):
    if solid.status() != cad.Error.NoError:
        raise ValueError(f"{name} has a CAD error: {solid.status()}")
    data = solid.to_mesh()
    mesh = trimesh.Trimesh(data.vert_properties[:, :3], data.tri_verts)
    if not (mesh.is_watertight and mesh.is_winding_consistent and mesh.is_volume
            and len(solid.decompose()) == 1
            and np.allclose(mesh.extents, expected) and np.isclose(mesh.bounds[0, 2], 0)):
        raise ValueError(f"{name} failed mesh or size checks: {mesh.bounds}")
    folder = Path(__file__).parent / "stl"
    folder.mkdir(exist_ok=True)
    mesh.export(folder / f"{name}.stl")
    print(f"{name}: {mesh.extents.tolist()} mm, watertight, {len(mesh.faces)} triangles")


if __name__ == "__main__":
    body = shell()
    cover = lid()
    seated = cover.rotate((180, 0, 0)).translate((0, WIDTH, LENGTH))
    if not np.isclose((body ^ seated).volume(), 0, atol=1e-5):
        raise ValueError("seated lid interferes with the shell")
    export("bench-shell", body, (WIDTH, WIDTH, LENGTH-LID))
    export("camera-lid", cover, (WIDTH, WIDTH, LID+LIP_HEIGHT))
    export("m3-fit-coupon", coupon(), (COUPON_PITCH*len(COUPON_DIAMETERS), 14, PILOT_DEPTH))

    frame = rail_frame()
    base = rail_face(camera=False)
    camera = rail_face(camera=True)
    verify_rail_set(frame, base, camera)
    corner = frame ^ cad.Manifold.cube((RAIL_COUPON_WIDTH, RAIL_COUPON_WIDTH, RAIL_COUPON_LENGTH))
    face_width = WIDTH - 2 * (SIDE_RECESS + FACE_CLEARANCE)
    export("rev-b-rail-corner-coupon", corner, (RAIL_COUPON_WIDTH, RAIL_COUPON_WIDTH, RAIL_COUPON_LENGTH))
    export("rev-b-rail-frame", frame, (WIDTH, WIDTH, LENGTH))
    export("rev-b-base-plate", base, (face_width, face_width, FACE_THICKNESS + LIP_HEIGHT))
    export("rev-b-camera-plate", camera, (face_width, face_width, FACE_THICKNESS + LIP_HEIGHT))
