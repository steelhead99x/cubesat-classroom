"""Placement helpers shared by the pedal boards' placement scripts (gen_pcb_r1p*.py).

Same conventions as gen_pcb.py (the dev kit's placer, left unchanged so its committed board
stays reproducible): board coordinates in mm from the top-left corner, offset by OX, OY on the
KiCad sheet; footprints loaded from KiCad's library (or the project's CSC_R1 footprints).
"""
import json
import math
import shutil

import pcbnew

import boardcfg

FPDIR = '/usr/share/kicad/footprints'
OX, OY = 60.0, 50.0
mm = pcbnew.FromMM


def V(x, y):
    return pcbnew.VECTOR2I(mm(OX + x), mm(OY + y))


class Board:
    def __init__(self, design):
        self.d = design
        self.parts = design.parts
        self.board = pcbnew.BOARD()
        self.board.SetCopperLayerCount(4)
        self.nets = {}
        for i, name in enumerate(sorted(design.nets())):
            ni = pcbnew.NETINFO_ITEM(self.board, name, i + 1)
            self.board.Add(ni)
            self.nets[name] = ni
        self.fps = {}
        for r, p in self.parts.items():
            lib, name = p['fp'].split(':')
            f = pcbnew.FootprintLoad(f"{boardcfg.KICAD if lib == 'CSC_R1' else FPDIR}/{lib}.pretty", name)
            assert f is not None, (r, p['fp'])
            f.SetReference(r); f.SetValue(p['value']); f.SetFPID(pcbnew.LIB_ID(lib, name))
            for pd in f.Pads():
                n = p['pins'].get(pd.GetNumber())
                if n and n != design.NC:
                    pd.SetNet(self.nets[n])
            if p['dnp'] and hasattr(pcbnew, 'FP_EXCLUDE_FROM_BOM'):
                f.SetAttributes(f.GetAttributes() | pcbnew.FP_EXCLUDE_FROM_POS_FILES | pcbnew.FP_EXCLUDE_FROM_BOM)
            f.Reference().SetTextSize(pcbnew.VECTOR2I(mm(0.6), mm(0.6))); f.Reference().SetTextThickness(mm(0.1))
            f.SetPosition(V(-30, -30)); self.board.Add(f); self.fps[r] = f
        self.placed = set()
        self.tracks = []

    # ---- finding parts
    def sel(self, prefix, *ns):
        return [r for r, p in self.parts.items()
                if r.rstrip('0123456789') == prefix and all(n in p['pins'].values() for n in ns)]

    def one(self, prefix, *ns, i=0, value=None):
        c = self.sel(prefix, *ns)
        if value is not None:
            c = [r for r in c if self.parts[r]['value'] == value]
        assert len(c) > i, (prefix, ns, value, c)
        return c[i]

    def pinof(self, ref, net):
        return [k for k, v in self.parts[ref]['pins'].items() if v == net][0]

    # ---- placing
    def pos(self, ref, pin):
        ps = [pd.GetPosition() for pd in self.fps[ref].Pads() if pd.GetNumber() == str(pin)]
        assert ps, (ref, pin)
        return (sum(pcbnew.ToMM(q.x) for q in ps) / len(ps) - OX, sum(pcbnew.ToMM(q.y) for q in ps) / len(ps) - OY)

    def centre(self, ref):
        p = self.fps[ref].GetPosition()
        return pcbnew.ToMM(p.x) - OX, pcbnew.ToMM(p.y) - OY

    def put(self, ref, x, y, rot=0, back=False):
        f = self.fps[ref]
        if back and not f.IsFlipped():
            f.Flip(f.GetPosition(), True)   # mirror left-right, as seen from above
        f.SetOrientationDegrees(rot); f.SetPosition(V(x, y)); self.placed.add(ref)
        return ref

    def pin_at(self, ref, pin, x, y, rot=0, back=False):
        self.put(ref, 0, 0, rot, back); px, py = self.pos(ref, pin); self.put(ref, x - px, y - py, rot, back)
        return ref

    def inline(self, ref, x, y, ang, pin='1'):
        """2-pad part: pad `pin` at (x, y), the other pad in screen direction ang (0 = +x, 90 = down)."""
        rot = -ang if str(pin) == '1' else 180 - ang
        self.pin_at(ref, pin, x, y, rot)
        other = '2' if str(pin) == '1' else '1'; ox, oy = self.pos(ref, other)
        d = math.degrees(math.atan2(oy - y, ox - x)) % 360
        assert abs((d - ang + 180) % 360 - 180) < 1, (ref, d, ang)
        return (ox, oy)

    def at(self, ref, net, x, y, ang):
        return self.inline(ref, x, y, ang, self.pinof(ref, net))

    # ---- hand routes
    def seg(self, net, a, b, w=0.35, layer=pcbnew.F_Cu):
        if math.hypot(a[0] - b[0], a[1] - b[1]) < 1e-4:
            return
        t = pcbnew.PCB_TRACK(self.board); t.SetStart(V(*a)); t.SetEnd(V(*b)); t.SetWidth(mm(w)); t.SetLayer(layer)
        t.SetNet(self.nets[net]); self.board.Add(t); self.tracks.append(t)

    def path(self, net, pts, w=0.35):
        for a, b in zip(pts[:-1], pts[1:]):
            self.seg(net, a, b, w)

    def connect(self, net, w=0.35, only=None):
        """straight-segment minimum spanning tree over the pads of a net"""
        pts = []
        for r, p in self.parts.items():
            if only and r not in only:
                continue
            for k, v in p['pins'].items():
                if v == net:
                    pts.append(self.pos(r, k))
        done = [pts[0]]; todo = pts[1:]
        while todo:
            best = min(((math.hypot(a[0] - b[0], a[1] - b[1]), a, b) for a in done for b in todo))
            self.seg(net, best[1], best[2], w); done.append(best[2]); todo.remove(best[2])

    # ---- outline: straight edges plus quarter-circle notches for the box's lid-screw posts
    def outline(self, pts_arcs):
        """pts_arcs: list of items, each ('L', (x, y)) to draw a line to, or ('A', (cx, cy), r, a0, a1)
        for an arc around (cx, cy) from angle a0 to a1 (degrees, screen coordinates)."""
        poly = []
        for it in pts_arcs:
            if it[0] == 'L':
                poly.append(it[1])
            else:
                _, (cx, cy), r, a0, a1 = it
                n = max(4, int(abs(a1 - a0) / 7.5))
                for k in range(n + 1):
                    a = math.radians(a0 + (a1 - a0) * k / n)
                    poly.append((cx + r * math.cos(a), cy + r * math.sin(a)))
        # A line that ends where the next arc starts gives the same point twice: drop repeats,
        # since a zero-length edge makes KiCad call the outline malformed.
        clean = []
        for p in poly:
            if not clean or math.dist(p, clean[-1]) > 1e-4:
                clean.append(p)
        if math.dist(clean[0], clean[-1]) <= 1e-4:
            clean.pop()
        poly = clean
        for a, b in zip(poly, poly[1:] + poly[:1]):
            s = pcbnew.PCB_SHAPE(self.board); s.SetShape(pcbnew.SHAPE_T_SEGMENT); s.SetStart(V(*a)); s.SetEnd(V(*b))
            s.SetLayer(pcbnew.Edge_Cuts); s.SetWidth(mm(0.1)); self.board.Add(s)
        return poly

    def drawing(self, layer, pts, w=0.15, closed=True):
        seq = pts + (pts[:1] if closed else [])
        for a, b in zip(seq, seq[1:]):
            s = pcbnew.PCB_SHAPE(self.board); s.SetShape(pcbnew.SHAPE_T_SEGMENT); s.SetStart(V(*a)); s.SetEnd(V(*b))
            s.SetLayer(layer); s.SetWidth(mm(w)); self.board.Add(s)

    # ---- save with the project's design rules (the dev kit's rules)
    def save(self, power_nets, partial=False):
        missing = [r for r in self.parts if r not in self.placed]
        assert partial or not missing, f'unplaced: {missing}'
        if missing:
            print('UNPLACED (partial save):', len(missing))
        out, proj = boardcfg.KICAD, boardcfg.PROJ

        def ncl(name, w, cl=0.15):
            return {"name": name, "clearance": cl, "track_width": w, "via_diameter": 0.6, "via_drill": 0.3,
                    "microvia_diameter": 0.3, "microvia_drill": 0.1, "diff_pair_width": 0.2, "diff_pair_gap": 0.25,
                    "diff_pair_via_gap": 0.25, "bus_width": 12, "wire_width": 6, "line_style": 0,
                    "pcb_color": "rgba(0, 0, 0, 0.000)", "schematic_color": "rgba(0, 0, 0, 0.000)"}
        pcbnew.SaveBoard(f'{out}/{proj}.kicad_pcb', self.board)
        full = json.load(open(f'{out}/{proj}.kicad_pro'))
        full['net_settings'] = {"classes": [ncl('Default', 0.2), ncl("Power", 0.3)], "meta": {"version": 3},
                                "net_colors": None, "netclass_assignments": None,
                                "netclass_patterns": [{"netclass": "Power", "pattern": n} for n in power_nets]}
        ds = full.setdefault('board', {}).setdefault('design_settings', {})
        ds.setdefault('rules', {}).update({
            "min_clearance": 0.127, "min_track_width": 0.127, "min_via_diameter": 0.45, "min_through_hole_diameter": 0.2,
            "min_via_annular_width": 0.1, "min_hole_clearance": 0.2, "min_hole_to_hole": 0.25, "min_copper_edge_clearance": 0.3,
            "min_microvia_diameter": 0.2, "min_microvia_drill": 0.1, "min_silk_clearance": 0.0, "min_text_height": 0.5,
            "min_text_thickness": 0.08, "solder_mask_to_copper_clearance": 0.0, "min_connection": 0.0, "min_resolved_spokes": 1,
            "allow_blind_buried_vias": False, "allow_microvias": False, "max_error": 0.005})
        ds.setdefault('rule_severities', {}).update({
            "silk_overlap": "ignore", "silk_over_copper": "ignore", "silk_edge_clearance": "ignore", "lib_footprint_issues": "ignore",
            "lib_footprint_mismatch": "ignore", "text_height": "ignore", "text_thickness": "ignore", "solder_mask_bridge": "warning"})
        ds['track_widths'] = [0.0, 0.2, 0.35, 0.4]
        ds['via_dimensions'] = [{"diameter": 0.0, "drill": 0.0}, {"diameter": 0.6, "drill": 0.3}]
        ds.setdefault('defaults', {}); ds.setdefault('meta', {"version": 2})
        json.dump(full, open(f'{out}/{proj}.kicad_pro', 'w'), indent=1)
        shutil.copy(f'{out}/{proj}.kicad_pcb', boardcfg.BASE)
        # The router loads the base board; without its project file beside it, KiCad would save
        # default rules (0.3 mm drill, 0.8 mm text) over these when the routed board is written.
        shutil.copy(f'{out}/{proj}.kicad_pro', boardcfg.BASE.replace('.kicad_pcb', '.kicad_pro'))
        print('saved', proj, 'parts', len(self.parts), 'hand tracks', len(self.tracks))


# The box: Hammond 1590N1 ("125B"). Lid-screw posts on a 104.10 x 49.00 mm pattern, centred.
# Both pedal boards share these board coordinates: x across the box (0 = left wall side as seen
# from above the face), y along it (0 = top end, where the power and USB jacks are).
def notch(c, r, p_from, p_to):
    """Outline arc of radius r around a lid post at c, from p_from to p_to, the short way round
    (into the board). Angles come from the end points, so the arc meets the lines exactly."""
    a0 = math.degrees(math.atan2(p_from[1] - c[1], p_from[0] - c[0]))
    a1 = math.degrees(math.atan2(p_to[1] - c[1], p_to[0] - c[0]))
    while a1 - a0 > 180:
        a1 -= 360
    while a0 - a1 > 180:
        a1 += 360
    return ('A', c, r, a0, a1)


BOX_POSTS = [(27 - 24.5, 52 - 52.05), (27 + 24.5, 52 - 52.05), (27 - 24.5, 52 + 52.05), (27 + 24.5, 52 + 52.05)]
POST_CLEAR = 5.5            # notch radius around each post centre
STANDOFFS = [(4.6, 62.6), (50.4, 23.3), (50.4, 66.2)]   # clear on both boards: check both placers   # M2.5, shared by both boards (targets; the placer nudges them)


# ---------------------------------------------------------------- automatic small-part placement
def _bbox(f):
    for l in (pcbnew.F_CrtYd, pcbnew.B_CrtYd):
        c = f.GetCourtyard(l)
        if c.OutlineCount():
            bb = c.BBox(); break
    else:
        bb = f.GetBoundingBox(False, False)
    T = pcbnew.ToMM
    return (T(bb.GetLeft()) - OX, T(bb.GetTop()) - OY, T(bb.GetRight()) - OX, T(bb.GetBottom()) - OY)


def _overlap(a, b, m=0.0):
    return min(a[2], b[2]) - max(a[0], b[0]) > -m and min(a[3], b[3]) - max(a[1], b[1]) > -m


class AutoPlace:
    """Places small parts next to a pin, trying nearby spots until one is free and on the board."""
    def __init__(self, k, poly, keepouts=(), edge=0.6):
        from matplotlib.path import Path
        self.k = k; self.path = Path(poly); self.keepouts = list(keepouts); self.edge = edge
        import os
        self.soft = bool(os.environ.get('PLACE_SOFT')); self.failed = []

    def _fail(self, ref, msg):
        if self.soft:
            self.failed.append(msg); self.k.put(ref, -20, -20 - 3 * len(self.failed)); self.k.placed.discard(ref)
            return None
        raise RuntimeError(msg)

    def on_board(self, bb):
        e = self.edge
        corners = [(bb[0] - e, bb[1] - e), (bb[2] + e, bb[1] - e), (bb[2] + e, bb[3] + e), (bb[0] - e, bb[3] + e),
                   ((bb[0] + bb[2]) / 2, bb[1] - e), ((bb[0] + bb[2]) / 2, bb[3] + e), (bb[0] - e, (bb[1] + bb[3]) / 2), (bb[2] + e, (bb[1] + bb[3]) / 2)]
        return all(self.path.contains_point(c) for c in corners) and not any(_overlap(bb, z) for z in self.keepouts)

    def free(self, ref, margin=0.15):
        bb = _bbox(self.k.fps[ref])
        if not self.on_board(bb):
            return False
        for r in self.k.placed:
            if r != ref and _overlap(bb, _bbox(self.k.fps[r]), margin):
                return False
        return True

    def near(self, ref, net, xy, ang, dist=0.9, along=None, steps=14, step=0.55):
        """Pad on `net` near xy, the part pointing away in direction ang (0 = +x, 90 = down).
        Tries sliding sideways (both ways) and moving further out until it fits."""
        k = self.k
        along = (ang + 90) % 360 if along is None else along
        ux, uy = math.cos(math.radians(ang)), math.sin(math.radians(ang))
        vx, vy = math.cos(math.radians(along)), math.sin(math.radians(along))
        for out in range(6):
            for s in range(steps):
                off = ((s + 1) // 2) * step * (1 if s % 2 else -1)
                x = xy[0] + ux * (dist + out * 0.9) + vx * off
                y = xy[1] + uy * (dist + out * 0.9) + vy * off
                try:
                    k.at(ref, net, x, y, ang)
                except AssertionError:
                    continue
                if self.free(ref):
                    return k.pos(ref, k.pinof(ref, net))
        k.placed.discard(ref)
        return self._fail(ref, f'no room for {ref} near {xy}')

    def block(self, ref, x, y, rot=0, tries=30, step=0.5):
        """A larger part at (x, y), nudged outward in a spiral until free."""
        k = self.k
        for i in range(tries):
            a = i * 2.4; rr = step * math.sqrt(i)
            k.put(ref, x + rr * math.cos(a), y + rr * math.sin(a), rot)
            if self.free(ref):
                return k.centre(ref)
        k.put(ref, x, y, rot); bb = _bbox(k.fps[ref])
        why = [r for r in k.placed if r != ref and _overlap(bb, _bbox(k.fps[r]), 0.15)]
        if not self.on_board(bb): why.append('board edge')
        k.placed.discard(ref)
        ref = ref + f' (blocked by {", ".join(why)})' if False else ref
        self._why = why
        return self._fail(ref, f'no room for {ref} at {(x, y)}: blocked by {", ".join(self._why)}')

    def decouple(self, chip, caps, dist=0.9):
        """Each cap next to the chip pin on its net, pointing away from the chip."""
        k = self.k
        cx, cy = k.centre(chip)
        for c in caps:
            net = [v for v in k.parts[c]['pins'].values() if v != 'GND'][0]
            pins = [pn for pn, v in k.parts[chip]['pins'].items() if v == net]
            best = None
            allp = [k.pos(chip, q) for q in k.parts[chip]['pins']]
            for pn in pins:
                px, py = k.pos(chip, pn)
                # fine-pitch chip: keep the cap 2 mm out so the neighbouring pins can still escape
                fine = any(0.01 < math.hypot(px - qx, py - qy) < 0.8 for qx, qy in allp)
                dist_ = max(dist, 2.0) if fine else dist
                ang = math.degrees(math.atan2(py - cy, px - cx))
                ang = round(ang / 90) * 90 % 360
                soft, self.soft = self.soft, True
                n0 = len(self.failed)
                r_ = self.near(c, net, (px, py), ang, dist_)
                self.soft = soft
                if r_ is not None:
                    best = True; break
                del self.failed[n0:]
            if not best:
                self._fail(c, f'no room for {c} by {chip}')
