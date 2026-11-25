"""Draw payload-plate-layout.png: the plate from the board side, with board outlines.

Optional, for the README. Needs matplotlib on top of hardware/requirements.txt.
Run generate.py first. GPL-3.0, like the rest of this folder.
"""

from pathlib import Path

import matplotlib
import numpy as np
import trimesh

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.patches import Circle, Rectangle  # noqa: E402

import generate as g  # noqa: E402

HERE = Path(__file__).resolve().parent
# Section heights through the plate, lightest (the plate) to darkest (the tallest parts).
LEVELS = [(1.5, "#d9dde3"), (8.0, "#b8bec7"), (14.0, "#8f98a3"), (16.5, "#6b7480"),
          (22.0, "#4a525c"), (28.0, "#2f353c")]


def fill_section(ax, mesh, z, colour):
    section = mesh.section(plane_origin=[0, 0, z], plane_normal=[0, 0, 1])
    if section is None:
        return
    flat, to_3d = section.to_2D()
    for poly in flat.polygons_full:
        for ring, c in [(poly.exterior, colour)] + [(r, "white") for r in poly.interiors]:
            xy = np.array(ring.coords)
            pts = trimesh.transform_points(np.c_[xy, np.zeros(len(xy))], to_3d)[:, :2]
            ax.fill(pts[:, 0], pts[:, 1], color=c, zorder=z if c != "white" else z + 0.01)


def main():
    plate = trimesh.load_mesh(HERE / "stl" / "payload-plate.stl")
    fig, ax = plt.subplots(figsize=(8, 8), dpi=110)
    for z, colour in LEVELS:
        fill_section(ax, plate, z, colour)

    ux, uy = g.UNO_ORIGIN
    ax.add_patch(Rectangle((ux, uy), *g.UNO, fill=False, ec="#1f6fb2", lw=2, zorder=50, label="Uno R3"))
    ax.add_patch(Rectangle((ux - 6.2, uy + 31.8), 16.2, 12.1, fill=False, ec="#1f6fb2", ls="--",
                           zorder=50, label="Uno USB-B"))
    ax.add_patch(Rectangle((ux - 1.9, uy + 3.3), 13.7, 8.9, fill=False, ec="#1f6fb2", ls=":",
                           zorder=50, label="Uno DC jack"))
    nx, ny = g.NANO_CENTER
    ax.add_patch(Rectangle((nx - g.NANO[0] / 2, ny - g.NANO[1] / 2), *g.NANO, fill=False,
                           ec="#0b8f8a", lw=2, zorder=50, label="Nano 33 BLE Sense"))
    for side in (-1, 1):
        ax.plot([nx - 19, nx + 19], [ny + side * g.NANO_ROW / 2] * 2, color="#0b8f8a", ls=":", zorder=50)
    for x, y in g.plate_holes():
        ax.add_patch(Circle((x, y), g.COLUMN_OD / 2, fill=False, ec="#c9382f", lw=1.5, zorder=50))
    ax.plot([], [], color="#c9382f", label="snap columns from the AMSAT bottom")

    ax.set_xlim(-50, 50)
    ax.set_ylim(-50, 50)
    ax.set_aspect("equal")
    ax.grid(alpha=0.3)
    ax.legend(loc="upper center", bbox_to_anchor=(0.5, -0.05), ncol=2, fontsize=8)
    ax.set_title("Payload plate, board side up (darker = taller), mm")
    plt.tight_layout()
    plt.savefig(HERE / "payload-plate-layout.png")


if __name__ == "__main__":
    main()
