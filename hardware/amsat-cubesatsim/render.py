"""Render the AMSAT kit with the Arduino payload as an exploded view for the website.

Writes render-exploded.png (transparent, for light pages), render-exploded-dark.png
(transparent, for dark pages), render-exploded-navy.png (on the mission-briefing Navy,
for link previews) and render-assembled.png. Run generate.py first.
Needs Pillow on top of hardware/requirements.txt. GPL-3.0, like the rest of this folder.

The parts are placed in the frame coordinates generate.py uses (end-piece face at z = 0,
frame interior towards -z), then turned over so the bottom piece is at the bottom and
pulled apart along the CubeSat's long axis. The Uno and Nano are simple boxes.
"""

import sys
from pathlib import Path

import numpy as np
import trimesh
from PIL import Image, ImageDraw

import generate as g

HERE = Path(__file__).resolve().parent

# CubeSat Classroom brand colours (site/src/theme.css).
NAVY, DEEP, ORBIT, FLAG = "#01017f", "#00005c", "#2f3fbf", "#ea0303"
FRAME, PLATE, BOARD, PARTS, STRAP = "#eef0fa", ORBIT, "#1a2a9a", "#c9cdea", FLAG

FRAME_H = 113.5


def rot_x180(m):
    return m.apply_transform(trimesh.transformations.rotation_matrix(np.pi, [1, 0, 0]))


def rot_z180(m):
    return m.apply_transform(trimesh.transformations.rotation_matrix(np.pi, [0, 0, 1]))


def box(x0, y0, z0, x1, y1, z1):
    b = trimesh.creation.box(extents=[x1 - x0, y1 - y0, z1 - z0])
    b.apply_translation([(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2])
    return b


def stl(folder, name):
    return trimesh.load_mesh(HERE / folder / name)


def boards():
    """Uno R3 and Nano 33 BLE Sense in plate coordinates (plate bottom at z = 0)."""
    top = g.PLATE_T
    ux, uy = g.UNO_ORIGIN
    z = top + g.UNO_POST_H
    uno = [(box(ux, uy, z, ux + g.UNO[0], uy + g.UNO[1], z + g.PCB_UNO), BOARD)]
    z1 = z + g.PCB_UNO
    uno += [
        (box(ux - 6.2, uy + 38.0, z1, ux + 9.8, uy + 50.0, z1 + 11.0), PARTS),        # USB-B
        (box(ux - 1.8, uy + 3.3, z1, ux + 12.2, uy + 12.3, z1 + 11.0), "#2b2f45"),    # power jack
        (box(ux + 17.0, uy + g.UNO[1] - 3.0, z1, ux + 64.0, uy + g.UNO[1] - 0.5, z1 + 8.5), "#2b2f45"),
        (box(ux + 26.0, uy + 0.5, z1, ux + 64.0, uy + 3.0, z1 + 8.5), "#2b2f45"),
        (box(ux + 31.0, uy + 20.0, z1, ux + 49.0, uy + 28.0, z1 + 2.0), "#2b2f45"),   # processor
    ]
    nx, ny = g.NANO_CENTER
    rail = top + g.NANO_RAIL_H
    nano = [(box(nx - g.NANO[0] / 2, ny - g.NANO[1] / 2, rail, nx + g.NANO[0] / 2, ny + g.NANO[1] / 2,
                 rail + g.PCB_NANO), BOARD)]
    for side in (-1, 1):
        row = ny + side * g.NANO_ROW / 2
        nano.append((box(nx - 19.1, row - 1.3, rail - 2.5, nx + 19.1, row + 1.3, rail), "#2b2f45"))
    nano.append((box(nx - g.NANO[0] / 2 - 1.0, ny - 4.0, rail + g.PCB_NANO, nx - g.NANO[0] / 2 + 5.0, ny + 4.0,
                     rail + g.PCB_NANO + 2.8), PARTS))                                # micro USB
    return uno + nano


def kit(spread, front=1.25, back=1.25, turn_kit=0.0, plate=PLATE):
    """Every part, positioned and exploded, as (mesh, colour) pairs."""
    parts = []

    def place(mesh, colour, lift):
        rot_x180(mesh)                         # bottom piece at the bottom, boards facing up
        mesh.apply_translation([0, 0, lift])
        parts.append((mesh, colour))

    bottom = rot_x180(stl("stl", "bottom-with-columns.stl"))   # exported face down, as printed
    bottom.apply_translation([0, 0, g.FACE_T])                   # face back to z = 0..3
    place(bottom, FRAME, 0)

    def seat_on_columns(mesh):
        rot_x180(mesh)
        mesh.apply_translation([0, 0, g.FACE_INNER_Z - g.COLUMN_LENGTH])
        return mesh

    plate_lift = spread * 0.55
    place(seat_on_columns(stl("stl", "payload-plate.stl")), plate, plate_lift)
    for mesh, colour in boards():
        place(seat_on_columns(mesh), colour, plate_lift + spread * 0.25)
    nx, ny = g.NANO_CENTER
    strap = rot_x180(stl("stl", "nano-strap.stl"))
    strap.apply_translation([nx, ny, g.PLATE_T + g.NANO_RAIL_H + g.PCB_NANO + g.STRAP_ARCH + g.STRAP_T])
    place(seat_on_columns(strap), STRAP, plate_lift + spread * 0.42)

    # AMSAT side panels: printed face down; the face becomes a frame wall, the long axis the frame's.
    for name, turn in (("cubesatsim_frame_side.v2.0.stl", False), ("cubesatsim_frame_side_camera.v2.0.stl", True)):
        side = stl("upstream", name)
        lo, hi = side.bounds
        side.apply_transform(np.array([[1, 0, 0, -(lo[0] + hi[0]) / 2],
                                       [0, 0, 1, -50.0],
                                       [0, 1, 0, g.FACE_T - hi[1]],
                                       [0, 0, 0, 1]], float))
        if turn:
            rot_z180(side)
        rot_x180(side)
        offset = back if turn else -front
        side.apply_translation([0, offset * spread, spread * 0.95])
        parts.append((side, FRAME))

    top = stl("upstream", "cubesatsim_frame_top_bottom.v2.0.stl")
    rot_x180(top)
    top.apply_translation([0, 0, -(FRAME_H - 2 * g.FACE_T)])    # far end: 113.5 mm face to face
    place(top, FRAME, spread * 1.9)
    if turn_kit:
        spin = trimesh.transformations.rotation_matrix(np.radians(turn_kit), [0, 0, 1])
        for mesh, _ in parts:
            mesh.apply_transform(spin)
    return parts


def shade(colour, normal, light):
    rgb = np.array([int(colour[i:i + 2], 16) for i in (1, 3, 5)], float)
    lam = max(0.0, float(normal @ light))
    rim = max(0.0, float(normal @ np.array([-0.4, 0.2, -0.9])))
    k = 0.52 + 0.48 * lam + 0.08 * rim
    out = rgb * k
    out = out + (np.array([1, 1, 92], float) - out) * 0.10 * (1 - lam)   # cool the shadows toward Deep
    return tuple(int(v) for v in np.clip(out, 0, 255))


def render(parts, size, background, scale=3, azim=-25.0, elev=22.0, margin=0.06):
    a, e = np.radians(azim), np.radians(elev)
    # View: turn about z (azimuth), then tilt (elevation). Camera looks along -y_view.
    rz = np.array([[np.cos(a), -np.sin(a), 0], [np.sin(a), np.cos(a), 0], [0, 0, 1]])
    rx = np.array([[1, 0, 0], [0, np.cos(e), -np.sin(e)], [0, np.sin(e), np.cos(e)]])
    view = rx @ rz
    light = view @ np.array([0.35, -0.55, 0.75])
    light /= np.linalg.norm(light)

    tris, cols = [], []
    for mesh, colour in parts:
        v = mesh.vertices @ view.T
        f = mesh.faces
        n = mesh.face_normals @ view.T
        keep = n[:, 1] < 0.02                  # back-face cull: normals facing the camera (-y)
        for tri, nn in zip(v[f[keep]], n[keep]):
            tris.append(tri)
            cols.append(shade(colour, nn, light))
    tris = np.array(tris)
    depth = tris[:, :, 1].mean(axis=1)
    order = np.argsort(-depth)                 # far (large y) first

    xy = tris[:, :, [0, 2]]
    lo, hi = xy.reshape(-1, 2).min(0), xy.reshape(-1, 2).max(0)
    w, h = size[0] * scale, size[1] * scale
    s = min(w * (1 - 2 * margin) / (hi[0] - lo[0]), h * (1 - 2 * margin) / (hi[1] - lo[1]))
    off = np.array([w, h]) / 2 - (lo + hi) / 2 * s * np.array([1, -1])

    img = Image.new("RGBA", (w, h), background)
    draw = ImageDraw.Draw(img)
    for i in order:
        p = xy[i] * s * np.array([1, -1]) + off
        draw.polygon([tuple(q) for q in p], fill=cols[i], outline=cols[i])
    return img.resize(size, Image.LANCZOS)


def main():
    exploded = dict(spread=60.0, front=2.0, back=2.0, turn_kit=90.0)
    render(kit(**exploded), (1600, 1200), (0, 0, 0, 0)).save(HERE / "render-exploded.png", optimize=True)
    # On Navy the Orbit-blue plate would vanish, so it gets the light brand blue.
    dark = kit(**exploded, plate="#9ea6dc")
    render(dark, (1600, 1200), (0, 0, 0, 0)).save(HERE / "render-exploded-dark.png", optimize=True)
    navy = render(dark, (1200, 630), NAVY, margin=0.07)
    navy.convert("RGB").save(HERE / "render-exploded-navy.png", optimize=True)
    # Assembled: the frame closed around the payload, seen from the camera side.
    render(kit(spread=0.0, turn_kit=90.0), (1200, 1200), (0, 0, 0, 0), azim=-35.0, elev=20.0).save(
        HERE / "render-assembled.png", optimize=True)
    print("wrote render-exploded.png, render-exploded-dark.png, render-exploded-navy.png and render-assembled.png")


if __name__ == "__main__":
    sys.exit(main())
