"""Write csc-r1-bom.csv from design.py (one line per value/footprint/part number), into the
folder given as the first argument (default ../fab)."""
import csv, collections, os, sys, boardcfg
from pedal_sourcing import specification
design = boardcfg.design()
g = collections.OrderedDict()
for r, p in design.parts.items():
    if p['sym'] in ('MountingHole_Pad', 'TestPoint'):
        continue
    sourcing = p['mpn']
    if boardcfg.PROJ in ('csc-r1p', 'csc-r1p-ctl') and not sourcing and not p['dnp']:
        sourcing = specification(p)
    k = (p['value'], p['fp'].split(':')[1], sourcing, 'DNP' if p['dnp'] else '', p['note'])
    g.setdefault(k, []).append(r)
out_dir = sys.argv[1] if len(sys.argv) > 1 else boardcfg.FAB
with open(os.path.join(out_dir, f'{boardcfg.PROJ}-bom.csv'), 'w', newline='') as f:
    w = csv.writer(f)
    w.writerow(['Qty', 'References', 'Value', 'Footprint', 'MPN / description', 'Fit', 'Note'])
    for (v, fp, mpn, dnp, note), refs in g.items():
        w.writerow([len(refs), ' '.join(refs), v, fp, mpn, dnp, note])
print('bom lines', len(g), 'fitted parts', sum(len(v) for k, v in g.items() if not k[3]),
      'not fitted', sum(len(v) for k, v in g.items() if k[3]))
