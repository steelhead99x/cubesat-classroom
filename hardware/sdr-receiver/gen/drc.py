"""KiCad DRC on the board.

    python3 drc.py nofill [type ...]   report counts (and list items of the given types)
    python3 drc.py fill   [type ...]   refill the pours first
    python3 drc.py gate                exit 1 unless the board is clean apart from the
                                       documented exception below

The only accepted DRC items are copper-to-edge clearance on the SMA edge-launch
connector J3: its pads must reach the board edge by design (README, "Checks").
Anything else, or any unconnected pad, fails the gate.
"""
import collections
import re
import sys

ALLOWED_TYPE = 'copper_edge_clearance'
ALLOWED_REF = 'J3'          # SMA edge connector
ALLOWED_MAX = 5             # its two pads, on the layers they occupy


def gate(report):
    """Return a list of reasons the report fails; empty means it passes."""
    problems = []
    m = re.search(r'Found (\d+) unconnected', report)
    if not m:
        problems.append('report has no unconnected-pad count')
    elif int(m.group(1)):
        problems.append(f'{m.group(1)} unconnected pads')
    blocks = [b for b in re.split(r'\n(?=\[)', report) if re.match(r'\[\w+\]', b)]
    allowed = 0
    for bl in blocks:
        kind = re.match(r'\[(\w+)\]', bl).group(1)
        if kind == ALLOWED_TYPE and re.search(rf'\bPad \S+ \[[^\]]*\] of {ALLOWED_REF} on', bl):
            allowed += 1
            continue
        problems.append(f'{kind}: ' + ' '.join(bl.split())[:200])
    if allowed > ALLOWED_MAX:
        problems.append(f'{allowed} edge-clearance items on {ALLOWED_REF}, expected at most {ALLOWED_MAX}')
    return problems


def run(fill):
    import pcbnew
    b = pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
    if fill:
        pcbnew.ZONE_FILLER(b).Fill(b.Zones())
    pcbnew.WriteDRCReport(b, '/tmp/drc.rpt', pcbnew.EDA_UNITS_MILLIMETRES, True)
    return open('/tmp/drc.rpt').read()


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else 'nofill'
    t = run(mode == 'fill')
    c = collections.Counter(re.findall(r'^\[(\w+)\]', t, re.M))
    print(dict(c))
    m = re.search(r'Found (\d+) unconnected', t)
    print('unconnected:', m.group(1) if m else '?')
    if mode == 'gate':
        problems = gate(t)
        for p in problems:
            print('DRC FAIL', p)
        print('DRC gate:', 'FAIL' if problems else f'pass (only the documented {ALLOWED_REF} edge items)')
        sys.exit(1 if problems else 0)
    blocks = re.split(r'\n(?=\[)', t)
    for k in sys.argv[2:]:
        n = 0
        for bl in blocks:
            if bl.startswith('[' + k + ']'):
                print(bl.strip().replace('\n', ' | ')[:330])
                n += 1
                if n >= 40:
                    break


if __name__ == '__main__':
    main()
