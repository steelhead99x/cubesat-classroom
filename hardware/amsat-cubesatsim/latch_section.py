"""Draw uno-latch-section.png: a cut through one Uno snap latch, snapping in and locked.

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
name = "right"
ry = g.UNO_ORIGIN[1] + g.LATCHES[name][1] + 2.0   # through one key prong
def section(solid):
    # plane y = ry: rotate so that y becomes z, then slice
    s = solid.translate((0, -ry, 0)).rotate((-90, 0, 0))   # y -> z (approximately), z -> -y... check below
    cs = s.slice(0.0)
    return [np.array(p) for p in cs.to_polygons()]
def draw(ax, solid, color, label=None, alpha=1.0):
    polys = section(solid)
    first = True
    for p in polys:
        x, z = p[:, 0], p[:, 1]
        ax.fill(x, z, color=color, alpha=alpha, lw=0.6, ec="black", label=label if first else None, zorder=3)
        first = False
fig, axes = plt.subplots(1, 2, figsize=(12, 5.6), dpi=120)
plate = g.payload_plate()
for ax, (title, deflect, key, board_z) in zip(axes, [
        ("1. Push the Uno down: the hook rides over the edge", g.LATCH_REACH, False, g.UNO_TOP_Z + 0.35),
        ("2. Seated, lock key in: the latch can't open", g.LATCH_PRELOAD, True, g.UNO_TOP_Z)]):
    fixed = plate - g.latch(name)
    window = g.box(26, ry - 30, 0, 46, ry + 30, 30)
    draw(ax, (fixed ^ window), "#c9ced6", "plate, post, backstop")
    draw(ax, g.latch(name, deflect) ^ window, "#8d96a3", "latch arm and hook")
    pcb = g.box(g.UNO_ORIGIN[0], ry - 5, board_z - g.PCB_UNO, g.UNO_ORIGIN[0] + g.UNO[0], ry + 5, board_z)
    draw(ax, pcb ^ window, "#2f6db5", "Uno R3 board")
    if key:
        draw(ax, g.key_in_place(name), "#d7261e", "lock key")
    ax.set_xlim(27, 47); ax.set_ylim(0, 23); ax.set_aspect("equal"); ax.grid(alpha=0.25)
    ax.set_title(title, fontsize=10); ax.set_xlabel("mm")
handles, labels = axes[1].get_legend_handles_labels()
axes[0].legend(handles, labels, loc="upper left", fontsize=7.5)
axes[1].annotate(f"{g.LATCH_ANGLE:.0f}° hook underside:\nspring force presses\nthe board down (~{g.latch_preload():.0f} N)",
                 xy=(32.6, 14.75), xytext=(27.6, 19.0), fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
axes[1].annotate("key leaves 0.2 mm:\nthe hook keeps 0.45 mm\non the board", xy=(36.0, 9), xytext=(40.0, 11.0),
                 fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
axes[0].annotate(f"arm bends {g.LATCH_REACH} mm\n({g.latch_strain():.1%} strain, PLA limit 2%)", xy=(34.8, 14), xytext=(38.0, 19.0),
                 fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
fig.suptitle("Uno snap latch with lock key — section through the right-hand latch (cut through one key prong)", fontsize=11)
plt.tight_layout()
plt.savefig(HERE / "uno-latch-section.png")
