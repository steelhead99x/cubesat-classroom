#!/usr/bin/env python3
"""Mission patches: website label outlines and 3D-printable models, from site/src/patch-art.json.

    python3 patches.py build      # write site/src/patch-shapes.json, print files, previews; run the checks
    python3 patches.py check      # the checks only; exit 1 if any fails or a committed file is stale

Each patch is one flat, three-colour part (hardware/DESIGN-GUIDE.md: no loose pieces, no supports,
one big flat face). Colours are stacked by height, so it prints either way:
  - multi-colour printer: three bodies in one 3MF (body, tone, white);
  - single-colour printer: pause and swap filament at the tone and white heights.
The label is cut through the tone layer, so it shows in the body colour.

Needs the exact versions in gen/requirements.txt: the 3MFs are compared byte for byte.
"""
import io
import json
import math
import re
import sys
import zipfile
from pathlib import Path

import numpy as np
import svgelements
import trimesh
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.ttLib import TTFont
from shapely import affinity
from shapely.geometry import LineString, MultiPolygon, Point, Polygon
from shapely.ops import unary_union

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ART = ROOT / 'site/src/patch-art.json'
QUESTS = ROOT / 'site/src/content/quests'
SHAPES = ROOT / 'site/src/patch-shapes.json'
FONT = ROOT / 'site/node_modules/@fontsource/ibm-plex-mono/files/ibm-plex-mono-latin-500-normal.woff2'
OUT = HERE.parent / 'print'
PREVIEW = HERE.parent / 'preview'

COLLINEAR_MM = 1e-4   # outline points this close to the line through their neighbours are dropped
COLOURS = {'body': '#01017f', 'white': '#ffffff'}   # tone comes from each patch
art = json.loads(ART.read_text())
F = art['frame']
P = art['print']


# ---------------------------------------------------------------- geometry helpers
def path_polys(d, step=0.25):
    """Closed subpaths of an SVG path as shapely polygons, combined even-odd. step: sample spacing."""
    rings = []
    for sub in svgelements.Path(d).as_subpaths():
        sp = svgelements.Path(sub)
        if sp.length() == 0:
            continue
        n = max(8, int(sp.length() / step))
        pts = [(p.x, p.y) for p in (sp.point(t) for t in np.linspace(0, 1, n))]
        if len(pts) >= 3:
            poly = Polygon(pts).buffer(0)
            if not poly.is_empty:
                rings.append(poly)
    out = Polygon()
    for r in rings:
        out = out.symmetric_difference(r)
    return out


def path_lines(d):
    """Each subpath of an SVG path as a polyline (for strokes)."""
    lines = []
    for sub in svgelements.Path(d).as_subpaths():
        sp = svgelements.Path(sub)
        L = sp.length()
        if L == 0:
            continue
        n = max(2, int(L / 0.25))
        pts = [(p.x, p.y) for p in (sp.point(t) for t in np.linspace(0, 1, n))]
        closed = abs(pts[0][0] - pts[-1][0]) < 1e-6 and abs(pts[0][1] - pts[-1][1]) < 1e-6
        lines.append((pts, closed))
    return lines


def stroke(d, width):
    parts = []
    for pts, closed in path_lines(d):
        ls = LineString(pts)
        parts.append(ls.buffer(width / 2, cap_style='flat' if closed else 'round', join_style='round', quad_segs=12))
    return unary_union(parts)


def hexagon(R, corner=0.0):
    pts = [(50 + R * math.cos(math.radians(a)), 50 + R * math.sin(math.radians(a))) for a in (-90, -30, 30, 90, 150, 210)]
    h = Polygon(pts)
    return h.buffer(-corner, join_style='mitre').buffer(corner, quad_segs=12) if corner else h


def star(cx, cy, ro, ri):
    pts = []
    for k in range(10):
        r = ro if k % 2 == 0 else ri
        a = math.radians(-90 + 36 * k)
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return Polygon(pts)


def ribbon():
    r = F['ribbon']
    x0, x1, y0, y1, n = r['x0'], r['x1'], r['y0'], r['y1'], r['notch']
    ym = (y0 + y1) / 2
    return Polygon([(x0, y0), (x1, y0), (x1 - n, ym), (x1, y1), (x0, y1), (x0 + n, ym)])


def svg_d(geom, nd=2, tol=0.03):
    """A shapely (Multi)Polygon as an SVG path, even-odd, simplified to `tol` viewBox units."""
    geom = geom.simplify(tol, preserve_topology=True)
    polys = [geom] if isinstance(geom, Polygon) else list(getattr(geom, 'geoms', []))
    out = []
    for p in polys:
        for ring in [p.exterior, *p.interiors]:
            c = list(ring.coords)[:-1]
            out.append('M' + 'L'.join(f'{x:.{nd}f} {y:.{nd}f}' for x, y in c) + 'Z')
    return ''.join(out)


# ---------------------------------------------------------------- the label, from the site's own font
_font = None


def label_geom(text):
    """Outlined, upper-case label centred on the ribbon, sized to fit. Returns (geometry, cap height used)."""
    global _font
    if _font is None:
        _font = TTFont(str(FONT))
    gs = _font.getGlyphSet()
    cmap = _font.getBestCmap()
    upm = _font['head'].unitsPerEm
    cap = _font['OS/2'].sCapHeight or 0.7 * upm
    lab = F['label']
    rb = F['ribbon']
    avail = (rb['x1'] - rb['x0']) - 2 * (rb['notch'] + rb['border'] + lab['pad'])
    s = lab['cap'] / cap                              # font units -> viewBox units
    chars = [cmap[ord(ch)] for ch in text.upper()]
    adv = sum(gs[g].width for g in chars)
    # Long names: close up the letter spacing first, and shrink the letters only if that isn't enough.
    track = lab['tracking'] * upm
    if (adv + track * (len(chars) - 1)) * s > avail and len(chars) > 1:
        track = max(0.0, (avail / s - adv) / (len(chars) - 1))
    x, glyphs = 0.0, []
    for g in chars:
        pen = SVGPathPen(gs)
        gs[g].draw(pen)
        glyphs.append((x, pen.getCommands()))
        x += gs[g].width + track
    width = (x - track) * s
    if width > avail:
        s *= avail / width
        width = avail
    shapes = []
    for gx, cmds in glyphs:
        if not cmds:
            continue
        g = path_polys(cmds, step=6.0)              # font units: about 0.05 of a viewBox unit
        shapes.append(affinity.translate(g, gx, 0))
    g = unary_union(shapes)
    g = affinity.scale(g, s, -s, origin=(0, 0))      # font y is up
    cap_used = cap * s
    g = affinity.translate(g, 50 - width / 2, lab['cy'] + cap_used / 2)
    # Letters with tails (Q, J) can drop below the capitals: shrink about the centre until it all fits.
    room = ribbon().buffer(-(rb['border'] + 0.6), join_style='mitre')
    y0, y1 = g.bounds[1], g.bounds[3]
    if y1 > room.bounds[3]:                           # a tail: centre the whole outline instead of the capitals
        g = affinity.translate(g, 0, lab['cy'] - (y0 + y1) / 2)
    while not g.within(room) and cap_used > 0.5 * lab['cap']:
        g = affinity.scale(g, 0.97, 0.97, origin=(50, lab['cy']))
        cap_used *= 0.97
    return g, cap_used


def read_labels():
    out = {}
    for f in sorted(QUESTS.glob('*.md')):
        m = re.search(r'^patch:\s*(.+)$', f.read_text(), re.M)
        if m:
            out[f.stem] = m.group(1).strip().strip('"\'')
    return out


# ---------------------------------------------------------------- one patch as three colour regions
def layers(pid, label):
    spec = art['patches'][pid]
    body = unary_union([hexagon(F['R_out'], F['corner']), ribbon()])
    key = hexagon(F['R_out'], F['corner']).difference(hexagon(F['R_key'], F['corner'] * 0.9))
    band = hexagon(F['R_key'], F['corner'] * 0.9).difference(hexagon(F['R_core'], F['corner'] * 0.8))
    white = [key] + [star(x, y, F['star']['r_out'], F['star']['r_in']) for x, y in F['stars']]
    tone = [band]
    white, tone = unary_union(white), unary_union(tone)
    dx, dy = F.get('art_shift', [0, 0])
    for sh in spec['shapes']:
        g = path_polys(sh['d']) if sh['kind'] == 'fill' else stroke(sh['d'], sh['width'])
        g = affinity.translate(g, dx, dy)
        if sh.get('gap'):
            cut = g.buffer(sh['gap'], quad_segs=12)
            white, tone = white.difference(cut), tone.difference(cut)
        if sh['layer'] == 'white':
            white = white.union(g)
            tone = tone.difference(g)
        else:
            tone = tone.union(g)
            white = white.difference(g)
    rb = ribbon()
    inner = rb.buffer(-F['ribbon']['border'], join_style='mitre')
    text, cap_used = label_geom(label)
    white = white.difference(rb).union(rb.difference(inner))
    tone = tone.difference(rb).union(inner.difference(text))
    white, tone = white.intersection(body), tone.intersection(body)
    tone = tone.difference(white)
    design = dict(white=white, tone=tone)
    # Print-ready: round every region to the smallest width the printer can make. That only takes off
    # sharp tips and slivers; check_patch fails if it would remove a real part of the design.
    w = P['min_feature_mm'] / P['mm_per_unit']
    white, tone = printable(white, w), printable(tone, w)
    tone = tone.difference(white)                     # one colour per spot: the bodies never overlap
    return dict(body=body, tone=tone, white=white, text=text, cap=cap_used, design=design)


def printable(geom, w):
    return geom.buffer(-w / 2, quad_segs=8).buffer(w / 2, quad_segs=8).intersection(geom.buffer(1e-3))


def frame_paths():
    """The frame as SVG paths for Patch.astro: the same geometry as the print, before rounding."""
    hex_out = hexagon(F['R_out'], F['corner'])
    hex_key = hexagon(F['R_key'], F['corner'] * 0.9)
    rb = ribbon()
    return {
        'body': svg_d(unary_union([hex_out, rb])),
        'key': svg_d(hex_out.difference(hex_key)),
        'band': svg_d(hex_key.difference(hexagon(F['R_core'], F['corner'] * 0.8))),
        'stars': svg_d(unary_union([star(x, y, F['star']['r_out'], F['star']['r_in']) for x, y in F['stars']])),
        'ribbon': svg_d(rb),
        'ribbonInner': svg_d(rb.buffer(-F['ribbon']['border'], join_style='mitre')),
    }


# ---------------------------------------------------------------- checks (hardware/DESIGN-GUIDE.md: every rule a check)
def thin_parts(geom, min_w):
    """Area that disappears under an opening of width min_w: features too thin to print."""
    if geom.is_empty:
        return Polygon()
    opened = geom.buffer(-min_w / 2, quad_segs=8).buffer(min_w / 2, quad_segs=8)
    return geom.difference(opened.buffer(1e-3))


def check_patch(pid, L):
    u = 1 / P['mm_per_unit']
    fails = []
    for name in ('white', 'tone'):
        # whatever printable() removed must be only tips and slivers, never a piece of the art
        # A sharp tip losing its point is fine; a piece of art losing much of itself is not.
        printed = L[name].buffer(1e-3)
        for c in getattr(L['design'][name], 'geoms', [L['design'][name]]):
            lost = c.difference(printed).area
            if c.area > 0 and lost > 0.3 * c.area and lost > 0.05:
                fails.append(f'{pid}: a {name} piece near ({c.centroid.x:.1f},{c.centroid.y:.1f}) is too thin to print '
                             f'({P["min_feature_mm"]} mm): {100 * lost / c.area:.0f}% of it would disappear')
        lost = L['design'][name].difference(printed)
        if lost.area > 0.02 * L['design'][name].area:
            fails.append(f'{pid}: rounding for print would remove {100 * lost.area / L["design"][name].area:.1f}% of the {name} layer')
    # engraved label: the gaps cut into the ribbon must be wide enough to print open
    rb_in = ribbon().buffer(-F['ribbon']['border'], join_style='mitre')
    cut = rb_in.difference(L['tone'])
    bad = thin_parts(cut, P['min_gap_mm'] * u)
    big = [g for g in getattr(bad, 'geoms', [bad]) if g.area > (P['min_gap_mm'] * u) ** 2]
    if big:
        fails.append(f'{pid}: {len(big)} label cut(s) narrower than {P["min_gap_mm"]} mm')
    if L['cap'] * P['mm_per_unit'] < 3.0:
        fails.append(f'{pid}: label letters only {L["cap"] * P["mm_per_unit"]:.1f} mm tall (need 3.0 mm); shorten the label')
    for sh in art['patches'].get(pid, {}).get('shapes', []):
        if sh['kind'] == 'stroke' and sh['width'] * P['mm_per_unit'] < P['min_feature_mm']:
            fails.append(f'{pid}: a stroke is {sh["width"] * P["mm_per_unit"]:.2f} mm wide (need {P["min_feature_mm"]} mm)')
    if not L['text'].within(rb_in):
        fails.append(f'{pid}: label runs off the ribbon')
    for name in ('white', 'tone'):
        if not L[name].within(L['body'].buffer(1e-2)):
            fails.append(f'{pid}: {name} overhangs the body: it would print on air')
    if L['tone'].intersection(L['white']).area > 1e-4:
        fails.append(f'{pid}: tone and white overlap: a multi-colour printer would get two bodies in one place')
    if isinstance(L['body'], MultiPolygon):
        fails.append(f'{pid}: body is in more than one piece')
    return fails


# ---------------------------------------------------------------- 3D: meshes and a 3MF with three coloured bodies
def extrude(geom, z0, z1, s):
    meshes = []
    for p in clean_parts(geom, s):
        m = trimesh.creation.extrude_polygon(p, z1 - z0, engine='earcut')
        m.apply_translation((0, 0, z0))
        meshes.append(m)
    # Each island is its own closed shell. Islands that touch at a point keep separate vertices,
    # so no edge is ever shared by more than two faces.
    m = trimesh.util.concatenate(meshes)
    flip_flat_triangles(m)
    return m


def flip_flat_triangles(m, decimals=4):
    """Where a hole touches an outline on a straight edge, earcut leaves a triangle whose three
    corners are in a line. Flipping its longest edge with the neighbouring triangle replaces the
    pair with two that have area, over the same outline, so the solid stays closed."""
    faces = m.faces.copy()
    v = np.round(m.vertices, decimals)
    def area(f):
        a, b, c = v[f]
        return np.linalg.norm(np.cross(b - a, c - a)) / 2
    for _ in range(4):                       # a flip can expose another; a few passes settle it
        bad = [i for i, f in enumerate(faces) if area(f) < 1e-9]
        if not bad:
            break
        edges = {}
        for i, f in enumerate(faces):
            for k in range(3):
                edges.setdefault(frozenset((f[k], f[(k + 1) % 3])), []).append(i)
        for i in bad:
            f = faces[i]
            # An earlier flip in this pass may already have replaced this triangle.
            if area(f) >= 1e-9:
                continue
            # The middle corner is opposite the longest edge.
            k = max(range(3), key=lambda j: np.linalg.norm(v[f[(j + 1) % 3]] - v[f[(j + 2) % 3]]))
            mid, a, c = f[k], f[(k + 1) % 3], f[(k + 2) % 3]
            other = [j for j in edges.get(frozenset((a, c)), []) if j != i]
            if len(other) != 1:
                continue
            g = faces[other[0]]
            d = [x for x in g if x not in (a, c)]
            if len(d) != 1:
                continue
            d = d[0]
            # Keep the winding: (mid, a, c) is f's order, so the pair becomes (mid, a, d) and (mid, d, c).
            t1, t2 = np.array([mid, a, d]), np.array([mid, d, c])
            if area(t1) < 1e-9 or area(t2) < 1e-9:
                continue
            faces[i], faces[other[0]] = t1, t2
            edges = {}
            for j, ff in enumerate(faces):
                for kk in range(3):
                    edges.setdefault(frozenset((ff[kk], ff[(kk + 1) % 3])), []).append(j)
    m.faces = faces


def clean_parts(geom, s):
    """The islands of `geom` in printer mm (y up), with collinear outline points dropped.

    Sampled straight edges leave points in a line, and earcut turns three in a line into a
    zero-area cap triangle. Removing them doesn't move the outline. A point that another ring of
    the same island also uses (a hole touching the edge) is kept, or the cap would have a vertex
    the side wall doesn't, and the solid would no longer be closed."""
    out = []
    for p in getattr(geom, 'geoms', [geom]):
        if p.is_empty or p.area < 1e-6:
            continue
        p = affinity.scale(p, s, -s, origin=(0, 0))     # svg y down -> printer y up, mm
        rings = [list(p.exterior.coords)[:-1]] + [list(r.coords)[:-1] for r in p.interiors]
        keys = [{(round(x, 6), round(y, 6)) for x, y in r} for r in rings]
        shared = set()
        for i, a in enumerate(keys):
            for b in keys[i + 1:]:
                shared |= a & b
        rings = [drop_collinear(r, shared) for r in rings]
        q = Polygon(rings[0], [r for r in rings[1:] if len(r) >= 3])
        if q.is_valid and q.area >= 1e-6:
            out.append(q)
        else:
            out.append(p)        # leave an island that would change shape exactly as it was
    return out


def drop_collinear(ring, keep, tol=COLLINEAR_MM):
    """Removes points that lie within `tol` mm of the line through their neighbours."""
    pts = list(ring)
    changed = True
    while changed and len(pts) > 3:
        changed = False
        i = 0
        while i < len(pts) and len(pts) > 3:
            a, b, c = np.array(pts[i - 1]), np.array(pts[i]), np.array(pts[(i + 1) % len(pts)])
            ac = c - a
            n = np.linalg.norm(ac)
            off = abs(ac[0] * (b - a)[1] - ac[1] * (b - a)[0]) / n if n > 0 else np.linalg.norm(b - a)
            between = n > 0 and 0 <= np.dot(b - a, ac) <= n * n
            if off < tol and between and (round(b[0], 6), round(b[1], 6)) not in keep:
                pts.pop(i)
                changed = True
            else:
                i += 1
    return pts


def zero_area_faces(m, decimals=4):
    """Triangles with no area once vertices are rounded as write_3mf writes them."""
    v = np.round(m.vertices, decimals)
    t = v[m.faces]
    return int((np.linalg.norm(np.cross(t[:, 1] - t[:, 0], t[:, 2] - t[:, 0]), axis=1) < 1e-9).sum())


def solids(L):
    s = P['mm_per_unit']
    b, t, w = P['body_mm'], P['tone_mm'], P['white_mm']
    return {
        'body': extrude(L['body'], 0, b, s),
        'tone': extrude(L['tone'], b, b + t, s),
        'white': extrude(L['white'], b, b + t + w, s),   # white starts on the body, so it never floats
    }


def write_3mf(path, parts, colours, title):
    """A plain 3MF: one mesh per colour, each with its display colour, grouped as the components of
    one object with a single build item, so a slicer keeps the three colours together as one part
    (3MF Core 3.4.3) instead of laying them out separately. Written deterministically."""
    centre = None
    allv = np.vstack([m.vertices for m in parts.values()])
    lo, hi = allv.min(0), allv.max(0)
    centre = ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2)
    mats = ''.join(f'<base name="{k}" displaycolor="{colours[k].upper()}FF"/>' for k in parts)
    objs, items = [], []
    for i, (k, m) in enumerate(parts.items(), start=2):
        v = ''.join(f'<vertex x="{x - centre[0]:.4f}" y="{y - centre[1]:.4f}" z="{z:.4f}"/>' for x, y, z in m.vertices)
        f = ''.join(f'<triangle v1="{a}" v2="{b}" v3="{c}"/>' for a, b, c in m.faces)
        objs.append(f'<object id="{i}" name="{title} - {k}" type="model" pid="1" pindex="{i - 2}">'
                    f'<mesh><vertices>{v}</vertices><triangles>{f}</triangles></mesh></object>')
        items.append(f'<component objectid="{i}"/>')
    whole = len(parts) + 2
    objs.append(f'<object id="{whole}" name="{title}" type="model"><components>{"".join(items)}</components></object>')
    items = [f'<item objectid="{whole}"/>']
    model = ('<?xml version="1.0" encoding="UTF-8"?>\n<model unit="millimeter" xml:lang="en-US" '
             'xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">'
             f'<metadata name="Title">{title}</metadata><resources><basematerials id="1">{mats}</basematerials>'
             f'{"".join(objs)}</resources><build>{"".join(items)}</build></model>')
    ct = ('<?xml version="1.0" encoding="UTF-8"?>\n<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
          '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
          '<Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>')
    rels = ('<?xml version="1.0" encoding="UTF-8"?>\n<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>')
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in (('[Content_Types].xml', ct), ('_rels/.rels', rels), ('3D/3dmodel.model', model)):
            zi = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(zi, data)
    return buf.getvalue()


def preview_png(L, tone):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from matplotlib.patches import PathPatch
    from matplotlib.path import Path as MPath

    def patch(g, c):
        for p in getattr(g, 'geoms', [g]):
            if p.is_empty:
                continue
            verts, codes = [], []
            for ring in [p.exterior, *p.interiors]:
                c_ = list(ring.coords)
                verts += c_
                codes += [MPath.MOVETO] + [MPath.LINETO] * (len(c_) - 2) + [MPath.CLOSEPOLY]
            ax.add_patch(PathPatch(MPath(verts, codes), facecolor=c, edgecolor='none'))
    fig = plt.figure(figsize=(2.4, 2.4), dpi=100)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, 100); ax.set_ylim(100, 0); ax.set_aspect('equal'); ax.axis('off')
    patch(L['body'], COLOURS['body']); patch(L['tone'], tone); patch(L['white'], COLOURS['white'])
    b = io.BytesIO()
    fig.savefig(b, format='png', transparent=True)
    plt.close(fig)
    return b.getvalue()


def coupon():
    """Test coupon (DESIGN-GUIDE: a coupon before a full part): the ribbon with the longest label, and
    a strip of the thinnest white and tone features, on a few grams of plastic."""
    labels = read_labels()
    longest = max(labels.values(), key=len)
    rb = ribbon()
    inner = rb.buffer(-F['ribbon']['border'], join_style='mitre')
    text, _ = label_geom(longest)
    u = 1 / P['mm_per_unit']
    bars = [Polygon([(10 + 9 * k, 46), (10 + 9 * k + w * u, 46), (10 + 9 * k + w * u, 56), (10 + 9 * k, 56)])
            for k, w in enumerate([0.6, 0.8, 1.0, 1.2])]
    base = Polygon([(3, 44), (97, 44), (97, 74), (3, 74)])
    body = unary_union([rb, base])
    white = unary_union([rb.difference(inner)] + bars[:2])
    tone = unary_union([inner.difference(text)] + bars[2:]).difference(white)
    return dict(body=body, tone=tone, white=white, text=text, cap=0, design=dict(white=white, tone=tone))


# ---------------------------------------------------------------- build / check
def build(write=True):
    labels = read_labels()
    missing = [q for q in labels if q not in art['patches']]
    fails = [f'quest {q} has no patch art in patch-art.json' for q in missing]
    web, files = {}, {}
    for pid, spec in art['patches'].items():
        if pid not in labels:
            fails.append(f'{pid}: no quest with that id (its label comes from the quest\'s patch field)')
            continue
        L = layers(pid, labels[pid])
        fails += check_patch(pid, L)
        web[pid] = {'label': labels[pid], 'd': svg_d(L['text'])}
        parts = solids(L)
        for k, m in parts.items():
            if not (m.is_watertight and m.is_winding_consistent) or m.volume <= 0:
                fails.append(f'{pid}: the {k} body is not a closed solid')
            if zero_area_faces(m):
                fails.append(f'{pid}: the {k} body has {zero_area_faces(m)} zero-area triangles')
        cols = dict(COLOURS, tone=spec['tone'])
        files[OUT / f'{pid}.3mf'] = write_3mf(None, parts, cols, f'CubeSat Classroom patch: {labels[pid]}')
        files[PREVIEW / f'{pid}.png'] = preview_png(L, spec['tone'])
    C = coupon()
    files[OUT / 'coupon-label-and-features.3mf'] = write_3mf(None, solids(C), dict(COLOURS, tone='#ffd23f'), 'Patch test coupon')
    files[SHAPES] = (json.dumps({'about': 'Generated by hardware/patches/gen/patches.py from patch-art.json. Do not edit.',
                                 'frame': frame_paths(), 'labels': web}, indent=1, sort_keys=True) + '\n').encode()
    stale = []
    for path, data in files.items():
        if path.suffix == '.png':
            continue                       # previews are not compared byte for byte (renderer versions differ)
        if not path.exists() or path.read_bytes() != data:
            stale.append(path.relative_to(ROOT))
    if write:
        OUT.mkdir(exist_ok=True); PREVIEW.mkdir(exist_ok=True)
        for path, data in files.items():
            path.write_bytes(data)
    for f in fails:
        print('FAIL', f)
    print(f'{len(web)} patches, {len(fails)} failures' + ('' if write else f', {len(stale)} stale files'))
    if not write:
        for s in stale:
            print('STALE', s, '(run: python3 hardware/patches/gen/patches.py build, with gen/requirements.txt installed)')
    return 1 if fails or (not write and stale) else 0


if __name__ == '__main__':
    cmd = sys.argv[1] if len(sys.argv) > 1 else 'build'
    sys.exit(build(write=(cmd == 'build')))
