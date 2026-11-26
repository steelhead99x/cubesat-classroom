"""Independent checks on one board: pads vs the schematic netlist KiCad exported (not vs design.py),
plus the mechanical rules for the face. Exits 1 on any failure."""
import sys, pcbnew
from sexp import parse, find, first
sys.setrecursionlimit(100000)
import design
VAR = sys.argv[1]; design.load(VAR)
D = f'../kicad/solar-face-{VAR}'
t = parse(open(f'{D}/rt.net').read())
sch = {}
for n in find(first(t, 'nets'), 'net'):
    name = first(n, 'name')[1].lstrip('/')
    for x in find(n, 'node'):
        sch[(first(x, 'ref')[1], first(x, 'pin')[1])] = name
comps = {first(c, 'ref')[1]: first(c, 'footprint')[1] for c in find(first(t, 'components'), 'comp')}
b = pcbnew.LoadBoard(f'{D}/solar-face-{VAR}.kicad_pcb')
bad = []
OX, OY = 100.0, 50.0
bb = b.GetBoardEdgesBoundingBox()
w, h = pcbnew.ToMM(bb.GetWidth()), pcbnew.ToMM(bb.GetHeight())
if abs(w - design.W) > 0.15 or abs(h - design.H) > 0.15:   # bounding box includes the 0.1 mm edge line
    bad.append(f'outline {w:.2f} x {h:.2f}, want {design.W} x {design.H}')
seen = set()
for f in b.GetFootprints():
    r = f.GetReference(); seen.add(r)
    fp = f'{f.GetFPID().GetLibNickname()}:{f.GetFPID().GetLibItemName()}'
    if comps.get(r) != fp: bad.append(f'footprint differs {r} {comps.get(r)} {fp}')
    is_cell = r.startswith('SC')
    if is_cell == f.IsFlipped(): bad.append(f'{r} on the wrong side: cells go on the front, everything else on the back')
    for p in f.Pads():
        if p.GetAttribute() != pcbnew.PAD_ATTRIB_SMD: bad.append(f'{r} pad {p.GetNumber()} is not surface mount')
        if p.GetNumber() in ('', 'MP'): continue
        s = sch.get((r, p.GetNumber()))
        s = None if (s is None or s.startswith('unconnected')) else s
        if s != (p.GetNetname() or None): bad.append(f'net differs {r}.{p.GetNumber()} sch {s} pcb {p.GetNetname()}')
    cy = f.GetCourtyard(pcbnew.B_CrtYd if f.IsFlipped() else pcbnew.F_CrtYd).BBox()
    x0, y0 = pcbnew.ToMM(cy.GetLeft()) - OX, pcbnew.ToMM(cy.GetTop()) - OY
    x1, y1 = pcbnew.ToMM(cy.GetRight()) - OX, pcbnew.ToMM(cy.GetBottom()) - OY
    if x0 < -0.01 or y0 < -0.01 or x1 > design.W + 0.01 or y1 > design.H + 0.01:
        bad.append(f'{r} courtyard leaves the board ({x0:.2f},{y0:.2f})-({x1:.2f},{y1:.2f})')
    if is_cell and (y0 < 1.0 or y1 > design.H - 1.0):
        bad.append(f'{r} within 1 mm of the top or bottom edge, where the printed lips hold the board')
for r in comps:
    if r not in seen: bad.append(f'missing on board {r}')
cell_area = sum(1 for r in comps if r.startswith('SC'))
for x in bad: print('FAIL', x)
print(f'{VAR}: footprints {len(seen)}, cells {cell_area}, outline {design.W} x {design.H} mm, failures {len(bad)}')
sys.exit(1 if bad else 0)
