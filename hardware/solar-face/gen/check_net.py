"""The schematic's netlist, exported by KiCad, must match design.py exactly."""
import sys; sys.setrecursionlimit(100000)
from sexp import parse, find, first
import design
VAR = sys.argv[1]; design.load(VAR)
t = parse(open(f'../kicad/solar-face-{VAR}/rt.net').read())
got = {}
for n in find(first(t, 'nets'), 'net'):
    name = first(n, 'name')[1]
    got[name.lstrip('/')] = sorted((first(x, 'ref')[1], first(x, 'pin')[1]) for x in find(n, 'node'))
want = {k: sorted(v) for k, v in design.nets().items()}
bad = 0
for k in sorted(set(got) | set(want)):
    g, w = got.get(k), want.get(k)
    if g != w and not (w is None and k.startswith('unconnected')):
        bad += 1; print('MISMATCH', k, '\n   sch :', g, '\n   want:', w)
comps = {first(c, 'ref')[1]: first(c, 'footprint')[1] for c in find(first(t, 'components'), 'comp')}
for r, p in design.parts.items():
    if r not in comps: bad += 1; print('missing comp', r)
    elif comps[r] != p['fp']: bad += 1; print('fp differs', r, comps[r], p['fp'])
print(f'{VAR}: nets', len(want), 'components', len(comps), 'mismatches', bad)
sys.exit(1 if bad else 0)
