"""Stage 1 for the CSC-R1P pedal main board: outline, footprints, placement, RF hand-routing.

Board: 54 x 104 mm, centred in a Hammond 1590N1 (125B), notched round the four lid-screw posts,
with a slot at the bottom for the footswitch body. x runs across the box, y along it (0 = top
end). The RF chain runs right to left along y = RFY (antenna U.FL at the right edge), then turns
down into the demodulator; the synthesizer sits below it and the ADC to its right. Power is top
left, the XIAO top right, audio and the control-board link on the right.
Writes ../pedal/kicad/csc-r1p.kicad_pcb (run with CSC_BOARD=csc-r1p).
"""
import math

import pcbnew

import boardcfg
from pcbkit import Board, BOX_POSTS, POST_CLEAR, STANDOFFS, notch

assert boardcfg.PROJ == 'csc-r1p', 'run with CSC_BOARD=csc-r1p'
d = boardcfg.design()
k = Board(d)
P = d.parts
one, sel, pinof, pos, put, pin_at, at, inline, path, connect = (k.one, k.sel, k.pinof, k.pos, k.put, k.pin_at, k.at,
                                                                 k.inline, k.path, k.connect)
W, H = 54.0, 104.0
SLOT = (19.0, 35.0, 78.0)          # footswitch slot (16 mm wide, for a 12 mm soft-touch switch body): x from, x to, top y

# ------------------------------------------------------------------ outline
(px0, py0), (px1, _), (_, py2), _ = BOX_POSTS
r = POST_CLEAR
dx = math.sqrt(r * r - (0 - py0) ** 2); dy = math.sqrt(r * r - px0 ** 2)

tl, tr, bl, br = (px0, py0), (px1, py0), (px0, py2), (px1, py2)
k.outline([
    ('L', (px0 + dx, 0)), ('L', (px1 - dx, 0)),
    notch(tr, r, (px1 - dx, 0), (W, py0 + dy)),
    ('L', (W, py2 - dy)),
    notch(br, r, (W, py2 - dy), (px1 - dx, H)),
    ('L', (SLOT[1], H)), ('L', (SLOT[1], SLOT[2])), ('L', (SLOT[0], SLOT[2])), ('L', (SLOT[0], H)),
    notch(bl, r, (px0 + dx, H), (0, py2 - dy)),
    ('L', (0, py0 + dy)),
    notch(tl, r, (0, py0 + dy), (px0 + dx, 0)),
])

from pcbkit import AutoPlace
poly_pts = [(pcbnew.ToMM(s_.GetStart().x) - 60, pcbnew.ToMM(s_.GetStart().y) - 50) for s_ in k.board.GetDrawings() if s_.GetLayer() == pcbnew.Edge_Cuts]
ap = AutoPlace(k, poly_pts)

# ------------------------------------------------------------------ RF chain, right to left at y = RFY
RFY = 29.0
ant = one('J', 'ANT'); pin_at(ant, '1', 50.4, RFY, 0)
c_dc = one('C', 'ANT', 'RF_A'); e = at(c_dc, 'ANT', 47.9, RFY, 180)
l_bt = one('L', 'BT_FILT', 'ANT'); at(l_bt, 'ANT', 48.55, RFY - 1.35, 180)
l_st = one('L', 'RF_A', 'GND'); at(l_st, 'RF_A', e[0] + 0.1, RFY + 1.7, 90)
bav = one('D', 'RF_A')
c = one('C', 'RF_A', 'LNA_G'); e = at(c, 'RF_A', e[0] - 2.4, RFY, 180)
lna = one('U', 'LNA_G', 'LNA_D'); pin_at(lna, '1', e[0] - 1.5, RFY, 270)
p3 = pos(lna, '3'); assert p3[0] < pos(lna, '1')[0] and abs(p3[1] - RFY) < 0.01, (p3, pos(lna, '1'))
l = one('L', '5V_LNA', 'LNA_D'); e2 = at(l, 'LNA_D', p3[0] - 0.4, RFY + 5.0, 90)
cb = [x for x in sel('C', '5V_LNA', 'GND') if P[x]['value'] in ('100n', '1n')]
at(cb[0], '5V_LNA', e2[0] - 1.3, e2[1], 180); at(cb[1], '5V_LNA', e2[0] + 1.3, e2[1], 0)
c = one('C', 'LNA_D', 'LNA_OUT'); e = at(c, 'LNA_D', p3[0] - 1.5, RFY, 180)


def breakpoint(n_a, n_b, x, ang=180):
    """U.FL, 0R link, U.FL, along direction ang from (x, RFY)."""
    ua = [q for q in sel('J', n_a) if P[q]['sym'] == 'Conn_Coaxial' and 'BP' in P[q]['value']][0]
    ub = [q for q in sel('J', n_b) if P[q]['sym'] == 'Conn_Coaxial' and 'BP' in P[q]['value']][0]
    pin_at(ua, '1', x, RFY, 90)
    r0 = one('R', n_a, n_b); at(r0, n_a, x - 3.25, RFY, ang)
    pin_at(ub, '1', x - 7.4, RFY, 270)
    return x - 7.4


x = breakpoint('LNA_OUT', 'FILT_IN', e[0] - 3.1)
sw1 = one('U', 'FILT_IN', 'F2_IN'); pin_at(sw1, '5', x - 3.4, RFY, 0)
assert pos(sw1, '1')[0] < pos(sw1, '5')[0]
xs = pos(sw1, '1')[0] - 1.9


def ladder(pfx, nin, y, out):
    L_ = d.LADDERS[pfx]; xx = xs; node = nin; last = None
    for q in L_['chain']:
        e_ = at(q, node, xx, y, 180); node = [v for kk, v in P[q]['pins'].items() if v != node][0]
        xx = e_[0] - 1.1; last = e_
        for k_, sh in enumerate(L_['shunts'].get(node, [])):
            at(sh, node, e_[0] - 0.6 - 1.1 * k_, y + out * 1.15, 90 if out > 0 else 270)
    assert not L_['shunts'].get(nin)
    return last[0]


xe = min(ladder('F2', 'F2_IN', RFY + 2.4, +1), ladder('F70', 'F70_IN', RFY - 2.4, -1))
sw2 = one('U', 'FILT_OUT', 'F2_OUT'); pin_at(sw2, '5', xe - 3.4, RFY, 180)
assert pos(sw2, '1')[0] > pos(sw2, '5')[0]
ap.block(bav, 43.6, RFY - 3.4, 270)   # input clamp above the RF line, clear of the U.FL and the bias-tee choke
FILTNETS = sorted({v for pf in ('F2', 'F70') for q in d.LADDERS[pf]['chain'] for v in P[q]['pins'].values()})

# ------------------------------------------------------------------ BP3 down the left edge, then into the balun
CX, CY = 22.0, 56.0                 # demodulator centre (Rev A orientation: RF in from the left, LO out to the right)
RFY2 = CY + 1.0                     # height of the RF line into the demodulator (pin 5 sits 0.25 above it)
BPX = 4.3
s5 = pos(sw2, '5')
ua = [q for q in sel('J', 'FILT_OUT') if 'BP3' in P[q]['value']][0]
ub = [q for q in sel('J', 'DEM_RF') if 'BP3' in P[q]['value']][0]
pin_at(ua, '1', BPX, RFY + 4.4, 180)
r0 = one('R', 'FILT_OUT', 'DEM_RF'); at(r0, 'FILT_OUT', BPX, RFY + 7.65, 90)
pin_at(ub, '1', BPX, RFY + 11.8, 0)
# Leave pin 5 sideways first: pin 6 (the switch's supply) sits right below it.
jx = s5[0] - 1.1
path('FILT_OUT', [s5, (jx, s5[1]), (jx, s5[1] + 1.3), (BPX, s5[1] + 1.3 + abs(BPX - jx)), pos(ua, '1')]); path('FILT_OUT', [pos(ua, '1'), pos(r0, pinof(r0, 'FILT_OUT'))])
path('DEM_RF', [pos(r0, pinof(r0, 'DEM_RF')), pos(ub, '1')])
cm = sel('C', 'DEM_RF', 'GND')[0]; rm = one('R', 'DEM_RF', 'DEM_RF_M'); cm2 = sel('C', 'DEM_RF_M', 'GND')[0]
at(cm, 'DEM_RF', BPX + 1.1, RFY2, 90)
path('DEM_RF', [pos(ub, '1'), (BPX, RFY2), pos(cm, pinof(cm, 'DEM_RF'))])
e = at(rm, 'DEM_RF', BPX + 2.2, RFY2, 0)
path('DEM_RF', [pos(cm, pinof(cm, 'DEM_RF')), pos(rm, pinof(rm, 'DEM_RF'))])
at(cm2, 'DEM_RF_M', e[0], RFY2 + 1.1, 90)
# the Rev A demodulator cluster from here on, moved to (CX, CY)
bal = one('T', 'DEM_RF_M'); pin_at(bal, '6', e[0] + 2.2, RFY2 - 1.27, -90)
b1 = pos(bal, '1'); b3 = pos(bal, '3'); assert b1[0] > pos(bal, '6')[0] and b1[1] < b3[1]
l6 = one('L', 'RF_BAL_P', 'RF_BAL_N'); at(l6, 'RF_BAL_P', b1[0] + 1.7, RFY2 - 0.48, 90)
cp = one('C', 'RF_BAL_P', 'DEM_RFP'); cn = one('C', 'RF_BAL_N', 'DEM_RFN')
ep = at(cp, 'RF_BAL_P', b1[0] + 2.9, RFY2 - 0.5, 0); at(cn, 'RF_BAL_N', b1[0] + 2.9, RFY2 + 0.5, 0)
dem = one('U', 'DEM_RFP', 'LO_P'); pin_at(dem, '5', ep[0] + 2.9, RFY2 - 0.25, 0)
CX, CY = k.centre(dem)
print('demod centre', round(CX, 2), round(CY, 2))
pll = one('U', 'PLL_CP')
dn = pos(dem, '16'); dp = pos(dem, '15'); assert dn[1] < dp[1]
ca = one('C', 'LOA_P', 'LO_N'); cb_ = one('C', 'LOA_N', 'LO_P')
ea = at(ca, 'LO_N', dn[0] + 2.9, CY - 0.5, 0); at(cb_, 'LO_P', dn[0] + 2.9, CY + 0.5, 0)
ra = one('R', '3V3_VCO', 'LOA_P'); rb = one('R', '3V3_VCO', 'LOA_N')
at(ra, 'LOA_P', ea[0], CY - 1.75, 270); at(rb, 'LOA_N', ea[0], CY + 1.75, 90)
pin_at(pll, '12', ea[0] + 2.5, CY - 0.25, -90)
PX, PY = k.centre(pll)
assert abs(PY - CY) < 0.01 and pos(pll, '13')[1] > pos(pll, '12')[1]
print('pll centre', round(PX, 2), round(PY, 2))
p14 = pos(pll, '14'); p15 = pos(pll, '15')
rbm = one('R', '3V3_VCO', 'LOB_N'); at(rbm, 'LOB_N', PX - 3.6, CY + 3.7, 90)
path('LOB_N', [p15, (PX - 3.6, p15[1]), (PX - 3.6, CY + 3.7)], 0.25)
rbp = one('R', '3V3_VCO', 'LOB_P'); at(rbp, 'LOB_P', PX - 5.3, CY + 4.2, 180)
cm_ = one('C', 'LOB_P', 'LO_MON'); e = at(cm_, 'LOB_P', PX - 4.2, CY + 5.7, 90)
path('LOB_P', [p14, (PX - 4.2, p14[1]), (PX - 4.2, CY + 5.7)], 0.25); path('LOB_P', [(PX - 4.2, CY + 4.2), (PX - 5.3, CY + 4.2)], 0.25)
um = one('J', 'LO_MON'); pin_at(um, '1', e[0], e[1] + 1.9, 270); path('LO_MON', [e, pos(um, '1')])
# RF hand routes
for n in ['ANT', 'RF_A', 'LNA_G', 'LNA_D', 'LNA_OUT', 'FILT_IN'] + FILTNETS + ['DEM_RF_M', 'RF_BAL_P', 'RF_BAL_N']:
    connect(n, 0.35)
csh = one('C', 'DEM_RFP', 'DEM_RFN'); at(csh, 'DEM_RFP', ep[0] + 0.9, RFY2 - 0.48, 90)
for n, cc, pn in (('DEM_RFP', cp, '5'), ('DEM_RFN', cn, '6')):
    q = pos(dem, pn); a_ = pos(cc, pinof(cc, n)); b_ = pos(csh, pinof(csh, n)); path(n, [a_, (b_[0], a_[1]), b_, (q[0] - 0.9, q[1]), q], 0.2)
for sgn, na, nl, pa, pd in ((-1, 'LOA_P', 'LO_N', '12', '16'), (1, 'LOA_N', 'LO_P', '13', '15')):
    pp = pos(pll, pa); cpad = (ea[0], CY + sgn * 0.5); rr = ra if sgn < 0 else rb
    path(na, [pp, (ea[0] + 0.49, pp[1]), cpad, pos(rr, pinof(rr, na))], 0.22)
    dd_ = pos(dem, pd); path(nl, [(dn[0] + 2.9, CY + sgn * 0.5), (dn[0] + 2.3, dd_[1]), dd_], 0.22)

# ------------------------------------------------------------------ demodulator support (as Rev A)
offs = {'IP': (-2.65, -1.55), 'IN': (0.65, -0.45), 'QP': (1.75, 2.85), 'QN': (5.05, 3.95)}
for kk, (xp, xs_) in offs.items():
    at(one('R', '5V_DEMOD', 'BB_' + kk), 'BB_' + kk, CX + xp, CY - 5.0, 270); at(one('R', 'BB_' + kk, 'BBF_' + kk), 'BB_' + kk, CX + xs_, CY - 5.0, 270)
at(one('C', 'BBF_IP', 'BBF_IN'), 'BBF_IP', CX - 1.48, CY - 7.4, 0); at(one('C', 'BBF_QP', 'BBF_QN'), 'BBF_QP', CX + 2.92, CY - 7.4, 0)
for kk, xx, adc in (('IP', -1.9, 'ADC_IN1P'), ('IN', -0.1, 'ADC_IN1M'), ('QP', 2.5, 'ADC_IN2P'), ('QN', 4.3, 'ADC_IN2M')):
    cc_ = one('C', 'BBF_' + kk, adc); at(cc_, 'BBF_' + kk, CX + xx, CY - 8.8, 270)
    connect('BBF_' + kk, 0.2, only=[one('R', 'BB_' + kk, 'BBF_' + kk), one('C', 'BBF_IP', 'BBF_IN') if kk[0] == 'I' else one('C', 'BBF_QP', 'BBF_QN'), cc_])
at(one('C', 'DEM_CMQ', 'DEM_VCAP'), 'DEM_CMQ', CX + 3.1, CY - 1.5, 0); at(one('C', 'DEM_CMI', 'DEM_VCAP'), 'DEM_CMI', CX + 2.7, CY - 3.3, 0)
at(one('C', 'DEM_REF', 'GND'), 'DEM_REF', CX - 2.3, CY - 3.3, 180)
dv = sel('C', '5V_DEMOD', 'GND')
c10u = [q for q in dv if P[q]['value'] == '10u'][0]; c1u = [q for q in dv if P[q]['value'] == '1u'][0]; c10n = [q for q in dv if P[q]['value'] == '10n'][0]
at(c10n, '5V_DEMOD', CX + 0.25, CY + 4.6, 90); at(c1u, '5V_DEMOD', CX + 0.25, CY + 6.8, 90)
at(one('R', 'DEM_VBIAS', 'GND'), 'DEM_VBIAS', CX - 1.1, CY + 4.6, 90); at(one('R', 'DEMOD_EN', 'GND'), 'DEMOD_EN', CX - 2.4, CY + 4.6, 90)
fbd = one('FB', '+5V', '5V_DEMOD'); at(fbd, '5V_DEMOD', CX - 1.5, CY + 10.2, 180); at(c10u, '5V_DEMOD', CX + 0.0, CY + 10.2, 0)
# trim network as Rev A; the trim DAC itself sits above it (its four outputs are left to the router)
for i, (dch, t) in enumerate((('DAC_D', 'TRIM_IP2Q'), ('DAC_C', 'TRIM_DCOQ'), ('DAC_B', 'TRIM_DCOI'), ('DAC_A', 'TRIM_IP2I'))):
    x0 = CX - 4.6 - 2.4 * i; y0 = CY - 7.8
    rs_ = one('R', dch, t); at(rs_, t, x0, y0, 180); at(one('R', t, 'GND'), t, x0, y0 + 1.0, 180); at(one('C', t, 'GND'), t, x0, y0 + 2.0, 180)
    connect(t, 0.2, only=[rs_, one('R', t, 'GND'), one('C', t, 'GND')])

# ------------------------------------------------------------------ ADC (as Rev A, above the demodulator)
AX, AY = CX + 1.2, CY - 15.1
adc = one('U', 'ADC_AREG'); put(adc, AX, AY, 0)
va = [q for q in sel('C', '3V3_ADC', 'GND') if P[q]['block'].startswith('5')]
at([q for q in va if P[q]['value'] == '100n'][0], '3V3_ADC', AX - 3.2, AY - 2.3, 180); at([q for q in va if P[q]['value'] == '10u'][0], '3V3_ADC', AX - 6.0, AY - 2.9, 180)
ar = sel('C', 'ADC_AREG', 'GND'); at([q for q in ar if P[q]['value'] == '100n'][0], 'ADC_AREG', AX - 3.2, AY - 1.2, 180); at([q for q in ar if P[q]['value'] == '10u'][0], 'ADC_AREG', AX - 6.0, AY - 0.6, 180)
at(one('C', 'ADC_VREF', 'GND'), 'ADC_VREF', AX - 3.5, AY + 1.3, 180)
dr = sel('C', 'ADC_DREG', 'GND'); at([q for q in dr if P[q]['value'] == '100n'][0], 'ADC_DREG', AX - 1.9, AY - 3.5, 270); at([q for q in dr if P[q]['value'] == '10u'][0], 'ADC_DREG', AX - 3.4, AY - 5.6, 180)
io = [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('5')][0]; at(io, '3V3_DIG', AX + 1.9, AY - 3.5, 270)
for n, xx in (('ADC_FSYNC', -1.75), ('ADC_BCLK', -0.8), ('ADC_SDOUT', 0.15)): at(one('R', n), n, AX + xx, AY - 5.6, 270)
at(one('R', 'ADC_SHDNZ', 'GND'), 'ADC_SHDNZ', AX + 3.4, AY + 0.75, 0)
osc = one('Y', 'MCLK_OSC_R'); put(osc, AX + 6.5, AY - 2.0, 0)
at(one('R', 'MCLK_OSC_R', 'ADC_MCLK'), 'MCLK_OSC_R', AX + 9.7, AY - 2.8, 270)
at(one('FB', '3V3_DIG', '3V3_MCLK'), '3V3_MCLK', AX + 4.6, AY - 4.7, 0); at(one('C', '3V3_MCLK', 'GND'), '3V3_MCLK', AX + 6.0, AY + 0.9, 0)

# ------------------------------------------------------------------ PLL support (as Rev A)
pl = sel('C', '3V3_PLL', 'GND'); p100 = [q for q in pl if P[q]['value'] == '100n']; p10 = [q for q in pl if P[q]['value'] == '10p']; p10u = [q for q in pl if P[q]['value'] == '10u'][0]
at(p100[0], '3V3_PLL', PX - 0.45, PY - 4.4, 270); at(p10[0], '3V3_PLL', PX - 2.6, PY - 4.5, 270)
at(p100[1], '3V3_PLL', PX - 3.8, PY - 1.25, 270); at(p10[1], '3V3_PLL', PX - 3.8, PY - 3.4, 270)
at(p100[2], '3V3_PLL', PX + 4.6, PY - 1.9, 270); at(p10[2], '3V3_PLL', PX + 5.7, PY - 1.9, 270)
at(p100[3], '3V3_PLL', PX + 4.6, PY + 1.0, 90); at(p10[3], '3V3_PLL', PX + 5.7, PY + 1.0, 90)
at(p10u, '3V3_PLL', PX + 5.6, PY - 6.6, 0)
at(one('C', 'PLL_CP', 'GND'), 'PLL_CP', PX - 1.9, PY - 7.2, 180); at(one('R', 'PLL_CP', 'PLL_VTUNE'), 'PLL_CP', PX - 0.6, PY - 7.2, 0)
at(one('R', 'PLL_CP', 'PLL_LF2'), 'PLL_CP', PX - 1.9, PY - 8.3, 270); at(one('C', 'PLL_LF2', 'GND'), 'PLL_LF2', PX - 1.9, PY - 10.5, 270)
at(one('R', 'PLL_CE', '3V3_DIG'), 'PLL_CE', PX + 3.0, PY - 6.5, 0)
vc = sel('C', '3V3_VCO', 'GND'); v100 = [q for q in vc if P[q]['value'] == '100n']; v10 = [q for q in vc if P[q]['value'] == '10p']; v10u = [q for q in vc if P[q]['value'] == '10u'][0]
at(v100[0], '3V3_VCO', PX - 2.0, PY + 5.2, 90); at(one('C', 'PLL_TEMP', 'GND'), 'PLL_TEMP', PX - 0.9, PY + 5.2, 90)
at(one('C', 'PLL_VTUNE', 'GND'), 'PLL_VTUNE', PX + 0.2, PY + 5.2, 90); at(one('R', 'PLL_RSET', 'GND'), 'PLL_RSET', PX + 1.3, PY + 5.2, 90)
at(one('C', 'PLL_VCOM', 'GND'), 'PLL_VCOM', PX + 2.4, PY + 5.2, 90); at(one('C', 'PLL_VREF', 'GND'), 'PLL_VREF', PX + 3.5, PY + 5.2, 90)
at(v10[0], '3V3_VCO', PX - 0.9, PY + 7.6, 90); at(v100[1], '3V3_VCO', PX + 0.2, PY + 7.6, 90); at(v10[1], '3V3_VCO', PX + 1.3, PY + 7.6, 90); at(v10u, '3V3_VCO', PX + 2.8, PY + 8.0, 0)
for n_, pn_ in (('3V3_VCO', '17'), ('PLL_TEMP', '19'), ('PLL_VTUNE', '20'), ('PLL_RSET', '22'), ('PLL_VCOM', '23'), ('PLL_VREF', '24')):
    rr_ = [q for q in sel('C', n_, 'GND') + sel('R', n_, 'GND') if abs(pos(q, pinof(q, n_))[1] - (PY + 5.2)) < 0.01][0]; q = pos(pll, pn_); t_ = pos(rr_, pinof(rr_, n_))
    path(n_, [q, (q[0], PY + 3.3), (t_[0], PY + 3.3 + abs(t_[0] - q[0])) if abs(t_[0] - q[0]) < 1.5 else (t_[0], PY + 4.9), t_], 0.2)
q = pos(pll, '28'); t_ = pos(p100[3], pinof(p100[3], '3V3_PLL')); path('3V3_PLL', [q, (PX + 3.9, q[1]), t_], 0.2)
q = pos(pll, '32'); t_ = pos(p100[2], pinof(p100[2], '3V3_PLL')); path('3V3_PLL', [q, (PX + 3.9, q[1]), t_], 0.2)
q = pos(pll, '6'); path('3V3_PLL', [q, (q[0], PY - 3.5), (q[0] + 0.15, PY - 4.15)], 0.2)
q = pos(pll, '10'); t_ = pos(p100[1], pinof(p100[1], '3V3_PLL')); path('3V3_PLL', [q, t_], 0.2)
put(one('TP', 'PLL_VTUNE'), PX + 5.6, PY + 10.6)
tcxo = one('Y', 'TCXO_OUT'); put(tcxo, PX + 8.8, PY - 0.4, 90)
at(one('C', 'TCXO_OUT', 'PLL_REFIN'), 'PLL_REFIN', PX + 4.5, PY - 0.45, 0)
q = pos(pll, '29'); cr_ = one('C', 'TCXO_OUT', 'PLL_REFIN'); path('PLL_REFIN', [q, (PX + 3.6, q[1]), pos(cr_, pinof(cr_, 'PLL_REFIN'))], 0.2)
at(one('FB', '3V3_PLL', '3V3_TCXO'), '3V3_TCXO', PX + 8.4, PY - 4.6, 180); at(one('C', '3V3_TCXO', 'GND'), '3V3_TCXO', PX + 9.7, PY - 4.6, 90)

# ------------------------------------------------------------------ everything else: fixed spots, then the auto-placer
# standoffs: M2.5 holes on the shared spots, written down for the control board
import json
spots = []
for h, (hx, hy) in zip(sel('H', 'GND'), STANDOFFS):
    spots.append(ap.block(h, hx, hy, 0, tries=1) or (hx, hy))   # exact: the control board uses the same spots
XL, XT = 1.9, 82.0                  # XIAO left socket pin 1, bottom left beside the footswitch slot; USB end up, to a
                                    # panel-mount USB-C extension on the top wall (15 mm standoffs clear the module)
xl = one('J', 'I2S_BCLK', 'CTL_INT'); pin_at(xl, '1', XL, XT, 0); xr = one('J', 'XIAO_5V'); pin_at(xr, '1', XL + 15.24, XT, 0)
assert pos(xl, '7')[1] > pos(xl, '1')[1]
ap.block(one('D', 'XIAO_5V'), 12.0, 79.2, 0)
# LNA rail
lx = pos(lna, '2')[0]
ap.near(one('FB', '+5V', '5V_LNA'), '5V_LNA', (lx, RFY + 4.0), 90, 1.2)
ap.near([q for q in sel('C', '5V_LNA', 'GND') if P[q]['value'] == '10u'][0], '5V_LNA', (lx + 2.4, RFY + 4.0), 90, 1.2)
# band switches' support (as Rev A, below the switches)
s1 = pos(sw1, '2'); s2 = pos(sw2, '2'); sc = [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('3')]
ap.near(sc[0], '3V3_DIG', (s1[0], RFY + 2.0), 90, 1.4); ap.near(sc[1], '3V3_DIG', (s2[0], RFY + 2.0), 90, 1.4)
ap.block(one('R', 'BAND_SEL', 'GND'), s1[0] + 3.0, RFY + 6.0, 90)
ap.block(one('Q', 'BAND_SEL', 'BAND_SEL_N'), s2[0] + 3.0, RFY + 5.8, 0)
ap.near(one('R', 'BAND_SEL_N', '3V3_DIG'), 'BAND_SEL_N', (s2[0] + 5.5, RFY + 5.8), 0, 0.8)
# bias tee, left of the XIAO
q1 = one('Q', 'BT_GATE', 'BT_SW'); ap.block(q1, 28.0, 20.5, 90)
q2 = one('Q', 'BIAS_EN', 'BT_GATE'); ap.block(q2, 28.0, 15.5, 90)
for ref, net, xy, ang in ((one('R', 'BT_FUSED', 'BT_GATE'), 'BT_GATE', (26.0, 18.0), 180), (one('R', 'BIAS_EN', 'GND'), 'BIAS_EN', (30.0, 15.5), 0),
                          (one('F', 'BT_FUSED'), 'BT_FUSED', (25.0, 21.5), 180), (one('R', 'BT_SW', 'BT_FILT'), 'BT_SW', (30.0, 21.5), 0),
                          (one('R', 'BT_SW', 'LED_BT_A'), 'BT_SW', (28.0, 23.6), 180), (one('D', 'LED_BT_A', 'GND'), 'LED_BT_A', (25.5, 24.5), 180)):
    ap.near(ref, net, xy, ang, 0.6)
for c_ in sel('C', 'BT_FILT', 'GND'):
    ap.near(c_, 'BT_FILT', (pos(l_bt, pinof(l_bt, 'BT_FILT'))[0] - 1.0, RFY - 4.5), 270, 0.6)
# power input, top left
jdc = one('J', 'VDC_RAW'); ap.block(jdc, 4.6, 12.0, 90)
qv = one('Q', 'VDC_G'); ap.block(qv, 9.6, 8.5, 0)
for ref, net, xy, ang in ((one('R', 'VDC_G', 'GND'), 'VDC_G', (9.6, 11.6), 90),):
    ap.near(ref, net, xy, ang, 0.4)
ap.block(one('D', 'VDC', 'VDC_G'), 13.6, 11.6, 0)
ap.block(one('D', 'VBUCK_IN_F', 'GND'), 6.0, 18.0, 0)
ap.block(one('F', 'VDC', 'VBUCK_IN_F'), 12.6, 17.2, 0)
ap.block(one('FB', 'VBUCK_IN_F', 'VBUCK_IN'), 15.8, 15.6, 90)
buck = one('U', 'BUCK_SW'); put(buck, 20.5, 9.0, 0)
# tight to the pins: input cap on VIN/GND (pins 3, 4), RT on pin 1, bootstrap across BST/SW (6, 5),
# soft start on pin 7, feedback divider above pin 8, inductor and output caps to the right.
at(one('C', 'VBUCK_IN', 'GND', value='100n'), 'VBUCK_IN', pos(buck, '3')[0] - 1.5, pos(buck, '3')[1], 180)
at(one('R', 'BUCK_RT', 'GND'), 'BUCK_RT', pos(buck, '1')[0] - 1.25, pos(buck, '1')[1] - 0.25, 180)
at(one('C', 'BUCK_BST', 'BUCK_SW'), 'BUCK_BST', pos(buck, '6')[0] + 1.3, pos(buck, '6')[1] + 0.35, 90)
at(one('C', 'BUCK_SS', 'GND'), 'BUCK_SS', pos(buck, '7')[0] + 2.4, pos(buck, '7')[1], 0)
at(one('R', 'BUCK_FB', 'GND'), 'BUCK_FB', pos(buck, '8')[0], pos(buck, '8')[1] - 1.3, 270)
at(one('R', 'VBUCK_OUT', 'BUCK_FB'), 'BUCK_FB', pos(buck, '8')[0] - 1.1, pos(buck, '8')[1] - 1.3, 270)
ap.block(one('L', 'BUCK_SW', 'VBUCK_OUT'), 23.6, 13.4, 90, tries=80, step=0.4)
for c_, (xx_, yy) in zip([q for q in sel('C', 'VBUCK_IN', 'GND') if P[q]['value'] == '10u'], ((16.2, 7.6), (12.9, 4.6))):
    ap.block(c_, xx_, yy, 0, tries=120, step=0.4)
for c_ in sel('C', 'VBUCK_OUT', 'GND'):
    ap.block(c_, 26.6, 11.0, 90, tries=120, step=0.4)
fb5 = one('FB', 'VBUCK_OUT', '+5V'); ap.block(fb5, 21.0, 20.4, 0)
for c_ in [q for q in sel('C', '+5V', 'GND') if P[q]['block'].startswith('1') and P[q]['value'] in ('22u', '100n')]:
    ap.near(c_, '+5V', (13.0, 22.6), 0, 0.6, along=180)
ap.block(one('R', '+5V', 'LED_PWR_A'), 4.0, 22.8, 90, tries=120, step=0.4); ap.block(one('D', 'LED_PWR_A', 'GND'), 2.4, 22.8, 90, tries=120, step=0.4)
# LDOs near their loads
def ldo(rail, x, y):
    u = one('U', '+5V', rail); ap.block(u, x, y, 0)
    cs = [q for q in sel('C', rail, 'GND') if P[q]['block'].startswith('1')]
    ci = [q for q in sel('C', '+5V', 'GND') if P[q]['block'].startswith('1') and P[q]['value'] == '1u']
    ap.decouple(u, cs[:1], 0.8)
    for c_ in ci:
        if c_ not in k.placed:
            ap.near(c_, '+5V', pos(u, '1'), 180, 0.8); break
ldo('3V3_VCO', PX - 5.5, PY + 15.6); ldo('3V3_PLL', PX + 4.5, PY + 17.2)
ldo('3V3_ADC', 37.0, 44.5); ldo('3V3_DIG', 14.5, 74.5)
b2b = one('J', 'CTL_INT', '3V3_DIG'); ap.block(b2b, 42.4, 59.3, 0, tries=150, step=0.35)   # under the control board
ap.block(one('R', 'PLL_PDBRF', '3V3_DIG'), 38.5, 50.5, 90, tries=150, step=0.35)
# audio, bottom right; its output JST beside it
ap.block(one('J', 'AUDIO_OUT'), 41.0, 98.0, 0, tries=150, step=0.4)
ldo('3V3_AUD', 46.0, 79.5)
dac = one('U', 'DAC_OUTL'); ap.block(dac, 44.5, 89.0, 180, tries=120, step=0.4)
ap.decouple(dac, [q for q in sel('C', '3V3_AUD', 'GND') if P[q]['block'].startswith('7')], 0.9)
for ref, pn in ((one('C', 'DAC_CAPP', 'DAC_CAPM'), '3'), (one('C', 'DAC_VNEG', 'GND'), '5'), (one('C', 'DAC_LDOO', 'GND'), '18')):
    px_, py_ = pos(dac, pn); cx_ = k.centre(dac)[0]
    ap.block(ref, px_ + (-2.2 if px_ < cx_ else 2.2), py_, 90, tries=150, step=0.35)
for ref, xy in ((one('R', 'DAC_XSMT', 'GND'), (39.6, 84.0)), (one('R', 'XD7', 'DAC_DIN'), (39.6, 86.0)),
                (one('R', 'DAC_OUTL', 'AUDIO_OUT'), (46.5, 96.5)), (one('C', 'AUDIO_OUT', 'GND'), (48.5, 96.5))):
    ap.block(ref, xy[0], xy[1], 0, tries=150, step=0.35)
# controller support: expander, pull-ups, the control-board header
exp = one('U', 'EXP_INT'); ap.block(exp, 15.0, 65.5, 0, tries=120, step=0.4)
ap.decouple(exp, [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('8')], 0.9)
for ref, net, xy in ((one('R', 'SDA', '3V3_DIG'), 'SDA', (13.0, 69.0)), (one('R', 'SCL', '3V3_DIG'), 'SCL', (15.0, 69.0)),
                     (one('R', 'CTL_INT', '3V3_DIG'), 'CTL_INT', (17.0, 66.5))):
    ap.block(ref, xy[0], xy[1], 90, tries=100, step=0.4)
ex13 = pos(exp, '13'); ap.near(one('R', 'EXP_INT', '3V3_DIG'), 'EXP_INT', ex13, 0, 1.0)
ap.block(one('TP', 'EXP_INT'), 18.0, 70.0, 0, tries=120, step=0.4)
ap.block(one('TP', 'GND'), 44.0, 70.0, 0, tries=150, step=0.4)
# trim DAC above its divider network
tdac = one('U', 'DAC_A'); ap.block(tdac, 12.6, 44.2, 0, tries=80, step=0.4)
ap.decouple(tdac, [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('4')], 0.9)
# 5 V and other leftovers
for c_ in [q for q in sel('C', '+5V', 'GND') if q not in k.placed]:
    ap.near(c_, '+5V', (14.0, 23.0), 90, 0.6, along=0)
json.dump({'b2b_pins': {pn: list(map(lambda v: round(v, 3), pos(b2b, pn))) for pn in '12345678'}, 'standoffs': [list(map(lambda v: round(v, 2), s)) for s in spots], 'b2b_pin1': list(map(lambda v: round(v, 3), pos(b2b, '1'))),
           'b2b_rot': k.fps[b2b].GetOrientationDegrees()}, open(f'{boardcfg.KICAD}/stack.json', 'w'), indent=1)
print('standoffs', spots, 'b2b pin 1', pos(b2b, '1'))
if ap.failed:
    print('PLACEMENT FAILED:', len(ap.failed)); [print('  ', f) for f in ap.failed]

power = ['+5V', 'VDC', 'VDC_RAW', 'VBUCK_IN', 'VBUCK_IN_F', 'VBUCK_OUT', 'BUCK_SW', '5V_DEMOD', '5V_LNA', 'BT_FUSED', 'BT_SW', 'BT_FILT', 'XIAO_5V'] + \
    ['3V3_VCO', '3V3_PLL', '3V3_ADC', '3V3_DIG', '3V3_AUD', '3V3_TCXO', '3V3_MCLK']
k.save(power, partial=True)
