"""Draw snap-pin-section.png: a cut through one plate snap pin, pushing on and clicked.

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
COLUMN = 0
X, Y = g.plate_holes()[COLUMN]
TOOTH = g.FINGER_A0[COLUMN] + g.FINGER_SPAN - g.TOOTH_SPAN / 2      # cut through both teeth


def section(solid):
    """Cut along the line through the pin and its teeth: (distance along it, height)."""
    s = solid.translate((-X, -Y, 0)).rotate((0, 0, -TOOTH)).rotate((-90, 0, 0))
    return [np.array(p) for p in s.slice(0.0).to_polygons()]


def draw(ax, solid, color, label=None):
    for i, p in enumerate(section(solid)):
        ax.fill(p[:, 0], p[:, 1], color=color, lw=0.6, ec="black", label=label if i == 0 else None, zorder=3)


def main():
    columns = g.bottom_with_columns().translate((0, 0, g.COLUMN_LENGTH)).rotate((180, 0, 0))
    window = g.box(X - 12, Y - 12, -6, X + 12, Y + 12, 10)
    fig, axes = plt.subplots(1, 2, figsize=(12, 5.2), dpi=120)
    for ax, (title, lift, tooth_r) in zip(axes, [
            ("1. Push the plate down: the head spreads the two teeth", 1.6, g.PIN_HEAD_R + 0.1),
            ("2. Click: the teeth spring in under the 45° face", 0.0, g.contact_r())]):
        draw(ax, columns ^ window, "#8d96a3", "column and snap pin (bottom piece)")
        plate = g.payload_plate(tooth_r).translate((0, 0, lift))
        draw(ax, plate ^ window.translate((0, 0, lift)), "#c9ced6", "payload plate and its flex fingers")
        ax.set_xlim(-11, 11); ax.set_ylim(-5, 8); ax.set_aspect("equal"); ax.grid(alpha=0.25)
        ax.set_title(title, fontsize=10); ax.set_xlabel("mm")
    axes[0].legend(loc="lower left", fontsize=7.5)
    axes[1].annotate(f"tooth pushed {g.contact_r() - g.TOOTH_R:.2f} mm out from rest:\nthe 45° face pulls the plate down,\n"
                     "whatever the print error", xy=(g.contact_r(), g.PLATE_T), xytext=(3.5, 6.0), fontsize=7.5,
                     arrowprops=dict(arrowstyle="->", lw=0.8))
    axes[0].annotate(f"each finger bends {g.finger_pass():.1f} mm\n({g.finger_strain():.2%} strain, limit "
                     f"{g.SNAP_STRAIN_LIMIT:.1%}),\nin the plate's own plane", xy=(g.PIN_HEAD_R + 0.1, 4.6),
                     xytext=(2.0, 6.4), fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
    fig.suptitle("Snap pin, plate to frame: section through one column, its pin head and both teeth", fontsize=11)
    plt.tight_layout()
    plt.savefig(HERE / "snap-pin-section.png")


if __name__ == "__main__":
    main()
