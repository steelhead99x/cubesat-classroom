"""Stage 1 for the CSC-R1P pedal control board: outline, placement. Run with CSC_BOARD=csc-r1p-ctl.

The board hangs under the face by the pot and encoder nuts. Its x/y match the main board's, so
the standoff holes and the stacking connector come straight from the main board's placement
(../pedal/kicad/stack.json, written by gen_pcb_r1p.py). Shaft, LED and display positions are
written to face.json for the face drilling template.
"""
import json
import math

import pcbnew

import boardcfg
from pcbkit import Board, BOX_POSTS, POST_CLEAR, AutoPlace, notch

assert boardcfg.PROJ == 'csc-r1p-ctl', 'run with CSC_BOARD=csc-r1p-ctl'
d = boardcfg.design()
k = Board(d)
P = d.parts
one, sel, pinof, pos, put, pin_at, at = k.one, k.sel, k.pinof, k.pos, k.put, k.pin_at, k.at
W, H = 54.0, 72.0
stack = json.load(open(f'{boardcfg.KICAD}/stack.json'))

(px0, py0), (px1, _), _, _ = BOX_POSTS
r = POST_CLEAR
dx = math.sqrt(r * r - py0 ** 2); dy = math.sqrt(r * r - px0 ** 2)
poly = k.outline([
    ('L', (px0 + dx, 0)), ('L', (px1 - dx, 0)),
    notch((px1, py0), r, (px1 - dx, 0), (W, py0 + dy)),
    ('L', (W, H)), ('L', (0, H)), ('L', (0, py0 + dy)),
    notch((px0, py0), r, (0, py0 + dy), (px0 + dx, 0)),
])
SHAFT_Y = 11.4
SHAFT = {'GAIN': 10.0, 'TUNE': 27.0, 'LEVEL': 44.0}
OLED_AREA = (7.0, 25.0, 47.0, 39.0)          # the module's footprint above the board: keep tall parts out
ap = AutoPlace(k, poly, keepouts=[OLED_AREA])


def shaft_at(ref, sx, sy):
    """Pots and the encoder: shaft at (sx, sy), pins below it (rotation 90: shaft is 7.5 mm above pin 1)."""
    put(ref, sx - 2.5, sy + 7.5, 90)


for name in ('GAIN', 'LEVEL'):
    shaft_at(one('RV', value=f'{name} 10k B'), SHAFT[name], SHAFT_Y)
shaft_at(one('SW', 'ENC_A'), SHAFT['TUNE'], SHAFT_Y)
# standoffs and the stacking socket (underside) exactly over the main board's
for h, (hx, hy) in zip(sel('H', 'GND'), stack['standoffs']):
    put(h, hx, hy, 0)
b2b = one('J', 'CTL_INT', '3V3_DIG')
# On the underside the socket is seen mirrored; KiCad keeps a flipped part's rotation separately,
# so try both and keep the one whose eight pins land on the main board's eight pins.
for rot_ in (stack['b2b_rot'], stack['b2b_rot'] + 180):
    put(b2b, 0, 0, rot_, back=True)
    p1 = pos(b2b, '1'); put(b2b, stack['b2b_pin1'][0] - p1[0], stack['b2b_pin1'][1] - p1[1], rot_, back=True)
    if all(abs(pos(b2b, pn)[0] - xy[0]) < 0.01 and abs(pos(b2b, pn)[1] - xy[1]) < 0.01 for pn, xy in stack['b2b_pins'].items()):
        break
oled = one('J', value='OLED'); pin_at(oled, '1', 8.6, 28.2, 0)
LED_XY = (27.0, 47.0)
led = one('D', 'LED_ENG_K'); pin_at(led, '1', LED_XY[0] - 1.27, LED_XY[1], 0)
ap.block(one('J', 'FOOT_SW'), 14.0, 63.0, 0, tries=100, step=0.4)
ap.block(one('J', 'BAND_SW'), 25.0, 63.0, 0, tries=100, step=0.4)
exp = one('U', 'CTL_INT', 'ENC_A'); ap.block(exp, 36.5, 54.0, 90, tries=100, step=0.4)
adc = one('U', 'POT_GAIN'); ap.block(adc, 14.5, 46.0, 0, tries=100, step=0.4)
ap.decouple(exp, [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('2')], 0.9)
ap.decouple(adc, [q for q in sel('C', '3V3_DIG', 'GND') if P[q]['block'].startswith('3')], 0.9)
# pull-ups in a column left of the expander, debounce caps beside them (room for the router)
for i, net in enumerate(('ENC_A', 'ENC_B', 'ENC_SW', 'FOOT_SW', 'BAND_SW', 'SPARE6', 'SPARE7')):
    ap.block(one('R', net, '3V3_DIG'), 29.0, 47.0 + 1.7 * i, 0, tries=60, step=0.3)
for i, net in enumerate(('ENC_A', 'ENC_B', 'ENC_SW', 'FOOT_SW')):
    ap.block(one('C', net, 'GND'), 25.6, 50.0 + 1.7 * i, 0, tries=60, step=0.3)
for net in ('POT_GAIN', 'POT_LEVEL'):
    pn = pinof(adc, net); px_, py_ = pos(adc, pn)
    rr = one('R', net + '_W', net); ap.near(rr, net, (px_, py_), 180, 1.6)
    ap.block(one('C', net, 'GND'), px_ - 2.4, py_ + (1.3 if net == 'POT_LEVEL' else -1.3), 0, tries=150, step=0.35)
ap.near(one('R', '3V3_DIG', 'LED_ENG_A'), 'LED_ENG_A', pos(led, '2'), 0, 1.5)
ap.block(one('C', '3V3_DIG', 'GND', value='10u'), 44.0, 42.0, 0)
ap.block(one('C', '+5V', 'GND'), 44.0, 39.5, 0)
for t, xx, yy, sz in d.SILK:
    pass
if ap.failed:
    print('PLACEMENT FAILED:', len(ap.failed)); [print('  ', f) for f in ap.failed]
# check: every stacking pin is over the main board's header pin
main_pins = stack['b2b_pins']
for pn, xy in main_pins.items():
    q = pos(b2b, pn); assert abs(q[0] - xy[0]) < 0.01 and abs(q[1] - xy[1]) < 0.01, ('stacking pin', pn, q, xy)
print('stacking connector: all 8 pins line up with the main board')
face = {'shafts': {n: [x, SHAFT_Y] for n, x in SHAFT.items()}, 'led': list(LED_XY), 'oled_pin1': list(pos(oled, '1')),
        'b2b_pins': {pn: list(map(lambda v: round(v, 3), pos(b2b, pn))) for pn in '12345678'}}
json.dump(face, open(f'{boardcfg.KICAD}/face.json', 'w'), indent=1)
k.save(['3V3_DIG', '+5V'], partial=bool(ap.failed))
