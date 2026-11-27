"""The pedal's receiver chain must be the dev kit's, part for part and value for value.

Compares the RF, filter, demodulator, ADC and synthesizer blocks (2 to 6) of design.py and
design_r1p.py. Only the differences listed in EXPECTED are allowed; anything else - a value
changed in one design and not the other, a part added or dropped - fails with exit code 1.
Run: python3 check_shared.py
"""
import collections
import sys

import design
import design_r1p

# Each entry: (only in the dev kit, only in the pedal, why)
EXPECTED = [
    ({('Conn_Coaxial', 'ANT (SMA)', 'Connector_Coaxial:SMA_Samtec_SMA-J-P-X-ST-EM1_EdgeMount', (('1', 'ANT'), ('2', 'GND')))},
     {('Conn_Coaxial', 'ANT', 'Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical', (('1', 'ANT'), ('2', 'GND')))},
     'antenna: edge SMA on the kit, U.FL to a bulkhead SMA pigtail in the pedal'),
    ({('Conn_Coaxial', 'BP1 ANT side', 'Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical', (('1', 'RF_A'), ('2', 'GND'))),
      ('Conn_Coaxial', 'BP1 LNA side', 'Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical', (('1', 'LNA_IN'), ('2', 'GND'))),
      ('R_Small', '0R', 'Resistor_SMD:R_0402_1005Metric', (('1', 'RF_A'), ('2', 'LNA_IN'))),
      ('C_Small', '1n', 'Capacitor_SMD:C_0402_1005Metric', (('1', 'LNA_IN'), ('2', 'LNA_G')))},
     {('C_Small', '1n', 'Capacitor_SMD:C_0402_1005Metric', (('1', 'RF_A'), ('2', 'LNA_G')))},
     'break point 1 removed: the LNA input capacitor now sits on RF_A'),
    ({('Conn_01x06_Pin', 'BB TEST', 'Connector_PinHeader_2.54mm:PinHeader_1x06_P2.54mm_Vertical',
       (('1', 'BBF_IP'), ('2', 'BBF_IN'), ('3', 'GND'), ('4', 'BBF_QP'), ('5', 'BBF_QN'), ('6', 'GND')))},
     set(), 'baseband test header removed'),
    ({('Jumper_3_Bridged12', 'MCLK OSC/CTRL', 'Connector_PinHeader_2.54mm:PinHeader_1x03_P2.54mm_Vertical',
       (('1', 'MCLK_OSC'), ('2', 'ADC_MCLK'), ('3', 'XD6'))),
      ('R_Small', '33R', 'Resistor_SMD:R_0402_1005Metric', (('1', 'MCLK_OSC_R'), ('2', 'MCLK_OSC')))},
     {('R_Small', '33R', 'Resistor_SMD:R_0402_1005Metric', (('1', 'MCLK_OSC_R'), ('2', 'ADC_MCLK')))},
     'MCLK jumper removed: the oscillator always clocks the ADC (XIAO D6 is the controls interrupt)'),
    ({('R_Small', '33R', 'Resistor_SMD:R_0402_1005Metric', (('1', 'PLL_MUXOUT_R'), ('2', 'PLL_MUXOUT'))),
      ('R_Small', '2k2', 'Resistor_SMD:R_0402_1005Metric', (('1', 'PLL_LD'), ('2', 'LED_LOCK_A'))),
      ('LED_Small', 'LOCK green', 'LED_SMD:LED_0603_1608Metric', (('1', 'GND'), ('2', 'LED_LOCK_A'))),
      ('TestPoint', 'LD', 'TestPoint:TestPoint_Pad_D1.0mm', (('1', 'PLL_LD'),)),
      ('TestPoint', 'PDBRF', 'TestPoint:TestPoint_Pad_D1.0mm', (('1', 'PLL_PDBRF'),))},
     set(), 'lock LED, LD and PDBRF test points and the MUXOUT resistor left off the pedal (lock is read from MUXOUT)'),
]


def signatures(mod):
    sig = collections.Counter()
    for p in mod.parts.values():
        if p['block'][0] in '23456':
            pins = dict(p['pins'])
            if p['sym'] == 'ADF4351':
                pins.pop('25', None); pins['30'] = 'PLL_MUXOUT'   # LD unused, MUXOUT direct in the pedal
            sig[(p['sym'], p['value'], p['fp'], tuple(sorted(pins.items())))] += 1
    return sig


def main():
    kit, pedal = signatures(design), signatures(design_r1p)
    only_kit = kit - pedal
    only_pedal = pedal - kit
    for k_, p_, why in EXPECTED:
        for s in k_:
            if only_kit[s] <= 0:
                print('EXPECTED DIFFERENCE MISSING (kit side):', why, s); return 1
            only_kit[s] -= 1
        for s in p_:
            if only_pedal[s] <= 0:
                print('EXPECTED DIFFERENCE MISSING (pedal side):', why, s); return 1
            only_pedal[s] -= 1
    bad = [('dev kit only', s) for s, n in only_kit.items() if n > 0] + [('pedal only', s) for s, n in only_pedal.items() if n > 0]
    for side, s in bad:
        print('UNEXPECTED', side, s)
    for name in ('F2', 'F70'):
        vk = [design.parts[r]['value'] for r in design.LADDERS[name]['chain']]
        vp = [design_r1p.parts[r]['value'] for r in design_r1p.LADDERS[name]['chain']]
        if vk != vp:
            print('FILTER', name, 'differs:', vk, vp); bad.append(name)
    print('shared receiver chain:', 'identical apart from the listed differences' if not bad else f'{len(bad)} unexpected differences')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
