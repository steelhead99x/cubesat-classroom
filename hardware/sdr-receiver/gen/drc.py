"""KiCad DRC on the board.

    python3 drc.py nofill [type ...]   report counts (and list items of the given types)
    python3 drc.py fill   [type ...]   refill the pours first
    python3 drc.py gate [output-report] exit 1 unless the board is clean apart from the
                                       documented exception below

The only accepted DRC items are copper-to-edge clearance on the SMA edge-launch
connector J3: its pads must reach the board edge by design (README, "Checks").
Anything else, or any unconnected pad, fails the gate.
"""
import collections
import re
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

import boardcfg
ALLOWED_TYPE = 'copper_edge_clearance'
ALLOWED_REF = boardcfg.EDGE_REF or 'none'   # Rev A: SMA edge connector J3; the pedal boards allow none
ALLOWED_MAX = boardcfg.EDGE_MAX            # Rev A: its two pads, on the layers they occupy


def gate(report):
    """Return a list of reasons the report fails; empty means it passes."""
    problems = []
    sections = re.findall(
        r'^\*\* Found (\d+) (DRC violations?|unconnected pads?|Footprint errors?) \*\*\s*\n'
        r'(.*?)(?=^\*\*|\Z)', report, re.M | re.S
    )
    categories = ['DRC violation', 'unconnected pad', 'Footprint error']
    if [name.rstrip('s') for _, name, _ in sections] != categories or not re.search(
        r'^\*\* End of Report \*\*\s*\Z', report, re.M
    ):
        return ['missing or incomplete DRC report sections']
    blocks = []
    for count, name, body in sections:
        entries = [line for line in body.splitlines() if line.startswith('[')]
        if int(count) != len(entries):
            problems.append(f'{name}: declared {count}, parsed {len(entries)}')
        if not name.startswith('DRC ') and int(count):
            problems.append(f'{count} {name}')
        if name.startswith('DRC '):
            blocks = re.findall(r'^\[(\w+)\]:(.*?)(?=^\[|\Z)', body, re.M | re.S)
            if len(entries) != len(blocks):
                problems.append('unrecognized DRC entry')
    allowed = 0
    for kind, bl in blocks:
        if kind == ALLOWED_TYPE and re.search(rf'\bPad \S+ \[[^\]]*\] of {ALLOWED_REF} on', bl):
            allowed += 1
            continue
        problems.append(f'{kind}: ' + ' '.join(bl.split())[:200])
    if allowed > ALLOWED_MAX:
        problems.append(f'{allowed} edge-clearance items on {ALLOWED_REF}, expected at most {ALLOWED_MAX}')
    return problems


def run(fill):
    import pcbnew
    with TemporaryDirectory() as tmp:
        report = Path(tmp) / 'board.rpt'
        if hasattr(pcbnew, 'EDA_UNITS_MILLIMETRES'):
            board = pcbnew.LoadBoard(boardcfg.PCB)
            if fill:
                pcbnew.ZONE_FILLER(board).Fill(board.Zones())
            if not pcbnew.WriteDRCReport(board, str(report), pcbnew.EDA_UNITS_MILLIMETRES, True):
                sys.exit('could not write DRC report')
        else:
            # The newer pcbnew binding cannot run DRC outside KiCad's application.
            command = [
                'kicad-cli', 'pcb', 'drc', '--all-track-errors',
                '--units', 'mm', '--output', str(report),
            ]
            if fill:
                command.append('--refill-zones')
            subprocess.run([*command, boardcfg.PCB], check=True)
        return report.read_bytes()


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else 'nofill'
    if mode == 'gate' and len(sys.argv) not in (2, 3):
        sys.exit('usage: drc.py gate [output-report]')
    report = run(mode == 'fill')
    t = report.decode('utf-8').replace('\r\n', '\n')
    c = collections.Counter(re.findall(r'^\[(\w+)\]', t, re.M))
    print(dict(c))
    m = re.search(r'Found (\d+) unconnected', t)
    print('unconnected:', m.group(1) if m else '?')
    if mode == 'gate':
        problems = gate(t)
        for p in problems:
            print('DRC FAIL', p)
        print('DRC gate:', 'FAIL' if problems else (f'pass (only the documented {ALLOWED_REF} edge items)' if ALLOWED_MAX else 'pass (no DRC items)'))
        if problems:
            sys.exit(1)
        if len(sys.argv) == 3:
            Path(sys.argv[2]).write_bytes(report)
        return
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
