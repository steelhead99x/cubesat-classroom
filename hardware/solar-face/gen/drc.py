"""KiCad DRC on one variant's board.

    python3 drc.py VARIANT [type ...]   run DRC, print counts (and list items of the given types)

Exits 1 when the report can't be written or read, when any item is an error, when an item
has a severity other than error or warning, or when anything is unconnected. Warnings are
printed but do not fail the build. `evaluate` holds the rules; test_drc.py checks them.
"""
import collections
import re
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

KNOWN = {'error', 'warning'}


def evaluate(report):
    """Return (counts, problems) for a KiCad DRC report; no problems means it passes."""
    problems = []
    if not re.search(r'^\*\* End of Report \*\*\s*\Z', report, re.M):
        problems.append('DRC report is incomplete (no "End of Report")')
    sections = {name.rstrip('s'): (int(count), body) for count, name, body in re.findall(
        r'^\*\* Found (\d+) (DRC violations?|unconnected pads?|Footprint errors?) \*\*\s*\n'
        r'(.*?)(?=^\*\*|\Z)', report, re.M | re.S)}
    for need in ('DRC violation', 'unconnected pad'):
        if need not in sections:
            problems.append(f'DRC report has no "{need}s" section')
    counts = collections.Counter()
    for name, (count, body) in sections.items():
        items = re.findall(r'^\[(\w+)\]:(.*?)(?=^\[|\Z)', body, re.M | re.S)
        if len(items) != count:
            problems.append(f'{name}s: declared {count}, parsed {len(items)}')
        for kind, text in items:
            m = re.search(r'Severity: (\w+)', text)
            sev = m.group(1) if m else '?'
            counts[(kind, sev)] += 1
            if sev not in KNOWN:
                problems.append(f'[{kind}] has unknown severity {sev!r}')
            elif sev == 'error' or name != 'DRC violation':
                problems.append(f'[{kind}] {" ".join(text.split())[:200]}')
        if name != 'DRC violation' and count and not items:
            problems.append(f'{count} {name}s')
    return counts, problems


def write_report(var, tmp):
    import pcbnew
    b = pcbnew.LoadBoard(f'../kicad/solar-face-{var}/solar-face-{var}.kicad_pcb')
    rpt = Path(tmp) / f'drc-{var}.rpt'
    if not pcbnew.WriteDRCReport(b, str(rpt), pcbnew.EDA_UNITS_MILLIMETRES, True) or not rpt.is_file():
        sys.exit(f'{var}: could not write the DRC report')
    return rpt.read_text(encoding='utf-8').replace('\r\n', '\n')


def main():
    if len(sys.argv) < 2:
        sys.exit('usage: drc.py VARIANT [type ...]')
    var, show = sys.argv[1], sys.argv[2:]
    with TemporaryDirectory() as tmp:
        t = write_report(var, tmp)
    counts, problems = evaluate(t)
    print(var, dict(counts))
    for bl in re.split(r'\n(?=\[)', t):
        if any(bl.startswith('[' + k + ']') for k in show):
            print('  ', bl.strip().replace('\n', ' | ')[:300])
    for p in problems:
        print('  FAIL', p)
    sys.exit(1 if problems else 0)


if __name__ == '__main__':
    main()
