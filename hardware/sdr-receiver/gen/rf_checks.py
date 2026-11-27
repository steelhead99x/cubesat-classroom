"""Desk checks on the values in design.py. These are calculations, not measurements.

1. ADF4351 loop filter: open-loop bandwidth and phase margin at both bands and at the
   ends of the data sheet's VCO-gain range.
2. Band filters: modelled insertion loss and rejection (inductor Q 40, capacitor Q 200).
3. 50-ohm microstrip width on the outer layer over the first inner (ground) layer.

Run: python3 rf_checks.py
"""
import math
import re
import numpy as np
import boardcfg
from calc import s21
design = boardcfg.design()

UNIT = {'p': 1e-12, 'n': 1e-9, 'u': 1e-6}


def value(text):
    """'4p7' -> 4.7e-12, '22nH' -> 22e-9, '3n9' -> 3.9e-9, '120R' -> 120."""
    t = text.replace('H', '').replace('R', '')
    m = re.fullmatch(r'(\d+)([pnu])(\d*)', t)
    if m:
        return float(m.group(1) + '.' + (m.group(3) or '0')) * UNIT[m.group(2)]
    m = re.fullmatch(r'(\d+(?:\.\d+)?)([pnu])', t)
    if m:
        return float(m.group(1)) * UNIT[m.group(2)]
    return float(t)


def part(prefix, *nets):
    refs = [r for r, p in design.parts.items()
            if r.rstrip('0123456789') == prefix and all(n in p['pins'].values() for n in nets)]
    assert len(refs) == 1, (prefix, nets, refs)
    return value(design.parts[refs[0]]['value'])


def loop_check():
    C1 = part('C', 'PLL_CP', 'GND'); R2 = part('R', 'PLL_CP', 'PLL_LF2'); C2 = part('C', 'PLL_LF2', 'GND')
    R3 = part('R', 'PLL_CP', 'PLL_VTUNE'); C3 = part('C', 'PLL_VTUNE', 'GND')
    icp = 2.5e-3; pfd = 25e6
    fs = np.logspace(2.5, 6.5, 4000); s = 1j * 2 * math.pi * fs
    zp = 1 / (s * C1 + 1 / (R2 + 1 / (s * C2)))
    zt = zp * (1 / (s * C3)) / (zp + R3 + 1 / (s * C3))
    print(f'Loop filter: C1 {C1*1e9:.3g} nF, R2 {R2:.0f} ohm, C2 {C2*1e9:.3g} nF, R3 {R3:.0f} ohm, C3 {C3*1e12:.0f} pF, Icp 2.5 mA')
    worst = 180
    for band, fout, div in (('2 m', 145.8e6, 16), ('70 cm', 437e6, 8)):
        n = fout * div / pfd
        for kv in (40e6, 80e6):
            h = icp / (2 * math.pi) * zt * 2 * math.pi * kv / s / n
            i = np.where(np.abs(h) < 1)[0][0]
            pm = 180 + math.degrees(np.angle(h[i])); worst = min(worst, pm)
            print(f'  {band:5} N={n:6.2f} Kv={kv/1e6:.0f} MHz/V: bandwidth {fs[i]/1e3:5.1f} kHz, phase margin {pm:4.1f} deg')
    return worst


def filter_check():
    probes = {'F2': [88e6, 98e6, 108e6, 144e6, 146e6, 437e6],
              'F70': [146e6, 300e6, 435e6, 438e6, 600e6, 915e6]}
    for pfx, lad in design.LADDERS.items():
        net = []
        node = lad['nin']
        for ref in lad['chain']:
            p = design.parts[ref]
            nxt = [v for v in p['pins'].values() if v != node][0]
            kind = 'Cs' if ref.startswith('C') else 'L'
            net.append((kind, value(p['value'])))
            for sh in lad['shunts'].get(nxt, []):
                net.append(('Lp' if sh.startswith('L') else 'C', value(design.parts[sh]['value'])))
            node = nxt
        print(f'{pfx} filter:', '  '.join(f'{f/1e6:.0f} MHz {s21(net, f):.1f} dB' for f in probes[pfx]))


def microstrip(w, h, er, t=0.035e-3):
    """Hammerstad-Jensen with thickness correction; returns ohms."""
    we = w + t / math.pi * math.log(4 * math.e / math.sqrt((t / h) ** 2 + (t / (w * math.pi + 1.1 * t * math.pi)) ** 2))
    u = we / h
    a = 1 + math.log((u ** 4 + (u / 52) ** 2) / (u ** 4 + 0.432)) / 49 + math.log(1 + (u / 18.1) ** 3) / 18.7
    b = 0.564 * ((er - 0.9) / (er + 3)) ** 0.053
    ee = (er + 1) / 2 + (er - 1) / 2 * (1 + 10 / u) ** (-a * b)
    f = 6 + (2 * math.pi - 6) * math.exp(-(30.666 / u) ** 0.7528)
    return 60 / math.sqrt(ee) * math.log(f / u + math.sqrt(1 + (2 / u) ** 2))


def width_check():
    h, er = 0.2104e-3, 4.4   # assumed: 1.6 mm 4-layer with 7628 prepreg between layers 1 and 2
    ws = np.arange(0.1e-3, 0.6e-3, 0.005e-3)
    w50 = ws[np.argmin([abs(microstrip(w, h, er) - 50) for w in ws])]
    print(f'Microstrip (dielectric {h*1e3:.3f} mm, er {er}): 0.35 mm = {microstrip(0.35e-3, h, er):.1f} ohm; 50 ohm = {w50*1e3:.3f} mm')


if __name__ == '__main__':
    pm = loop_check()
    filter_check()
    width_check()
    print('worst phase margin', round(pm, 1), 'deg:', 'OK (>= 45)' if pm >= 45 else 'LOW')
