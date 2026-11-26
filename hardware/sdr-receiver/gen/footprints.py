"""Project footprints, written to ../kicad/CSC_R1.pretty as plain KiCad 7 text.

LTC_UF24: Analog Devices (Linear) UF24 QFN, drawing 05-08-1697 Rev B: 4 x 4 mm body, 0.5 mm pitch.
Recommended land: 0.70 x 0.25 mm pads whose rows span 3.10 mm (inner edges) to 4.50 mm (outer edges),
exposed pad 2.45 x 2.45 mm. Built from KiCad's generic QFN-24 4x4 footprint with the pads and
exposed pad moved and resized to that drawing."""
import os
from sexp import parse, dump, find, first, Sym as S

FPDIR = '/usr/share/kicad/footprints'
OUT = '../kicad/CSC_R1.pretty'


def ltc_uf24():
    src = f'{FPDIR}/Package_DFN_QFN.pretty/QFN-24-1EP_4x4mm_P0.5mm_EP2.6x2.6mm.kicad_mod'
    t = parse(open(src).read())
    name = 'LTC_UF24_QFN-24-1EP_4x4mm_P0.5mm_EP2.45mm'
    t[1] = name
    centre = (3.10 / 2 + 4.50 / 2) / 2   # 1.90 mm from the body centre
    length = (4.50 - 3.10) / 2           # 0.70 mm
    keep = []
    for item in t:
        if isinstance(item, list) and item and item[0] == 'pad':
            num = item[1]
            at = first(item, 'at'); size = first(item, 'size')
            if num.isdigit() and 1 <= int(num) <= 24:
                x, y = float(at[1]), float(at[2])
                if abs(abs(x) - 1.9375) < 0.01:        # left and right rows
                    at[1] = centre if x > 0 else -centre
                    size[1], size[2] = length, 0.25
                else:                                  # top and bottom rows
                    at[2] = centre if y > 0 else -centre
                    size[1], size[2] = 0.25, length
                keep.append(item)
            continue                                   # drop the old exposed pad and paste windows
        if isinstance(item, list) and item and item[0] in ('descr', 'tags'):
            item[1] = ('Analog Devices UF24 QFN 4x4 mm, 0.5 mm pitch, exposed pad 2.45 mm, '
                       'land per LTC drawing 05-08-1697 Rev B' if item[0] == 'descr' else 'QFN UF24 LTC5584')
        if isinstance(item, list) and item and item[0] == 'fp_text' and len(item) > 2 and item[1] == 'value':
            item[2] = name
        if isinstance(item, list) and item and item[0] == 'property' and item[1] == 'Value':
            item[2] = name
        keep.append(item)
    t[:] = keep
    # exposed pad (copper and mask only), four paste windows (about 50 % coverage), 3 x 3 thermal vias
    t.append([S('pad'), '25', S('smd'), S('rect'), [S('at'), 0, 0], [S('size'), 2.45, 2.45],
              [S('layers'), 'F.Cu', 'F.Mask']])
    for i in (-0.6, 0.6):
        for j in (-0.6, 0.6):
            t.append([S('pad'), '', S('smd'), S('roundrect'), [S('at'), i, j], [S('size'), 0.85, 0.85],
                      [S('layers'), 'F.Paste'], [S('roundrect_rratio'), 0.25]])
    for i in (-0.8, 0.0, 0.8):
        for j in (-0.8, 0.0, 0.8):
            t.append([S('pad'), '25', S('thru_hole'), S('circle'), [S('at'), i, j], [S('size'), 0.4, 0.4],
                      [S('drill'), 0.2], [S('layers'), '*.Cu']])
    return name, t


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    name, t = ltc_uf24()
    open(f'{OUT}/{name}.kicad_mod', 'w').write(dump(t) + '\n')
    print('wrote', f'{OUT}/{name}.kicad_mod')
