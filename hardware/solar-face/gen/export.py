"""Review files for one variant, and the fab package only when every cell land pattern is verified.

Always: schematic PDF, BOM. Gerbers, drill and placement files are written only when the board's cell
footprint has verified=True in cells.py; otherwise ../fab/<variant>/NOT-FOR-FAB.txt says why."""
import csv, os, shutil, subprocess, sys
import cells, design
VAR = sys.argv[1]; design.load(VAR)
PROJ = f'solar-face-{VAR}'; D = f'../kicad/{PROJ}'; FAB = f'../fab/{VAR}'
shutil.rmtree(FAB, ignore_errors=True); os.makedirs(FAB)
run = lambda *a: subprocess.run(a, check=True, capture_output=True, cwd=D)
run('kicad-cli', 'sch', 'export', 'pdf', '-o', f'{PROJ}-schematic.pdf', f'{PROJ}.kicad_sch')
groups = {}
for r, p in design.parts.items():
    if p['sym'] == 'TestPoint': continue
    groups.setdefault((p['value'], p['fp'].split(':')[1], p['mpn'], p['note']), []).append(r)
with open(f'{FAB}/{PROJ}-bom.csv', 'w', newline='') as f:
    w = csv.writer(f); w.writerow(['Qty', 'References', 'Value', 'Footprint', 'Part', 'Note'])
    for (v, fp, mpn, note), refs in groups.items(): w.writerow([len(refs), ' '.join(refs), v, fp, mpn, note])
c = cells.CELLS[design.variant['cell']]
if not c['verified']:
    open(f'{FAB}/NOT-FOR-FAB.txt', 'w').write(
        f"No Gerbers: the {c['name']} land pattern is not verified.\n{c['why']}\n"
        f"When it is checked, set verified=True for {design.variant['cell']} in gen/cells.py and run gen/build.sh.\n")
    print(f'{VAR}: review files only (cell land pattern not verified)')
    sys.exit(0)
g = os.path.abspath(f'{FAB}/gerbers'); os.makedirs(g)
run('kicad-cli', 'pcb', 'export', 'gerbers', '-o', g + '/', '--layers',
    'F.Cu,B.Cu,F.Paste,B.Paste,F.SilkS,B.SilkS,F.Mask,B.Mask,Edge.Cuts', f'{PROJ}.kicad_pcb')
run('kicad-cli', 'pcb', 'export', 'drill', '-o', g + '/', '--format', 'excellon', '--excellon-units', 'mm', f'{PROJ}.kicad_pcb')
for side in ('front', 'back'):
    run('kicad-cli', 'pcb', 'export', 'pos', '-o', os.path.abspath(f'{FAB}/{PROJ}-cpl-{side}.csv'), '--format', 'csv',
        '--units', 'mm', '--side', side, f'{PROJ}.kicad_pcb')
print(f'{VAR}: fab package written')
