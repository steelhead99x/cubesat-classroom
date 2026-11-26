"""Project footprints for the solar cells, written to ../kicad/solar_face.pretty as KiCad 7 text.

Both are built from cells.py. Neither land pattern has been checked against the maker's drawing yet
(cells.py `verified`), so both carry a NOT VERIFIED note on their fab layer.
"""
import os
import cells
from sexp import dump, Sym as S

OUT = '../kicad/solar_face.pretty'


def _rect(layer, x0, y0, x1, y1, w=0.1):
    return [S('fp_rect'), [S('start'), x0, y0], [S('end'), x1, y1],
            [S('stroke'), [S('width'), w], [S('type'), S('solid')]], [S('fill'), S('none')], [S('layer'), layer]]


def _text(kind, txt, x, y, layer, size=1.0, hide=False):
    t = [S('fp_text'), S(kind), txt, [S('at'), x, y], [S('layer'), layer]]
    if hide:
        t.append(S('hide'))
    t.append([S('effects'), [S('font'), [S('size'), size, size], [S('thickness'), size * 0.15]]])
    return t


def _pad(num, x, y, w, h, paste=False):
    layers = [S('layers'), 'F.Cu', 'F.Mask'] + (['F.Paste'] if paste else [])
    return [S('pad'), num, S('smd'), S('rect'), [S('at'), x, y], [S('size'), w, h], layers]


def _fp(name, descr, items):
    return [S('footprint'), name, [S('version'), 20221018], [S('generator'), S('solar_face_gen')],
            [S('layer'), 'F.Cu'], [S('descr'), descr], [S('tags'), 'solar cell'],
            [S('attr'), S('smd')]] + items


def cic_3g30a_8x4():
    c = cells.CELLS['CIC_3G30A_8x4']
    bw, bh = c['body']
    hx, hy = bw / 2, bh / 2
    strip_y = hy + 0.5 + 1.5           # 3.0 mm land for the interconnectors, 0.5 mm past the cell edge
    items = [
        _text('reference', 'REF**', -hx + 4, -hy + 2.5, 'F.Fab'),
        _text('value', 'CIC_3G30A_8x4', 0, 0, 'F.Fab'),
        _text('user', 'NOT VERIFIED: check rear polarity and interconnector edge on the AZUR drawing', 0, 3, 'F.Fab', 1.0),
        _rect('F.Fab', -hx, -hy, hx, hy),
        _rect('F.SilkS', -hx - 0.15, -hy - 0.15, hx + 0.15, hy + 0.15, 0.12),
        _rect('F.CrtYd', -hx - 0.35, -hy - 0.35, hx + 0.35, strip_y + 1.85, 0.05),
        _text('user', '+', -hx + 1.5, -hy + 1.4, 'F.SilkS', 1.2),
        _text('user', '-', hx - 1.5, strip_y, 'F.SilkS', 1.2),
        # pad 1: rear contact (+). Bonded with conductive adhesive, so no paste.
        _pad('1', 0, 0, bw - 4.0, bh - 4.0),
        # pad 2: land for the welded interconnectors (-), along the lower long edge.
        _pad('2', 0, strip_y, bw - 4.0, 3.0),
    ]
    return _fp('CIC_3G30A_8x4', f"{c['name']}: rear contact pad (+) and interconnector land (-). "
               f"Body {bw} x {bh} mm. Land pattern NOT VERIFIED (cells.py).", items)


def kxob25_05x3f():
    c = cells.CELLS['KXOB25_05X3F']
    bw, bh = c['body']
    hx, hy = bw / 2, bh / 2
    px = hx - 1.25                     # placeholder: 2.0 x 6.0 mm pads at the two ends, under the body
    items = [
        _text('reference', 'REF**', 0, -hy - 1.0, 'F.Fab', 0.8),
        _text('value', 'KXOB25-05X3F', 0, 0, 'F.Fab', 0.8),
        _text('user', 'PADS NOT VERIFIED', 0, 1.6, 'F.Fab', 0.8),
        _rect('F.Fab', -hx, -hy, hx, hy),
        _rect('F.SilkS', -hx - 0.15, -hy - 0.15, hx + 0.15, hy + 0.15, 0.12),
        _rect('F.CrtYd', -hx - 0.3, -hy - 0.3, hx + 0.3, hy + 0.3, 0.05),
        _text('user', '+', -hx - 1.0, 0, 'F.SilkS', 0.8),
        _pad('1', -px, 0, 2.0, 6.0, paste=True),
        _pad('2', px, 0, 2.0, 6.0, paste=True),
    ]
    return _fp('KXOB25_05X3F', f"{c['name']}: two end pads. Body {bw} x {bh} mm. Land pattern NOT VERIFIED (cells.py).",
               items)


def write():
    os.makedirs(OUT, exist_ok=True)
    for f in (cic_3g30a_8x4(), kxob25_05x3f()):
        open(f'{OUT}/{f[1]}.kicad_mod', 'w').write(dump(f) + '\n')
    print('footprints written:', len(os.listdir(OUT)))


if __name__ == '__main__':
    write()
