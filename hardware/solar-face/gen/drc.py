"""KiCad DRC on one variant's board. Exits 1 on any error or unconnected item."""
import sys, re, collections, pcbnew
VAR = sys.argv[1]
b = pcbnew.LoadBoard(f'../kicad/solar-face-{VAR}/solar-face-{VAR}.kicad_pcb')
rpt = f'/tmp/drc-{VAR}.rpt'
pcbnew.WriteDRCReport(b, rpt, pcbnew.EDA_UNITS_MILLIMETRES, True)
t = open(rpt).read()
blocks = re.split(r'\n(?=\[)', t)
errs = [bl for bl in blocks if bl.startswith('[') and 'error' in bl.split('\n')[1] if '\n' in bl] if False else []
sev = collections.Counter()
for bl in blocks:
    m = re.match(r'\[(\w+)\]', bl)
    if m:
        s = re.search(r'; (error|warning)', bl)
        sev[(m.group(1), s.group(1) if s else '?')] += 1
un = re.search(r'Found (\d+) unconnected', t)
un = int(un.group(1)) if un else -1
print(VAR, dict(sev), 'unconnected', un)
show = sys.argv[2:]
for bl in blocks:
    if any(bl.startswith('[' + k + ']') for k in show): print('  ', bl.strip().replace('\n', ' | ')[:300])
bad = sum(v for (k, s), v in sev.items() if s == 'error') + max(un, 0)
sys.exit(1 if bad else 0)
