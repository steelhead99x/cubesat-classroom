"""CSC-R1P pedal main board, Rev A - single source of truth for its schematic and PCB.

The receiver chain (bias tee, LNA, band filters, demodulator, synthesizer, ADC) is the dev kit's
(design.py), part for part and value for value; check_shared.py fails if the two drift apart.
What changes for the pedal:
  - 9 V pedal power (2.1 mm centre-negative jack, wired to J_DC) through reverse-polarity
    protection, a TVS, a fuse and a TPS62933F buck to 5.1 V. No USB-C power input: the XIAO's
    own USB-C (reached through the box's top wall) is for data and flashing.
  - Audio out: PCM5102A DAC on the ADC's I2S clocks, line level to the 1/4 inch OUT jack.
  - Controls on a separate board (design_r1p_ctl.py) over I2C, joined by J_B2B.
  - Antenna: U.FL, to a bulkhead SMA on the box wall by a short pigtail.
  - Dev-kit extras removed: BP1, current jumpers, controller header, AUX, BB test header,
    MCLK jumper, user LED, lock LED, LD and PDBRF test points, MUXOUT series resistor.
    Break points 2 and 3 stay for the bench filter quest.
Every part: ref, symbol (lib, name), value, footprint, pins {number: net}, block, fields."""
from collections import OrderedDict, defaultdict
parts = OrderedDict(); _cnt = defaultdict(int)
FP_R = 'Resistor_SMD:R_0402_1005Metric'; FP_C = 'Capacitor_SMD:C_0402_1005Metric'
FP_C6 = 'Capacitor_SMD:C_0603_1608Metric'; FP_C8 = 'Capacitor_SMD:C_0805_2012Metric'
FP_C12 = 'Capacitor_SMD:C_1206_3216Metric'
FP_L = 'Inductor_SMD:L_0402_1005Metric'; FP_L6 = 'Inductor_SMD:L_0603_1608Metric'
FP_LED = 'LED_SMD:LED_0603_1608Metric'; FP_UFL = 'Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical'
FP_TP = 'TestPoint:TestPoint_Pad_D1.0mm'
FP_JST2 = 'Connector_JST:JST_PH_B2B-PH-K_1x02_P2.00mm_Vertical'
NC = '__NC__'
TITLE = 'CSC-R1P satellite pedal, main board, Rev A'


def add(prefix, lib, sym, value, fp, pins, block, dnp=False, mpn='', note='', ref=None):
    if ref is None:
        _cnt[prefix] += 1; ref = f'{prefix}{_cnt[prefix]}'
    assert ref not in parts, ref
    parts[ref] = dict(ref=ref, lib=lib, sym=sym, value=value, fp=fp, pins={str(k): v for k, v in pins.items()},
                      block=block, dnp=dnp, mpn=mpn, note=note)
    return ref
def R(v, a, b, blk, **k): return add('R', 'Device', 'R_Small', v, FP_R, {1: a, 2: b}, blk, **k)
def C(v, a, b, blk, fp=FP_C, **k): return add('C', 'Device', 'C_Small', v, fp, {1: a, 2: b}, blk, **k)
def L(v, a, b, blk, fp=FP_L, **k): return add('L', 'Device', 'L_Small', v, fp, {1: a, 2: b}, blk, **k)
def FB(a, b, blk): return add('FB', 'Device', 'FerriteBead_Small', '220R@100MHz', FP_L6, {1: a, 2: b}, blk, mpn='Murata BLM18PG221SN1D (DCR <= 0.1 ohm)')
def LED(v, a, k, blk): return add('D', 'Device', 'LED_Small', v, FP_LED, {1: k, 2: a}, blk)
def UFL(name, sig, blk): return add('J', 'Connector', 'Conn_Coaxial', name, FP_UFL, {1: sig, 2: 'GND'}, blk, mpn='Hirose U.FL-R-SMT-1(80)')
def TP(name, net, blk): return add('TP', 'Connector', 'TestPoint', name, FP_TP, {1: net}, blk)
def bypass(net, blk, vals=('100n',)):
    for v in vals:
        fp = FP_C8 if v in ('10u', '22u') else FP_C6 if v in ('1u', '2u2') else FP_C
        C(v, net, 'GND', blk, fp=fp)


# ---------------------------------------------------------------- POWER: 9 V pedal supply -> 5.1 V
B = '1 Power input (9 V pedal supply)'
add('J', 'Connector', 'Conn_01x02_Pin', 'DC 9V IN', FP_JST2, {1: 'VDC_RAW', 2: 'GND'}, B,
    mpn='JST B2B-PH-K-S', note='wire to a 2.1 mm centre-negative panel jack: sleeve = +9 V to pin 1, centre pin = GND to pin 2')
add('Q', 'Transistor_FET', 'AO3401A', 'AO3401A', 'Package_TO_SOT_SMD:SOT-23', {1: 'VDC_G', 2: 'VDC', 3: 'VDC_RAW'}, B,
    mpn='AOS AO3401A (-30 V)', note='reverse-polarity protection: a backwards supply leaves the gate off')
R('10k', 'VDC_G', 'GND', B)
add('D', 'Diode', 'BZT52Bxx', 'BZT52C10', 'Diode_SMD:D_SOD-123', {1: 'VDC', 2: 'VDC_G'}, B,
    mpn='BZT52C10 (10 V zener)', note='clamps the FET gate to 10 V (AO3401A gate limit is 12 V)')
add('D', 'Device', 'D_Zener', 'SMAJ15A', 'Diode_SMD:D_SMA', {1: 'VDC', 2: 'GND'}, B,
    mpn='SMAJ15A (15 V stand-off, unidirectional)', note='input surge clamp; cathode (band, pad 1) to VDC. Use a 9-12 V supply, not 18 V')
add('F', 'Device', 'Polyfuse_Small', '1A hold', 'Fuse:Fuse_1812_4532Metric', {1: 'VDC', 2: 'VBUCK_IN_F'}, B, mpn='PTC resettable 1 A hold, >= 16 V, 1812')
FB('VBUCK_IN_F', 'VBUCK_IN', B)
C('10u', 'VBUCK_IN', 'GND', B, fp=FP_C12, note='25 V X7R'); C('10u', 'VBUCK_IN', 'GND', B, fp=FP_C12, note='25 V X7R'); C('100n', 'VBUCK_IN', 'GND', B, fp=FP_C6, note='25 V')
# TPS62933F: forced continuous mode (fixed frequency at every load), RT 9.09k -> about 2.1 MHz.
# At 2.1 MHz its harmonics fall at 144.9 and 147.0 MHz, clear of 145.8-146.0 MHz; they still need measuring.
add('U', 'Regulator_Switching', 'TPS62933F', 'TPS62933F', 'Package_TO_SOT_SMD:SOT-583-8',
    {1: 'BUCK_RT', 2: NC, 3: 'VBUCK_IN', 4: 'GND', 5: 'BUCK_SW', 6: 'BUCK_BST', 7: 'BUCK_SS', 8: 'BUCK_FB'}, B,
    mpn='TI TPS62933FDRLR', note='3.8-30 V in, 3 A, FCCM; EN left floating = enabled')
R('9k09', 'BUCK_RT', 'GND', B, note='1%: about 2.1 MHz')
C('10n', 'BUCK_SS', 'GND', B, note='soft start'); C('100n', 'BUCK_BST', 'BUCK_SW', B, note='bootstrap')
L('2.2uH', 'BUCK_SW', 'VBUCK_OUT', B, fp='Inductor_SMD:L_Coilcraft_XAL4020-XXX', mpn='Coilcraft XAL4020-222ME (shielded, check saturation >= 3 A)')
R('53k6', 'VBUCK_OUT', 'BUCK_FB', B, note='1%: 0.8 V x (1 + 53.6/10) = 5.09 V'); R('10k', 'BUCK_FB', 'GND', B, note='1%')
C('22u', 'VBUCK_OUT', 'GND', B, fp=FP_C8, note='10 V X7R'); C('22u', 'VBUCK_OUT', 'GND', B, fp=FP_C8, note='10 V X7R')
# second-stage filter: keeps the switching ripple off the receiver rails
FB('VBUCK_OUT', '+5V', B); bypass('+5V', B, ('22u', '22u', '100n'))
R('2k2', '+5V', 'LED_PWR_A', B); LED('PWR green', 'LED_PWR_A', 'GND', B)
FB('+5V', '5V_DEMOD', B); bypass('5V_DEMOD', B, ('10u',))
FB('+5V', '5V_LNA', B); bypass('5V_LNA', B, ('10u',))
def ldo(rail):
    add('U', 'Regulator_Linear', 'LP5907MFX-3.3', 'LP5907MFX-3.3', 'Package_TO_SOT_SMD:SOT-23-5',
        {1: '+5V', 2: 'GND', 3: '+5V', 4: NC, 5: rail}, B, mpn='TI LP5907MFX-3.3/NOPB')
    C('1u', '+5V', 'GND', B, fp=FP_C6); C('1u', rail, 'GND', B, fp=FP_C6)
for r in ('3V3_VCO', '3V3_PLL', '3V3_ADC', '3V3_DIG', '3V3_AUD'): ldo(r)

# ---------------------------------------------------------------- RF FRONT END (as the dev kit, without BP1)
B = '2 RF input, bias tee, LNA'
UFL('ANT', 'ANT', B)
add('F', 'Device', 'Polyfuse_Small', '100mA hold', 'Fuse:Fuse_0805_2012Metric', {1: '+5V', 2: 'BT_FUSED'}, B, mpn='PTC resettable 100mA 0805')
add('Q', 'Transistor_FET', 'AO3401A', 'AO3401A', 'Package_TO_SOT_SMD:SOT-23', {1: 'BT_GATE', 2: 'BT_FUSED', 3: 'BT_SW'}, B, mpn='AOS AO3401A')
add('Q', 'Transistor_FET', '2N7002', '2N7002', 'Package_TO_SOT_SMD:SOT-23', {1: 'BIAS_EN', 2: 'GND', 3: 'BT_GATE'}, B, mpn='2N7002')
R('100k', 'BT_FUSED', 'BT_GATE', B); R('100k', 'BIAS_EN', 'GND', B)
R('10R', 'BT_SW', 'BT_FILT', B); C('100n', 'BT_FILT', 'GND', B); C('1n', 'BT_FILT', 'GND', B)
L('470nH', 'BT_FILT', 'ANT', B, fp=FP_L6, mpn='0603 wirewound, SRF > 600 MHz')
R('1k', 'BT_SW', 'LED_BT_A', B); LED('BIAS-T amber', 'LED_BT_A', 'GND', B)
C('1n', 'ANT', 'RF_A', B, note='DC block, C0G')
L('470nH', 'RF_A', 'GND', B, fp=FP_L6, note='static bleed')
add('D', 'Diode', 'BAV99', 'BAV99', 'Package_TO_SOT_SMD:SOT-23', {1: 'GND', 2: 'GND', 3: 'RF_A'}, B, mpn='BAV99', note='antiparallel clamp')
C('1n', 'RF_A', 'LNA_G', B)
add('U', 'RF_Amplifier', 'PGA-103', 'PGA-103+', 'Package_TO_SOT_SMD:SOT-89-3', {1: 'LNA_G', 2: 'GND', 3: 'LNA_D'}, B, mpn='Mini-Circuits PGA-103+')
L('470nH', '5V_LNA', 'LNA_D', B, fp=FP_L6, mpn='0603 wirewound, >= 150 mA'); bypass('5V_LNA', B, ('100n', '1n'))
C('1n', 'LNA_D', 'LNA_OUT', B)
UFL('BP2 LNA side', 'LNA_OUT', B); R('0R', 'LNA_OUT', 'FILT_IN', B, note='BP2 link'); UFL('BP2 filter side', 'FILT_IN', B)

B = '3 Band filters'
def sw(rfc, rf1, rf2, blk, ctrl='BAND_SEL'):
    add('U', 'CSC_R1', 'PE4259', 'PE4259', 'Package_TO_SOT_SMD:SOT-363_SC-70-6',
        {1: rf1, 2: 'GND', 3: rf2, 4: ctrl, 5: rfc, 6: '3V3_DIG'}, blk, mpn='pSemi PE4259-63')
    C('100n', '3V3_DIG', 'GND', blk)
sw('FILT_IN', 'F70_IN', 'F2_IN', B)
LADDERS = {}
def ladder(pfx, nin, nout, hp, lp, blk):
    chain = []; shunts = {}; node = nin; k = 0
    for i, v in enumerate(hp):
        if i % 2 == 0:
            k += 1; nxt = f'{pfx}_H{k}' if i < len(hp) - 1 else f'{pfx}_M'
            chain.append(C(v, node, nxt, blk)); node = nxt
        else: shunts.setdefault(node, []).append(L(v, node, 'GND', blk))
    k = 0
    for i, v in enumerate(lp):
        if i % 2 == 0: shunts.setdefault(node, []).append(C(v, node, 'GND', blk))
        else:
            k += 1; nxt = f'{pfx}_L{k}' if i < len(lp) - 2 else nout
            chain.append(L(v, node, nxt, blk)); node = nxt
    assert node == nout
    LADDERS[pfx] = dict(nin=nin, nout=nout, chain=chain, shunts=shunts)
ladder('F2', 'F2_IN', 'F2_OUT', ('22p', '47nH', '12p', '39nH', '12p', '47nH', '22p'), ('18p', '56nH', '33p', '56nH', '18p'), B)
ladder('F70', 'F70_IN', 'F70_OUT', ('8p2', '15nH', '4p7', '15nH', '8p2'), ('6p8', '22nH', '12p', '22nH', '12p', '22nH', '6p8'), B)
sw('FILT_OUT', 'F2_OUT', 'F70_OUT', B, ctrl='BAND_SEL_N')
add('Q', 'Transistor_FET', '2N7002', '2N7002', 'Package_TO_SOT_SMD:SOT-23', {1: 'BAND_SEL', 2: 'GND', 3: 'BAND_SEL_N'}, B, mpn='2N7002', note='inverter for output switch')
R('10k', 'BAND_SEL_N', '3V3_DIG', B)
R('100k', 'BAND_SEL', 'GND', B)
UFL('BP3 filter side', 'FILT_OUT', B); R('0R', 'FILT_OUT', 'DEM_RF', B, note='BP3 link'); UFL('BP3 demod side', 'DEM_RF', B)

# ---------------------------------------------------------------- DEMODULATOR (as the dev kit)
B = '4 I/Q demodulator'
C('DNP', 'DEM_RF', 'GND', B, dnp=True, note='match pad'); R('0R', 'DEM_RF', 'DEM_RF_M', B, note='match series pad'); C('DNP', 'DEM_RF_M', 'GND', B, dnp=True, note='match pad')
add('T', 'Transformer', 'TC1-1-13M+', 'TC1-1-13M+', 'Transformer_SMD:Transformer_MiniCircuits_AT224-1A',
    {6: 'DEM_RF_M', 4: 'GND', 1: 'RF_BAL_P', 3: 'RF_BAL_N'}, B, mpn='Mini-Circuits TC1-1-13M+')
L('15nH', 'RF_BAL_P', 'RF_BAL_N', B, dnp=True, note='L6 per LTC5584 data sheet: 15nH @450MHz, 68nH @140MHz; confirm position against data sheet Figure 4')
C('10n', 'RF_BAL_P', 'DEM_RFP', B); C('10n', 'RF_BAL_N', 'DEM_RFN', B)
C('DNP', 'DEM_RFP', 'DEM_RFN', B, dnp=True, note='match pad: differential shunt C at RF+/RF-')
add('U', 'CSC_R1', 'LTC5584', 'LTC5584IUF', 'CSC_R1:LTC_UF24_QFN-24-1EP_4x4mm_P0.5mm_EP2.45mm',
    {1: 'TRIM_IP2Q', 2: 'TRIM_DCOQ', 3: 'TRIM_DCOI', 4: 'TRIM_IP2I', 5: 'DEM_RFP', 6: 'DEM_RFN', 7: 'DEMOD_EN', 8: 'GND', 9: 'DEM_VBIAS',
     10: '5V_DEMOD', 11: 'DEMOD_EDC', 12: 'DEMOD_EIP2', 13: 'GND', 14: 'GND', 15: 'LO_P', 16: 'LO_N', 17: 'DEM_VCAP', 18: 'DEM_CMQ', 19: 'DEM_CMI',
     20: 'BB_QN', 21: 'BB_QP', 22: 'BB_IN', 23: 'BB_IP', 24: 'DEM_REF', 25: 'GND'}, B, mpn='Analog Devices LTC5584IUF#PBF')
bypass('5V_DEMOD', B, ('1u', '10n'))
R('294R', 'DEM_VBIAS', 'GND', B, dnp=True, note='optional: 294R cuts ICC to ~145 mA (lower IIP3)')
C('100n', 'DEM_CMQ', 'DEM_VCAP', B); C('100n', 'DEM_CMI', 'DEM_VCAP', B); C('100n', 'DEM_REF', 'GND', B)
for n in ('BB_IP', 'BB_IN', 'BB_QP', 'BB_QN'): R('100R', '5V_DEMOD', n, B, note='output pull-up, 1%')
R('100k', 'DEMOD_EN', 'GND', B)
add('U', 'Analog_DAC', 'MCP4728', 'MCP4728', 'Package_SO:MSOP-10_3x3mm_P0.5mm',
    {1: '3V3_DIG', 2: 'SCL', 3: 'SDA', 4: 'GND', 5: NC, 6: 'DAC_A', 7: 'DAC_B', 8: 'DAC_C', 9: 'DAC_D', 10: 'GND'}, B, mpn='Microchip MCP4728-E/UN', note='I2C 0x60')
bypass('3V3_DIG', B, ('100n', '10u'))
for d, t in (('DAC_A', 'TRIM_IP2I'), ('DAC_B', 'TRIM_DCOI'), ('DAC_C', 'TRIM_DCOQ'), ('DAC_D', 'TRIM_IP2Q')):
    R('330R', d, t, B); R('330R', t, 'GND', B); C('100n', t, 'GND', B)

B = '5 Baseband and ADC'
for s, adc in (('I', 'ADC_IN1'), ('Q', 'ADC_IN2')):
    R('100R', f'BB_{s}P', f'BBF_{s}P', B); R('100R', f'BB_{s}N', f'BBF_{s}N', B)
    C('2n2', f'BBF_{s}P', f'BBF_{s}N', B, note='C0G; ~240 kHz corner')
    C('1u', f'BBF_{s}P', adc + 'P', B, fp=FP_C6, note='X7R 25V'); C('1u', f'BBF_{s}N', adc + 'M', B, fp=FP_C6, note='X7R 25V')
add('U', 'CSC_R1', 'TLV320ADC3140', 'TLV320ADC3140', 'Package_DFN_QFN:Texas_RTW_WQFN-24-1EP_4x4mm_P0.5mm_EP2.7x2.7mm_ThermalVias',
    {1: '3V3_ADC', 2: 'ADC_AREG', 3: 'ADC_VREF', 4: 'GND', 5: NC, 6: 'ADC_IN1P', 7: 'ADC_IN1M', 8: 'ADC_IN2P', 9: 'ADC_IN2M', 10: NC, 11: NC, 12: NC, 13: NC,
     14: 'ADC_SHDNZ', 15: 'GND', 16: 'GND', 17: 'SCL', 18: 'SDA', 19: '3V3_DIG', 20: 'ADC_MCLK', 21: 'ADC_SDOUT', 22: 'ADC_BCLK', 23: 'ADC_FSYNC',
     24: 'ADC_DREG', 25: 'GND'}, B, mpn='TI TLV320ADC3140IRTWR', note='I2C 0x4C; controller mode: drives BCLK and FSYNC for the XIAO and the audio DAC')
bypass('3V3_ADC', B, ('10u', '100n')); bypass('ADC_AREG', B, ('10u', '100n')); bypass('ADC_VREF', B, ('1u',))
bypass('ADC_DREG', B, ('10u', '100n')); bypass('3V3_DIG', B, ('100n',))
R('100k', 'ADC_SHDNZ', 'GND', B)
R('33R', 'ADC_SDOUT', 'I2S_DOUT', B); R('33R', 'ADC_BCLK', 'I2S_BCLK', B); R('33R', 'ADC_FSYNC', 'I2S_FSYNC', B)
add('Y', 'Oscillator', 'ASE-xxxMHz', '24.576MHz', 'Oscillator:Oscillator_SMD_Abracon_ASE-4Pin_3.2x2.5mm',
    {1: '3V3_MCLK', 2: 'GND', 3: 'MCLK_OSC_R', 4: '3V3_MCLK'}, B, mpn='Abracon ASE-24.576MHZ-LC-T (3.3 V CMOS)')
FB('3V3_DIG', '3V3_MCLK', B); bypass('3V3_MCLK', B, ('100n',)); R('33R', 'MCLK_OSC_R', 'ADC_MCLK', B)

# ---------------------------------------------------------------- SYNTHESIZER (as the dev kit)
B = '6 Local oscillator'
add('U', 'Timer_PLL', 'ADF4351', 'ADF4351BCPZ', 'Package_CSP:LFCSP-32-1EP_5x5mm_P0.5mm_EP3.1x3.1mm_ThermalVias',
    {1: 'PLL_CLK', 2: 'PLL_DATA', 3: 'PLL_LE', 4: 'PLL_CE', 5: NC, 6: '3V3_PLL', 7: 'PLL_CP', 8: 'GND', 9: 'GND', 10: '3V3_PLL', 11: 'GND',
     12: 'LOA_P', 13: 'LOA_N', 14: 'LOB_P', 15: 'LOB_N', 16: '3V3_VCO', 17: '3V3_VCO', 18: 'GND', 19: 'PLL_TEMP', 20: 'PLL_VTUNE', 21: 'GND',
     22: 'PLL_RSET', 23: 'PLL_VCOM', 24: 'PLL_VREF', 25: NC, 26: 'PLL_PDBRF', 27: 'GND', 28: '3V3_PLL', 29: 'PLL_REFIN', 30: 'PLL_MUXOUT',
     31: 'GND', 32: '3V3_PLL', 33: 'GND'}, B, mpn='Analog Devices ADF4351BCPZ')
for i in range(4): bypass('3V3_PLL', B, ('100n', '10p'))
bypass('3V3_PLL', B, ('10u',)); bypass('3V3_VCO', B, ('100n', '10p', '100n', '10p', '10u'))
R('5k1', 'PLL_RSET', 'GND', B, note='1%'); C('100n', 'PLL_TEMP', 'GND', B); C('100n', 'PLL_VCOM', 'GND', B); C('100n', 'PLL_VREF', 'GND', B)
R('10k', 'PLL_CE', '3V3_DIG', B); R('10k', 'PLL_PDBRF', '3V3_DIG', B)
C('3n9', 'PLL_CP', 'GND', B, note='loop C1, C0G'); R('120R', 'PLL_CP', 'PLL_LF2', B); C('180n', 'PLL_LF2', 'GND', B, fp=FP_C6, note='loop C2, X7R or film')
R('560R', 'PLL_CP', 'PLL_VTUNE', B); C('680p', 'PLL_VTUNE', 'GND', B, note='loop C3, C0G'); TP('VTUNE', 'PLL_VTUNE', B)
add('Y', 'Oscillator', 'TG2520SMN-xx.xxxxxxMhz-xxxxNM', '25MHz TCXO', 'Oscillator:Oscillator_SMD_SeikoEpson_TG2520SMN-xxx-xxxxxx-4Pin_2.5x2.0mm',
    {1: NC, 2: 'GND', 3: 'TCXO_OUT', 4: '3V3_TCXO'}, B, mpn='25 MHz TCXO 2.5x2.0 mm, 3.3 V, <= 2 ppm (Epson TG2520SMN family or equiv.)')
FB('3V3_PLL', '3V3_TCXO', B); bypass('3V3_TCXO', B, ('100n',)); C('1n', 'TCXO_OUT', 'PLL_REFIN', B)
R('51R', '3V3_VCO', 'LOA_P', B, note='or 27 nH choke for more LO power'); R('51R', '3V3_VCO', 'LOA_N', B, note='or 27 nH choke for more LO power')
C('10n', 'LOA_P', 'LO_N', B, note='polarity swapped on purpose: no crossing'); C('10n', 'LOA_N', 'LO_P', B)
R('51R', '3V3_VCO', 'LOB_P', B); R('51R', '3V3_VCO', 'LOB_N', B)
C('1n', 'LOB_P', 'LO_MON', B); UFL('LO MON out', 'LO_MON', B)
# Lock is read from MUXOUT by the firmware: the dev kit's lock LED, LD test point and MUXOUT resistor are left off the pedal.

# ---------------------------------------------------------------- AUDIO OUT
B = '7 Audio out'
# PCM5102A in hardware mode, 3-wire I2S: SCK tied low so its PLL runs from BCK (64 x 192 kHz).
add('U', 'Audio', 'PCM5102A', 'PCM5102A', 'Package_SO:TSSOP-20_4.4x6.5mm_P0.65mm',
    {1: '3V3_AUD', 2: 'DAC_CAPP', 3: 'GND', 4: 'DAC_CAPM', 5: 'DAC_VNEG', 6: 'DAC_OUTL', 7: NC, 8: '3V3_AUD', 9: 'GND',
     10: 'GND', 11: 'GND', 12: 'GND', 13: 'I2S_BCLK', 14: 'DAC_DIN', 15: 'I2S_FSYNC', 16: 'GND', 17: 'DAC_XSMT',
     18: 'DAC_LDOO', 19: 'GND', 20: '3V3_AUD'}, B, mpn='TI PCM5102APWR',
    note='DEMP low (no de-emphasis), FLT low (normal filter), FMT low (I2S), SCK low (BCK PLL); XSMT from the expander, pulled low = muted')
C('2u2', 'DAC_CAPP', 'DAC_CAPM', B, fp=FP_C6, note='charge-pump flying cap, X7R')
C('2u2', 'DAC_VNEG', 'GND', B, fp=FP_C6, note='charge-pump output, X7R')
C('1u', 'DAC_LDOO', 'GND', B, fp=FP_C6)
bypass('3V3_AUD', B, ('10u', '100n', '100n'))
R('100k', 'DAC_XSMT', 'GND', B, note='muted until firmware unmutes')
R('33R', 'XD7', 'DAC_DIN', B)
R('470R', 'DAC_OUTL', 'AUDIO_OUT', B, note='output filter from the PCM510xA data sheet')
C('2n2', 'AUDIO_OUT', 'GND', B, note='C0G')
add('J', 'Connector', 'Conn_01x02_Pin', 'AUDIO OUT', FP_JST2, {1: 'AUDIO_OUT', 2: 'GND'}, B,
    mpn='JST B2B-PH-K-S', note='wire to the 1/4 inch mono OUT jack: tip = pin 1, sleeve = pin 2. Line level, 2.1 V rms full scale')

# ---------------------------------------------------------------- CONTROLLER
B = '8 Controller'
add('J', 'Connector', 'Conn_01x07_Socket', 'XIAO left', 'Connector_PinSocket_2.54mm:PinSocket_1x07_P2.54mm_Vertical',
    {1: 'I2S_BCLK', 2: 'I2S_FSYNC', 3: 'I2S_DOUT', 4: 'PLL_MUXOUT', 5: 'SDA', 6: 'SCL', 7: 'CTL_INT'}, B,
    note='XIAO D0..D6, USB end at pin 1: D0 BCLK, D1 FSYNC, D2 ADC data, D3 lock, D4 SDA, D5 SCL, D6 controls interrupt')
add('J', 'Connector', 'Conn_01x07_Socket', 'XIAO right', 'Connector_PinSocket_2.54mm:PinSocket_1x07_P2.54mm_Vertical',
    {1: 'XIAO_5V', 2: 'GND', 3: NC, 4: 'PLL_DATA', 5: 'PLL_LE', 6: 'PLL_CLK', 7: 'XD7'}, B,
    note='XIAO 5V, GND, 3V3 (unused), D10 synth DATA, D9 synth LE, D8 synth CLK, D7 audio data; USB end at pin 1')
add('D', 'Device', 'D_Schottky_Small', 'B5819W', 'Diode_SMD:D_SOD-123', {2: '+5V', 1: 'XIAO_5V'}, B, mpn='B5819W / MBR0520',
    note='board 5 V feeds the XIAO; no back-feed when the XIAO is on USB alone')
R('4k7', 'SDA', '3V3_DIG', B); R('4k7', 'SCL', '3V3_DIG', B)
add('U', 'Interface_Expansion', 'TCA9534', 'TCA9534PWR', 'Package_SO:TSSOP-16_4.4x5mm_P0.65mm',
    {1: 'GND', 2: 'GND', 3: 'GND', 4: 'BAND_SEL', 5: 'BIAS_EN', 6: 'DEMOD_EN', 7: 'DEMOD_EDC', 8: 'GND', 9: 'DEMOD_EIP2', 10: 'ADC_SHDNZ', 11: 'PLL_CE',
     12: 'DAC_XSMT', 13: 'EXP_INT', 14: 'SCL', 15: 'SDA', 16: '3V3_DIG'}, B, mpn='TI TCA9534PWR', note='I2C 0x20; P7 = audio unmute')
bypass('3V3_DIG', B, ('100n',)); R('10k', 'EXP_INT', '3V3_DIG', B); TP('INT', 'EXP_INT', B)
R('10k', 'CTL_INT', '3V3_DIG', B)
add('J', 'Connector_Generic', 'Conn_02x04_Odd_Even', 'TO CONTROLS', 'Connector_PinHeader_2.54mm:PinHeader_2x04_P2.54mm_Vertical',
    {1: '3V3_DIG', 2: 'GND', 3: 'SDA', 4: 'SCL', 5: 'CTL_INT', 6: 'GND', 7: '+5V', 8: 'GND'}, B,
    note='to the control board (stacking header; 15 mm standoffs)')
add('TP', 'Connector', 'TestPoint', 'GND', 'TestPoint:TestPoint_Loop_D1.80mm_Drill1.0mm_Beaded', {1: 'GND'}, B)
for i in range(3): add('H', 'Mechanical', 'MountingHole_Pad', 'M2.5', 'MountingHole:MountingHole_2.7mm_M2.5_Pad_Via', {1: 'GND'}, B,
                       note='15 mm M2.5 standoff to the control board')


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

SILK = [('CSC-R1P satellite pedal  Rev A', 3.0, 70.5, 1.3), ('receive-only, 2 m / 70 cm', 3.0, 72.8, 1.0),
        ('UNTESTED PROTOTYPE', 3.0, 74.8, 1.0)]
