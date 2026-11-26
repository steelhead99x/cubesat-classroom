"""Draw quarter-turn-section.png: a cut through one plate fastener, pushed in and locked.

Optional, for the README. Needs matplotlib on top of hardware/requirements.txt.
GPL-3.0, like the rest of this folder.
"""

from pathlib import Path

import matplotlib
import numpy as np

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

import generate as g  # noqa: E402

HERE = Path(__file__).resolve().parent
X, Y = g.STACK_HOLES[2]          # (29.0, -38.99): a plate corner, frame coordinates


def section(solid):
    """Cut on the plane y = Y; returns polygons in (x, z)."""
    cs = solid.translate((0, -Y, 0)).rotate((-90, 0, 0)).slice(0.0)
    return [np.array(p) for p in cs.to_polygons()]


def draw(ax, solid, colour, label):
    for i, p in enumerate(section(solid)):
        ax.fill(p[:, 0], p[:, 1], color=colour, lw=0.6, ec="black", label=label if i == 0 else None, zorder=3)


def washer(z_hub_drop):
    """Washer at rest, feet on the face; its hub pushed down by z_hub_drop."""
    hub_r, _, _, _ = g.washer_radii()
    w = g.qt_washer().mirror((0, 0, 1)).translate((X, Y, g.FACE_INNER_Z + g.FACE_T + g.WASHER_H))
    hub = g.cyl(2 * hub_r, g.WASHER_H + 1, X, Y, g.FACE_INNER_Z + g.FACE_T) - g.cyl(g.WASHER_ID, 20, X, Y, -5)
    return (w - hub) + (w ^ hub).translate((0, 0, -z_hub_drop))


def main():
    bottom = g.bottom_with_columns()
    plate = g.payload_plate().rotate((180, 0, 0)).translate((0, 0, g.FACE_INNER_Z - g.COLUMN_LENGTH))
    window = g.box(X - 11, Y - 11, -24, X + 11, Y + 11, 12)
    z_head = g.FACE_INNER_Z + g.FACE_T + g.WASHER_H
    a0 = g.plate_stud_a0()
    fig, axes = plt.subplots(1, 2, figsize=(12, 7.5), dpi=120)
    for ax, phi, title in [(axes[0], 0.0, "1. Push the stud in: its slot lines up with the cross-bar"),
                           (axes[1], 90.0, "2. Turn 90° clockwise: the cam draws it in and clicks")]:
        draw(ax, bottom ^ window, "#c9ced6", "bottom piece: face and hollow column")
        draw(ax, plate ^ window, "#8d96a3", "payload plate with cross-bar")
        draw(ax, washer(g.qt_drop(phi)), "#e39b3b", "spring washer")
        draw(ax, g.placed_stud(a0, X, Y, z_head, phi), "#d7261e", "quarter-turn stud")
        ax.set_xlim(X - 11, X + 11)
        ax.set_ylim(-27, 11)
        ax.set_aspect("equal")
        ax.grid(alpha=0.25)
        ax.set_title(title, fontsize=10)
        ax.set_xlabel("mm")
    axes[0].text(X + 0.6, -21.0, "the slot lies in this cut,\nso the stud's two tip\nprongs are hidden", fontsize=7.5)
    handles, labels = axes[0].get_legend_handles_labels()
    axes[0].legend(handles, labels, loc="lower left", fontsize=7.5)
    axes[1].annotate(f"washer hub pressed {g.QT_LIFT} mm:\n{g.washer_force(g.QT_LIFT):.1f} N per stud "
                     f"holds the plate\ndown on its column", xy=(X + 3.6, 5.2), xytext=(X - 10.6, 8.2),
                     fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
    axes[1].annotate(f"cam bears on the bar; a {g.QT_DET} mm detent\nmeans it must be pushed in "
                     "to turn back", xy=(X + 2.0, -16.6), xytext=(X - 10.6, -25.6),
                     fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
    fig.suptitle("Quarter-turn fastener, plate to frame: section through one corner "
                 "(frame upright, outside face at the top)", fontsize=11)
    plt.tight_layout()
    plt.savefig(HERE / "quarter-turn-section.png")


if __name__ == "__main__":
    main()
