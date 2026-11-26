"""Write label.json: outlines of the raised lettering on the fit plate.

Optional, only needed to change the wording. Needs matplotlib (for its bundled DejaVu Sans
Bold font) on top of hardware/requirements.txt. generate.py reads label.json, so the CAD
itself doesn't need matplotlib. GPL-3.0, like the rest of this folder; DejaVu fonts are
free to use, and only the letter outlines are stored.
"""

import json
from pathlib import Path

from matplotlib.font_manager import FontProperties
from matplotlib.textpath import TextPath

HERE = Path(__file__).resolve().parent
FONT = FontProperties(family="DejaVu Sans", weight="bold")

# (text, centre x, centre y, cap height or None, max width), in plate coordinates (mm).
# The nameplate sits under the Uno, between its posts; the axis labels sit in the free
# space between the Uno and the Nano.
LINES = [
    ("CUBESAT", 0.0, 14.8, 7.0, 44.0),
    ("CLASSROOM", 0.0, 5.6, 5.2, 44.0),
    ("PAYLOAD DECK", 0.0, -2.0, 3.6, 44.0),
    ("FIT CHECK", 0.0, -7.6, 3.6, 46.0),
    ("+X", 22.5, -29.0, 3.6, 20.0),
    ("+Y", 10.2, -25.6, 3.6, 20.0),
]
TRACKING = 0.08                 # extra letter spacing, as a fraction of the cap height


def outline(text, cap):
    """Polygons for one line, tracked out, with its bounding box."""
    probe = TextPath((0, 0), "H", size=1.0, prop=FONT)
    size = cap / (probe.get_extents().y1 - probe.get_extents().y0)
    polys, x = [], 0.0
    for ch in text:
        if ch == " ":
            x += 0.45 * cap + TRACKING * cap
            continue
        path = TextPath((x, 0), ch, size=size, prop=FONT)
        polys += [p.tolist() for p in path.to_polygons(closed_only=True) if len(p) > 2]
        advance = TextPath((0, 0), ch + "H", size=size, prop=FONT).get_extents().x1 - \
            TextPath((0, 0), "H", size=size, prop=FONT).get_extents().x1
        x += advance + TRACKING * cap
    xs = [p[0] for poly in polys for p in poly]
    ys = [p[1] for poly in polys for p in poly]
    return polys, (min(xs), min(ys), max(xs), max(ys))


def main():
    out = []
    for text, cx, cy, cap, width in LINES:
        polys, (x0, y0, x1, y1) = outline(text, cap)
        scale = min(1.0, width / (x1 - x0))
        mx, my = (x0 + x1) / 2, (y0 + y1) / 2
        out.append({"text": text, "polygons": [
            [[round(cx + (x - mx) * scale, 3), round(cy + (y - my) * scale, 3)] for x, y in poly]
            for poly in polys]})
        print(f"{text}: {(x1 - x0) * scale:.1f} x {(y1 - y0) * scale:.1f} mm")
    (HERE / "label.json").write_text(json.dumps(out, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    main()
