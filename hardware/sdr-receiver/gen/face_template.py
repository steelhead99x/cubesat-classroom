"""Drilling template for the CSC-R1P pedal: Hammond 1590N1 ("125B") face and walls, 1:1 SVG.

Hole positions come from the boards' own placement (../pedal/kicad/face.json, written by
gen_pcb_r1p_ctl.py), so the template always matches the boards. Panel parts that are wired,
not soldered (footswitch, toggle, jacks), are placed here and listed in the pedal README.

Print at 100 % (no "fit to page"), check the 50 mm scale bar with a ruler, tape it to the box,
centre-punch, then drill small first. Check every hole against your own enclosure and parts
before drilling: box makers and part makers differ.

Run: python3 face_template.py   (writes ../pedal/face-template.svg)
"""
import json
import os

KICAD = '../pedal/kicad'
OUT = '../pedal/face-template.svg'

# Hammond 1590N1 outside 121.2 x 65.5 x 39.8 mm (inside 115.91 long, about 62 wide).
BOX_L, BOX_W, BOX_H = 121.2, 65.5, 39.8
INSIDE_L, INSIDE_W = 115.91, 62.0
BOARD_L, BOARD_W = 104.0, 54.0
# board coordinates (x across, y along from the top end) -> outside face coordinates
DX = (BOX_W - BOARD_W) / 2          # boards are centred in the box
DY = (BOX_L - BOARD_L) / 2

face = json.load(open(os.path.join(KICAD, 'face.json')))

# (label, x_board, y_board, hole diameter mm, note)
holes = [(f'{n}', x, y, 7.5, 'M7 bushing (Alpha 9 mm pot / EC11 encoder)') for n, (x, y) in face['shafts'].items()]
holes += [('ENGAGE LED', face['led'][0], face['led'][1], 5.0, '3 mm LED in a bezel: check the bezel')]
holes += [('BAND toggle', 40.0, 79.0, 6.5, 'mini toggle: check the bushing'),
          ('ENGAGE footswitch', 27.0, 92.0, 12.2, 'soft-touch momentary footswitch: check the bushing')]
# OLED window: the module sits to the right of its header (pin 1 at oled_pin1, pins going down)
ox, oy = face['oled_pin1']
oled_window = (ox + 6.0, oy - 2.8, 25.0, 9.0)   # x, y, w, h (board coords): check against your module's glass

# Wall holes: (wall, label, position along the wall from its left end as seen from outside, height from the face, diameter)
walls = [
    ('top', 'DC 9 V (2.1 mm, centre negative)', 20.0, 12.0, 8.0),
    ('top', 'USB-C panel extension', 36.0, 12.0, None),
    ('top', 'Wi-Fi antenna RP-SMA (XIAO U.FL pigtail)', 50.0, 12.0, 6.5),
    ('left', 'OUT 1/4 inch mono jack', 40.0, 14.0, 9.5),
    ('right', 'ANT SMA bulkhead (U.FL pigtail)', 40.0, 14.0, 6.5),
]


def svg():
    m = 12.0                                   # margin
    W = m + BOX_W + m + BOX_L + m              # face, then the walls drawn beside it
    H = m + BOX_L + m + 40
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}mm" height="{H}mm" viewBox="0 0 {W} {H}" font-family="Helvetica, Arial, sans-serif">',
         '<rect width="100%" height="100%" fill="white"/>']
    def t(x, y, s, size=2.6, anchor='start', weight='normal'):
        o.append(f'<text x="{x:.2f}" y="{y:.2f}" font-size="{size}" text-anchor="{anchor}" font-weight="{weight}">{s}</text>')
    def cross(x, y, r):
        o.append(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="{r / 2:.2f}" fill="none" stroke="black" stroke-width="0.25"/>')
        o.append(f'<path d="M{x - r / 2 - 1.5:.2f} {y:.2f}H{x + r / 2 + 1.5:.2f}M{x:.2f} {y - r / 2 - 1.5:.2f}V{y + r / 2 + 1.5:.2f}" stroke="black" stroke-width="0.15"/>')
    t(m, 7, 'CSC-R1P satellite pedal: drilling template, Hammond 1590N1 (125B). Print at 100 %. Check every hole against your box and parts.', 3.0, weight='bold')
    # face
    fx, fy = m, m
    o.append(f'<rect x="{fx}" y="{fy}" width="{BOX_W}" height="{BOX_L}" rx="3" fill="none" stroke="black" stroke-width="0.4"/>')
    t(fx + BOX_W / 2, fy + BOX_L + 5, 'FACE (seen from outside; top end at the top)', 2.8, 'middle', 'bold')
    for label, x, y, d, note in holes:
        X, Y = fx + DX + x, fy + DY + y
        cross(X, Y, d)
        t(X + (d / 2 + 2 if label == 'BAND toggle' else 0), Y - d / 2 - 2.2, f'{label}  {d} mm', 2.2, 'start' if label == 'BAND toggle' else 'middle')
        t(X, Y + d / 2 + 3.4, f'{X - fx:.1f}, {Y - fy:.1f}', 1.8, 'middle')
    wx, wy, ww, wh = oled_window
    o.append(f'<rect x="{fx + DX + wx:.2f}" y="{fy + DY + wy:.2f}" width="{ww}" height="{wh}" fill="none" stroke="black" stroke-width="0.25" stroke-dasharray="1 0.6"/>')
    t(fx + DX + wx + ww / 2, fy + DY + wy - 1.2, 'OLED window (check your module)', 2.0, 'middle')
    # walls
    def wall(x0, y0, length, name, items):
        o.append(f'<rect x="{x0}" y="{y0}" width="{length}" height="{BOX_H}" fill="none" stroke="black" stroke-width="0.4"/>')
        t(x0 + length / 2, y0 - 2.5, name, 2.6, 'middle', 'bold')
        for i, (_, label, along, h, d) in enumerate(items):
            X, Y = x0 + along, y0 + h
            if d:
                cross(X, Y, d)
            else:
                o.append(f'<rect x="{X - 6:.2f}" y="{Y - 3:.2f}" width="12" height="6" rx="2" fill="none" stroke="black" stroke-width="0.25"/>')
            dy = (d or 6) / 2 + 3.2 + 4.2 * (i % 2)     # stagger neighbouring labels
            t(X, Y + dy, label, 1.9, 'middle')
    x1 = m + BOX_W + m
    wall(x1, m + 6, BOX_W, 'TOP END wall (face edge at the top)', [w for w in walls if w[0] == 'top'])
    wall(x1, m + 6 + BOX_H + 14, BOX_L, 'LEFT wall (face edge at the top)', [w for w in walls if w[0] == 'left'])
    wall(x1, m + 6 + 2 * (BOX_H + 14), BOX_L, 'RIGHT wall (face edge at the top)', [w for w in walls if w[0] == 'right'])
    # scale bar
    sy = H - 12
    o.append(f'<path d="M{m} {sy}H{m + 50}M{m} {sy - 2}V{sy + 2}M{m + 50} {sy - 2}V{sy + 2}" stroke="black" stroke-width="0.4"/>')
    t(m + 25, sy - 3, 'this bar must measure 50 mm', 2.4, 'middle')
    t(m + 60, sy + 1, 'Coordinates are mm from the face\'s top-left outside corner. Desk design: nothing here has been test-fitted.', 2.2)
    o.append('</svg>')
    return '\n'.join(o) + '\n'


if __name__ == '__main__':
    open(OUT, 'w').write(svg())
    print('wrote', OUT)
    for label, x, y, d, note in holes:
        print(f'  {label:18s} x {x + DX:6.1f}  y {y + DY:6.1f}  {d} mm  {note}')
