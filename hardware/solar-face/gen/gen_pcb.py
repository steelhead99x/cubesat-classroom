"""Board for one solar face variant: outline, placement and routing. Writes ../kicad/solar-face-<v>/solar-face-<v>.kicad_pcb

Front: cells only. Back: connector, blocking diode and its bypass jumper, thermistor, test pads, all
surface mount so nothing pokes through under the cells. Both variants share the back.
Coordinates are mm from the board's top-left corner, front view.
"""
import json
import sys
import pcbnew
import design

VAR = sys.argv[1]
design.load(VAR)
parts = design.parts
PROJ = f'solar-face-{VAR}'
OUT = f'../kicad/{PROJ}'
FPDIR = '/usr/share/kicad/footprints'
W, H = design.W, design.H
OX, OY = 100.0, 50.0
mm = pcbnew.FromMM


def V(x, y):
    return pcbnew.VECTOR2I(mm(OX + x), mm(OY + y))


board = pcbnew.BOARD()
board.SetCopperLayerCount(2)
nets = {}
for i, name in enumerate(sorted(design.nets())):
    ni = pcbnew.NETINFO_ITEM(board, name, i + 1)
    board.Add(ni)
    nets[name] = ni

fps = {}
for r, p in parts.items():
    lib, name = p['fp'].split(':')
    d = '../kicad' if lib == design.FP_LIB else FPDIR
    f = pcbnew.FootprintLoad(f'{d}/{lib}.pretty', name)
    assert f is not None, p['fp']
    f.SetReference(r)
    f.SetValue(p['value'])
    f.SetFPID(pcbnew.LIB_ID(lib, name))
    for pd in f.Pads():
        n = p['pins'].get(pd.GetNumber())
        if n and n != design.NC:
            pd.SetNet(nets[n])
    f.Reference().SetTextSize(pcbnew.VECTOR2I(mm(0.8), mm(0.8)))
    f.Reference().SetTextThickness(mm(0.12))
    board.Add(f)
    fps[r] = f


def put(ref, x, y, rot=0, back=False):
    f = fps[ref]
    if back and not f.IsFlipped():
        f.Flip(f.GetPosition(), False)
    f.SetOrientationDegrees(rot)
    f.SetPosition(V(x, y))
    return f


def pad_xy(ref, pin):
    ps = [pd.GetPosition() for pd in fps[ref].Pads() if pd.GetNumber() == str(pin)]
    return (sum(pcbnew.ToMM(q.x) for q in ps) / len(ps) - OX, sum(pcbnew.ToMM(q.y) for q in ps) / len(ps) - OY)


tracks = []


def seg(net, a, b, w, layer):
    t = pcbnew.PCB_TRACK(board)
    t.SetStart(V(*a)); t.SetEnd(V(*b)); t.SetWidth(mm(w)); t.SetLayer(layer); t.SetNet(nets[net])
    board.Add(t); tracks.append(t)


def path(net, pts, w, layer):
    for a, b in zip(pts, pts[1:]):
        seg(net, a, b, w, layer)


def via(net, x, y, d=0.8, drill=0.4):
    v = pcbnew.PCB_VIA(board)
    v.SetPosition(V(x, y)); v.SetWidth(mm(d)); v.SetDrill(mm(drill)); v.SetNet(nets[net])
    board.Add(v); tracks.append(v)
    return (x, y)


F, B = pcbnew.F_Cu, pcbnew.B_Cu
PWR = 1.2       # mm, cell-current traces (0.5 A worst case on the tj face)

# ------------------------------------------------------------------ front: cells
# Power comes down the right of the back, ground down the left; the thermistor line runs between.
raw_vias, gnd_vias = [], []
if VAR == 'tj':
    import cells
    bw, bh = cells.CELLS['CIC_3G30A_8x4']['body']
    top = 2.0
    c1y = top + bh / 2
    c2y = top + bh + 4.5 + bh / 2           # 3 mm land + 1.5 mm gap between the cells
    put('SC1', W / 2, c1y)
    put('SC2', W / 2, c2y)
    # SC1 '-' land feeds SC2 '+' rear pad (same net), three short bridges
    s1 = pad_xy('SC1', 2)[1]
    p2 = pad_xy('SC2', 1)[1] - (bh - 4.0) / 2
    for dx in (-25, 0, 25):
        seg('S1_1', (W / 2 + dx, s1), (W / 2 + dx, p2 + 1.0), 2.0, F)
    # PV_RAW: SC1 rear pad up into the top margin, vias to the back
    p1top = c1y - (bh - 4.0) / 2
    for dx in (-25, 0, 25):
        seg('PV_RAW', (W / 2 + dx, p1top + 1.0), (W / 2 + dx, 1.2), PWR, F)
    raw_vias = [via('PV_RAW', W / 2 + dx, 1.2) for dx in (-25, 0, 25)]
    # GND: SC2 '-' land down into the bottom margin, vias to the back
    s2 = pad_xy('SC2', 2)[1]
    for dx in (-25, -10):
        seg('GND', (W / 2 + dx, s2), (W / 2 + dx, 93.0), PWR, F)
    gnd_vias = [via('GND', W / 2 + dx, 93.0) for dx in (-25, -10)]
    cell_bottom = s2 + 1.5
else:
    # each row is one string, '+' at the right: SC(3i+1) on the right, SC(3i+3) on the left
    pitch_x, pitch_y = 25.5, 10.5
    xs = [W / 2 + pitch_x, W / 2, W / 2 - pitch_x]
    ys = [6.0 + i * pitch_y for i in range(design.variant['strings'])]
    n = 0
    for y in ys:
        for x in xs:
            n += 1
            put(f'SC{n}', x, y, 180)
    bus_raw, bus_gnd = W - 2.0, 2.0
    for i, y in enumerate(ys):
        a, b, c = (f'SC{3 * i + k}' for k in (1, 2, 3))
        seg('PV_RAW', (bus_raw, y), pad_xy(a, 1), 0.8, F)
        seg(f'S{i + 1}_1', pad_xy(a, 2), pad_xy(b, 1), 0.8, F)
        seg(f'S{i + 1}_2', pad_xy(b, 2), pad_xy(c, 1), 0.8, F)
        seg('GND', pad_xy(c, 2), (bus_gnd, y), 0.8, F)
    yb = 88.0
    seg('PV_RAW', (bus_raw, ys[0]), (bus_raw, yb), PWR, F)
    seg('GND', (bus_gnd, ys[0]), (bus_gnd, yb), PWR, F)
    raw_vias = [via('PV_RAW', bus_raw, yb)]
    gnd_vias = [via('GND', bus_gnd, yb)]
    cell_bottom = ys[-1] + 4.0

# ------------------------------------------------------------------ back: output parts (same on both)
put('J1', W / 2, 92.0, 0, back=True)
if pad_xy('J1', 1)[0] < pad_xy('J1', 3)[0]:      # pin 1 (PV+) on the right, nearest the diode
    put('J1', W / 2, 92.0, 180, back=True)
put('D1', 64.0, 92.0, 0, back=True)
if pad_xy('D1', 1)[0] > pad_xy('D1', 2)[0]:      # cathode toward the connector (left)
    put('D1', 64.0, 92.0, 180, back=True)
put('JP1', 64.0, 87.0, 0, back=True)
if pad_xy('JP1', 2)[0] > pad_xy('JP1', 1)[0]:    # pin 2 (PV_OUT) above the cathode
    put('JP1', 64.0, 87.0, 180, back=True)
put('TH1', W / 2, 49.0, 90, back=True)
if pad_xy('TH1', 1)[1] < pad_xy('TH1', 2)[1]:    # NTC pin below the GND pin
    put('TH1', W / 2, 49.0, 270, back=True)
put('TP1', 56.0, 85.0, back=True)
put('TP3', 72.0, 92.0, back=True)
put('TP2', 6.0, 92.0, back=True)

k = pad_xy('D1', 1); a = pad_xy('D1', 2)
jp_out = pad_xy('JP1', 2); jp_raw = pad_xy('JP1', 1)
j1 = {n: pad_xy('J1', n) for n in (1, 2, 3)}
pad_top = j1[1][1] - 2.75
pad_bot = j1[1][1] + 2.75   # signal pads are 1.0 x 5.5 mm

# PV_RAW: right lane down to D1 anode; JP1 pin 1 straight above the anode
RL = W - 6.5
for v in raw_vias:
    path('PV_RAW', [v, (RL, v[1])], PWR, B)
top_raw = min(v[1] for v in raw_vias)
path('PV_RAW', [(RL, top_raw), (RL, a[1]), a], PWR, B)
path('PV_RAW', [jp_raw, (a[0], jp_raw[1])], 0.6, B)
path('PV_RAW', [(a[0], jp_raw[1]), a], PWR, B)

# PV_OUT: cathode up past JP1 pin 2, then left along y 85 and down into J1 pin 1 from above
path('PV_OUT', [jp_out, (k[0], jp_out[1])], 0.6, B)
path('PV_OUT', [k, (k[0], 85.0), (j1[1][0], 85.0), (j1[1][0], pad_top + 1.0)], PWR, B)

# GND: left trunk at x 6 from the thermistor down to y 96.8, along to J1 pin 2, up into it from below
GL, GY = 6.0, 96.8
th_g = pad_xy('TH1', 2); th_n = pad_xy('TH1', 1)
path('GND', [th_g, (GL, th_g[1]), (GL, GY), (j1[2][0], GY), (j1[2][0], pad_bot - 1.0)], PWR, B)
for v in gnd_vias:
    path('GND', [v, (GL, v[1])] if v[0] < GL else [v, (v[0], GY)], PWR, B)

# NTC: thermistor straight down to J1 pin 3 from above
path('NTC', [th_n, (j1[3][0], th_n[1]), (j1[3][0], pad_top + 1.0)], 0.4, B)


# ------------------------------------------------------------------ outline and labels
def edge(a, b):
    s = pcbnew.PCB_SHAPE(board)
    s.SetShape(pcbnew.SHAPE_T_SEGMENT); s.SetStart(V(*a)); s.SetEnd(V(*b))
    s.SetLayer(pcbnew.Edge_Cuts); s.SetWidth(mm(0.1)); board.Add(s)


for a, b in (((0, 0), (W, 0)), ((W, 0), (W, H)), ((W, H), (0, H)), ((0, H), (0, 0))):
    edge(a, b)


def text(s, x, y, layer, size=1.2, mirror=False):
    t = pcbnew.PCB_TEXT(board)
    t.SetText(s); t.SetPosition(V(x, y)); t.SetLayer(layer)
    t.SetTextSize(pcbnew.VECTOR2I(mm(size), mm(size))); t.SetTextThickness(mm(size * 0.15))
    if mirror:
        t.SetMirrored(True)
    board.Add(t)


fig = design.string_figures()
label_y = max(cell_bottom + 2.5, 93.5)
text(f'CubeSat Classroom  1U SOLAR FACE {VAR.upper()}  REV A', W / 2, label_y, pcbnew.F_SilkS, 1.2)
text('UNBUILT  CELL PADS NOT VERIFIED', W / 2, label_y + 2.2, pcbnew.F_SilkS, 1.0)
text('J1: 1 PV+  2 GND  3 NTC', W / 2, 83.5, pcbnew.B_SilkS, 1.0, True)
text('JP1 bridged = no diode', 64.0, 82.5, pcbnew.B_SilkS, 0.9, True)
text(f'{fig["cell_count"]} cells  Vmp {fig["vmp"]:.2f} V  desk figure', W / 2, 12.0, pcbnew.B_SilkS, 1.0, True)
text('Hold by the top and bottom edges only', W / 2, 30.0, pcbnew.B_SilkS, 1.0, True)

# ------------------------------------------------------------------ rules and save
pcbnew.SaveBoard(f'{OUT}/{PROJ}.kicad_pcb', board)
pro = json.load(open(f'{OUT}/{PROJ}.kicad_pro'))
pro.setdefault('board', {})['design_settings'] = {
    'defaults': {}, 'meta': {'version': 2},
    'rules': {'min_clearance': 0.2, 'min_track_width': 0.2, 'min_via_diameter': 0.6, 'min_through_hole_diameter': 0.3,
              'min_via_annular_width': 0.15, 'min_hole_clearance': 0.25, 'min_hole_to_hole': 0.25,
              'min_copper_edge_clearance': 0.5, 'min_silk_clearance': 0.0, 'min_text_height': 0.6,
              'min_text_thickness': 0.08, 'solder_mask_to_copper_clearance': 0.0, 'min_connection': 0.0,
              'min_resolved_spokes': 1, 'allow_blind_buried_vias': False, 'allow_microvias': False, 'max_error': 0.005},
    'rule_severities': {'silk_overlap': 'ignore', 'silk_over_copper': 'ignore', 'silk_edge_clearance': 'ignore',
                        'lib_footprint_issues': 'ignore', 'lib_footprint_mismatch': 'ignore'},
    'track_widths': [0.0, 0.4, 0.8, 1.2], 'via_dimensions': [{'diameter': 0.0, 'drill': 0.0}, {'diameter': 0.8, 'drill': 0.4}]}
json.dump(pro, open(f'{OUT}/{PROJ}.kicad_pro', 'w'), indent=1)
print(f'{VAR}: footprints {len(fps)} tracks+vias {len(tracks)}')
