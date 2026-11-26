"""1U solar face, Rev A: single source of truth for both boards.

One board outline, one back side (connector, blocking diode, bypass jumper, thermistor, test pads),
two front sides:
  si: 24 ANYSOLAR KXOB25-05X3F silicon cells, 8 strings of 3 (for Earth: bench, ground stations)
  tj: 2 AZUR 3G30A 8 x 4 cm triple-junction CICs in series (flight-style)
Both strings make about 5 V at maximum power, so the same power board or logger takes either face.

Usage: design.load('si') or design.load('tj'), then design.parts / design.nets().
"""
from collections import OrderedDict

W, H = 82.5, 98.0            # board, mm. 82.5 x 98 is the usual 1U side-panel size (fits between CDS rails)
NC = '__NC__'
FP_LIB = 'solar_face'

BACK = dict(  # parts on the back, the same on both boards
    D='Diode_SMD:D_SMA',
    JP='Jumper:SolderJumper-2_P1.3mm_Open_RoundedPad1.0x1.5mm',
    TH='Resistor_SMD:R_0603_1608Metric',
    J='Connector_JST:JST_PH_B3B-PH-SM4-TB_1x03-1MP_P2.00mm_Vertical',
    TP='TestPoint:TestPoint_Pad_D2.0mm',
)

VARIANTS = {
    'si': dict(cell='KXOB25_05X3F', strings=8, series=3,
               title='1U solar face, silicon (KXOB25-05X3F x 24)'),
    'tj': dict(cell='CIC_3G30A_8x4', strings=1, series=2,
               title='1U solar face, triple junction (3G30A 8x4 x 2)'),
}

parts = OrderedDict()
variant = None


def add(ref, lib, sym, value, fp, pins, block, mpn='', note='', dnp=False):
    assert ref not in parts, ref
    parts[ref] = dict(ref=ref, lib=lib, sym=sym, value=value, fp=fp, block=block, mpn=mpn, note=note, dnp=dnp,
                      pins={str(k): v for k, v in pins.items()})
    return ref


def load(name):
    global variant
    import cells
    parts.clear()
    v = VARIANTS[name]
    variant = dict(v, name=name)
    c = cells.CELLS[v['cell']]
    blk = '1 Cells'
    n = 0
    for s in range(v['strings']):
        for k in range(v['series']):
            n += 1
            top = 'PV_RAW' if k == 0 else f'S{s + 1}_{k}'
            bot = 'GND' if k == v['series'] - 1 else f'S{s + 1}_{k + 1}'
            add(f'SC{n}', 'Device', 'Solar_Cell', c['name'].split(' ')[-1] if name == 'si' else '3G30A 8x4',
                f'{FP_LIB}:{v["cell"]}', {1: top, 2: bot}, blk, mpn=c['name'],
                note='bond with silicone adhesive; mentor step' if name == 'tj' else 'reflow or hot-air; mentor step')
    blk = '2 Output'
    add('D1', 'Device', 'D_Schottky', 'SS14', BACK['D'], {1: 'PV_OUT', 2: 'PV_RAW'}, blk,
        mpn='SS14 Schottky, 1 A 40 V, SMA',
        note='blocking diode: stops a battery or charger driving current back into the cells in the dark')
    add('JP1', 'Jumper', 'SolderJumper_2_Open', 'BYPASS D1', BACK['JP'], {1: 'PV_RAW', 2: 'PV_OUT'}, blk,
        note='open = diode in; bridge with solder to measure the cells without the diode drop')
    add('TH1', 'Device', 'Thermistor_NTC', '10k NTC', BACK['TH'], {1: 'NTC', 2: 'GND'}, blk,
        mpn='Murata NCP18XH103F03RB (10 kOhm, B 3380 K, 0603)', note='behind the cells; reader supplies the pull-up')
    add('J1', 'Connector_Generic', 'Conn_01x03', 'PV OUT', BACK['J'], {1: 'PV_OUT', 2: 'GND', 3: 'NTC'}, blk,
        mpn='JST B3B-PH-SM4-TB (mates with PHR-3)', note='1 PV+, 2 GND, 3 NTC')
    add('TP1', 'Connector', 'TestPoint', 'PV+', BACK['TP'], {1: 'PV_OUT'}, blk)
    add('TP2', 'Connector', 'TestPoint', 'GND', BACK['TP'], {1: 'GND'}, blk)
    add('TP3', 'Connector', 'TestPoint', 'CELLS', BACK['TP'], {1: 'PV_RAW'}, blk, note='before the diode')
    return parts


def nets():
    out = {}
    for r, p in parts.items():
        for pin, net in p['pins'].items():
            if net != NC:
                out.setdefault(net, []).append((r, pin))
    return out


def string_figures():
    """Desk numbers for the string, from the cell data in cells.py (not measured)."""
    import cells
    c = cells.CELLS[variant['cell']]
    s, p = variant['series'], variant['strings']
    return dict(voc=c['voc'] * s, vmp=c['vmp'] * s, imp=c['imp'] * p, pmp=c['vmp'] * s * c['imp'] * p, sun=c['sun'],
                cell_count=s * p)
