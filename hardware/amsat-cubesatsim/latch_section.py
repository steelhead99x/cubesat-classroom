"""Draw uno-latch-section.png: a cut through one Uno latch clip, snapping in and locked.

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
fig, axes = plt.subplots(1, 2, figsize=(12, 6.4), dpi=120)
features = g.board_features()
slab = g.box(26, ry - 30, 0, 46, ry + 30, g.PLATE_T)
for ax, (title, deflect, key, board_z) in zip(axes, [
        ("1. Push the Uno down: the hook rides over the edge", g.LATCH_REACH, False, g.UNO_TOP_Z + 0.35),
        ("2. Seated, lock key in: the clip can't open", g.LATCH_PRELOAD, True, g.UNO_TOP_Z)]):
    window = g.box(26, ry - 30, 0, 46, ry + 30, 32)
    draw(ax, (features + slab) ^ window, "#c9ced6", "plate, post, backstop with tunnel")
    draw(ax, g.latch_clip(name, deflect) ^ window, "#8d96a3", "latch clip (printed lying flat)")
    pcb = g.box(g.UNO_ORIGIN[0], ry - 5, board_z - g.PCB_UNO, g.UNO_ORIGIN[0] + g.UNO[0], ry + 5, board_z)
    draw(ax, pcb ^ window, "#2f6db5", "Uno R3 board")
    if key:
        draw(ax, g.key_in_place(name), "#d7261e", "lock key")
    ax.set_xlim(27, 44); ax.set_ylim(0, 30); ax.set_aspect("equal"); ax.grid(alpha=0.25)
    ax.set_title(title, fontsize=10); ax.set_xlabel("mm")
handles, labels = axes[1].get_legend_handles_labels()
axes[0].legend(handles, labels, loc="upper left", fontsize=7.5)
edge = g.UNO_ORIGIN[0] + g.UNO[0]
axes[1].annotate(f"{g.LATCH_ANGLE:.0f}° hook underside:\nspring force presses\nthe board down (~{g.latch_preload():.0f} N)",
                 xy=(edge, g.UNO_TOP_Z + 0.2), xytext=(27.6, 25.5), fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
axes[1].annotate(f"key leaves {2 * g.KEY_CLEAR:.1f} mm:\nthe hook keeps {g.LATCH_REACH - g.LATCH_PRELOAD - 2 * g.KEY_CLEAR:.2f} mm\non the board",
                 xy=(edge + 3.5, 12), xytext=(38.5, 14.0), fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
axes[0].annotate(f"arm bends {g.LATCH_REACH} mm in its print plane\n({g.latch_strain():.2%} strain, limit {g.SNAP_STRAIN_LIMIT:.1%})",
                 xy=(edge + 1.5, 15), xytext=(35.0, 26.0), fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
axes[0].annotate("foot in the backstop's tunnel", xy=(edge + 6.0, g.PLATE_T + 1.2), xytext=(36.5, 7.0),
                 fontsize=7.5, arrowprops=dict(arrowstyle="->", lw=0.8))
fig.suptitle("Uno latch clip with lock key — section through the right-hand latch (cut through one key prong)", fontsize=11)
plt.tight_layout()
plt.savefig(HERE / "uno-latch-section.png")
