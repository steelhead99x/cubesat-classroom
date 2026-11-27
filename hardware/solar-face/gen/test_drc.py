"""Tests for the solar-face DRC rules. Run: python3 test_drc.py (no KiCad needed)."""
from drc import evaluate

HEAD = '** Drc report for x.kicad_pcb **\n** Created on 2026-11-27 **\n\n'
TAIL = '\n** Found 0 Footprint errors **\n\n** End of Report **\n'


def report(items, unconnected=''):
    n = unconnected.count('\n[') + unconnected.startswith('[')
    return (HEAD + f'** Found {len(items)} DRC violations **\n' + ''.join(items)
            + f'\n** Found {n} unconnected pads **\n' + unconnected + TAIL)


def item(kind, sev):
    return f'[{kind}]: something\n    Rule: r; Severity: {sev}\n    @(1 mm, 2 mm): Pad 1 of D1\n'


def fails(t):
    return bool(evaluate(t)[1])


def test_clean_passes():
    assert not fails(report([]))


def test_warning_passes():
    assert not fails(report([item('silk_overlap', 'warning')]))


def test_error_fails():
    # Review finding: "; Severity: error" never matched the old pattern, so errors passed.
    assert fails(report([item('clearance', 'error')]))


def test_unknown_severity_fails():
    assert fails(report([item('clearance', 'exclusion')]))
    assert fails(report(['[clearance]: no severity line\n']))


def test_unconnected_fails():
    assert fails(report([], '[unconnected_items]: Missing connection\n    Rule: r; Severity: error\n'))


def test_incomplete_or_missing_report_fails():
    assert fails('')
    assert fails(report([]).replace('** End of Report **', ''))
    assert fails(report([]).replace('unconnected pads', 'other things'))
    assert fails(report([item('clearance', 'warning')]).replace('Found 1 DRC', 'Found 2 DRC'))


if __name__ == '__main__':
    for name, t in list(globals().items()):
        if name.startswith('test_'):
            t()
            print('ok ', name)
