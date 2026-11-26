"""Write ../fab/csc-r1-bom.csv from design.py (one line per value/footprint/part number)."""
import csv, collections, design
g = collections.OrderedDict()
for r, p in design.parts.items():
    if p['sym'] in ('MountingHole_Pad', 'TestPoint'):
        continue
    k = (p['value'], p['fp'].split(':')[1], p['mpn'], 'DNP' if p['dnp'] else '', p['note'])
    g.setdefault(k, []).append(r)
with open('../fab/csc-r1-bom.csv', 'w', newline='') as f:
    w = csv.writer(f)
    w.writerow(['Qty', 'References', 'Value', 'Footprint', 'MPN / description', 'Fit', 'Note'])
    for (v, fp, mpn, dnp, note), refs in g.items():
        w.writerow([len(refs), ' '.join(refs), v, fp, mpn, dnp, note])
print('bom lines', len(g), 'fitted parts', sum(len(v) for k, v in g.items() if not k[3]),
      'not fitted', sum(len(v) for k, v in g.items() if k[3]))
