"""CSC-R1P pedal control board, Rev A: the knobs, LED and display under the pedal's face.

It hangs from the face by the pot and encoder nuts and talks to the main board over I2C through
an 8-pin stacking connector (15 mm M2.5 standoffs between the boards). Everything here runs from
the main board's 3V3_DIG rail.

  TUNE   Alps EC11E encoder with push switch     -> TCA9534 (0x21) P0, P1, P2
  GAIN   Alpha 9 mm pot                           -> ADS1015 (0x48) AIN0
  LEVEL  Alpha 9 mm pot                           -> ADS1015 (0x48) AIN1
  ENGAGE soft-touch footswitch (panel, wired)     -> TCA9534 P3
  BAND   mini toggle (panel, wired)               -> TCA9534 P4   (closed = 70 cm)
  LED    3 mm red engage LED through the face     <- TCA9534 P5 (low = lit)
  OLED   0.91 inch 128x32 SSD1306 I2C module (0x3C), soldered by its header pins

I2C addresses on the shared bus: 0x20 main expander, 0x21 controls expander, 0x3C OLED,
0x48 pot ADC, 0x4C ADC, 0x60 trim DAC. No two are the same.
"""
from collections import OrderedDict, defaultdict
parts = OrderedDict(); _cnt = defaultdict(int)
FP_R = 'Resistor_SMD:R_0402_1005Metric'; FP_C = 'Capacitor_SMD:C_0402_1005Metric'
FP_C8 = 'Capacitor_SMD:C_0805_2012Metric'
FP_JST2 = 'Connector_JST:JST_PH_B2B-PH-K_1x02_P2.00mm_Vertical'
NC = '__NC__'
TITLE = 'CSC-R1P satellite pedal, control board, Rev A'
LADDERS = {}   # no RF on this board; rf_checks.py skips it


def add(prefix, lib, sym, value, fp, pins, block, dnp=False, mpn='', note='', ref=None):
    if ref is None:
        _cnt[prefix] += 1; ref = f'{prefix}{_cnt[prefix]}'
    assert ref not in parts, ref
    parts[ref] = dict(ref=ref, lib=lib, sym=sym, value=value, fp=fp, pins={str(k): v for k, v in pins.items()},
                      block=block, dnp=dnp, mpn=mpn, note=note)
    return ref
def R(v, a, b, blk, **k): return add('R', 'Device', 'R_Small', v, FP_R, {1: a, 2: b}, blk, **k)
def C(v, a, b, blk, fp=FP_C, **k): return add('C', 'Device', 'C_Small', v, fp, {1: a, 2: b}, blk, **k)


B = '1 Link to the main board'
add('J', 'Connector_Generic', 'Conn_02x04_Odd_Even', 'FROM MAIN', 'Connector_PinSocket_2.54mm:PinSocket_2x04_P2.54mm_Vertical',
    {1: '3V3_DIG', 2: 'GND', 3: 'SDA', 4: 'SCL', 5: 'CTL_INT', 6: 'GND', 7: '+5V', 8: 'GND'}, B,
    note='on the underside; mates with the main board TO CONTROLS header')
C('10u', '3V3_DIG', 'GND', B, fp=FP_C8)
C('100n', '+5V', 'GND', B, note='+5V is unused on Rev A; kept for later add-ons')

B = '2 Switches and LED'
add('U', 'Interface_Expansion', 'TCA9534', 'TCA9534PWR', 'Package_SO:TSSOP-16_4.4x5mm_P0.65mm',
    {1: '3V3_DIG', 2: 'GND', 3: 'GND', 4: 'ENC_A', 5: 'ENC_B', 6: 'ENC_SW', 7: 'FOOT_SW', 8: 'GND', 9: 'BAND_SW',
     10: 'LED_ENG_K', 11: 'SPARE6', 12: 'SPARE7', 13: 'CTL_INT', 14: 'SCL', 15: 'SDA', 16: '3V3_DIG'}, B,
    mpn='TI TCA9534PWR', note='I2C 0x21 (A0 high); INT is open drain, pulled up on the main board')
C('100n', '3V3_DIG', 'GND', B)
for n in ('ENC_A', 'ENC_B', 'ENC_SW', 'FOOT_SW', 'BAND_SW', 'SPARE6', 'SPARE7'):
    R('10k', n, '3V3_DIG', B, note='pull-up: the expander has none')
for n in ('ENC_A', 'ENC_B', 'ENC_SW', 'FOOT_SW'):
    C('10n', n, 'GND', B, note='debounce with the 10k pull-up')
add('SW', 'Device', 'RotaryEncoder_Switch', 'TUNE', 'Rotary_Encoder:RotaryEncoder_Alps_EC11E-Switch_Vertical_H20mm',
    {'A': 'ENC_A', 'B': 'ENC_B', 'C': 'GND', 'S1': 'ENC_SW', 'S2': 'GND'}, B,
    mpn='Alps EC11E18244A5 (or EC11E with push switch, 20 detents)', note='turn = 5 kHz steps, push = step size')
add('J', 'Connector', 'Conn_01x02_Pin', 'ENGAGE FOOTSWITCH', FP_JST2, {1: 'FOOT_SW', 2: 'GND'}, B,
    mpn='JST B2B-PH-K-S', note='to a soft-touch momentary SPST footswitch on the face')
add('J', 'Connector', 'Conn_01x02_Pin', 'BAND TOGGLE', FP_JST2, {1: 'BAND_SW', 2: 'GND'}, B,
    mpn='JST B2B-PH-K-S', note='to a mini SPST/SPDT toggle on the face: closed = 70 cm, open = 2 m')
R('1k', '3V3_DIG', 'LED_ENG_A', B)
add('D', 'Device', 'LED', 'ENGAGE red', 'LED_THT:LED_D3.0mm', {1: 'LED_ENG_K', 2: 'LED_ENG_A'}, B,
    mpn='3 mm red LED in a bezel through the face')

B = '3 Knobs'
add('U', 'Analog_ADC', 'ADS1015IDGS', 'ADS1015', 'Package_SO:TSSOP-10_3x3mm_P0.5mm',
    {1: 'GND', 2: NC, 3: 'GND', 4: 'POT_GAIN', 5: 'POT_LEVEL', 6: 'GND', 7: 'GND', 8: '3V3_DIG', 9: 'SDA', 10: 'SCL'}, B,
    mpn='TI ADS1015IDGSR', note='I2C 0x48 (ADDR to GND); unused inputs grounded')
C('100n', '3V3_DIG', 'GND', B)
for name, net in (('GAIN', 'POT_GAIN'), ('LEVEL', 'POT_LEVEL')):
    add('RV', 'Device', 'R_Potentiometer', f'{name} 10k B', 'Potentiometer_THT:Potentiometer_Alpha_RD901F-40-00D_Single_Vertical',
        {1: 'GND', 2: net + '_W', 3: '3V3_DIG'}, B, mpn='Alpha RD901F-40 9 mm, 10k linear (B)')
    R('1k', net + '_W', net, B); C('100n', net, 'GND', B, note='quiet the pot reading')

B = '4 Display'
add('J', 'Connector', 'Conn_01x04_Pin', 'OLED', 'Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical',
    {1: 'GND', 2: '3V3_DIG', 3: 'SCL', 4: 'SDA'}, B,
    mpn='0.91 inch 128x32 SSD1306 I2C module',
    note='solder the module by its own header pins, body about 3 mm above the board (a socket is too tall for the face). '
         'GND, VCC, SCL, SDA is the common order: check your module before fitting')
for i in range(3):
    add('H', 'Mechanical', 'MountingHole_Pad', 'M2.5', 'MountingHole:MountingHole_2.7mm_M2.5_Pad_Via', {1: 'GND'}, B,
        note='15 mm M2.5 standoff to the main board')

SILK = [('CSC-R1P controls  Rev A', 15.0, 50.0, 1.4), ('UNTESTED PROTOTYPE', 15.0, 52.5, 1.0)]


def nets():
    n = defaultdict(list)
    for p in parts.values():
        for pin, net in p['pins'].items():
            if net != NC: n[net].append((p['ref'], pin))
    return n


if __name__ == '__main__':
    n = nets(); print(len(parts), 'parts', len(n), 'nets')
    bad = 0
    for k, v in sorted(n.items()):
        if len(v) < 2: print('SINGLE-PIN NET', k, v); bad += 1
    raise SystemExit(1 if bad else 0)
