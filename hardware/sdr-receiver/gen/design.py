"""CSC-R1 dev kit Rev A - single source of truth for schematic and PCB.
Every part: ref, symbol (lib, name), value, footprint, pins {number: net}, block, fields."""
from collections import OrderedDict, defaultdict
parts=OrderedDict(); _cnt=defaultdict(int)
FP_R='Resistor_SMD:R_0402_1005Metric'; FP_C='Capacitor_SMD:C_0402_1005Metric'
FP_C6='Capacitor_SMD:C_0603_1608Metric'; FP_C8='Capacitor_SMD:C_0805_2012Metric'
FP_L='Inductor_SMD:L_0402_1005Metric'; FP_L6='Inductor_SMD:L_0603_1608Metric'
FP_LED='LED_SMD:LED_0603_1608Metric'; FP_UFL='Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical'
FP_H2='Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical'
FP_H3='Connector_PinHeader_2.54mm:PinHeader_1x03_P2.54mm_Vertical'
FP_TP='TestPoint:TestPoint_Pad_D1.0mm'
NC='__NC__'
def add(prefix,lib,sym,value,fp,pins,block,dnp=False,mpn='',note='',ref=None):
    if ref is None:
        _cnt[prefix]+=1; ref=f'{prefix}{_cnt[prefix]}'
    assert ref not in parts, ref
    parts[ref]=dict(ref=ref,lib=lib,sym=sym,value=value,fp=fp,pins={str(k):v for k,v in pins.items()},
                    block=block,dnp=dnp,mpn=mpn,note=note)
    return ref
def R(v,a,b,blk,**k): return add('R','Device','R_Small',v,FP_R,{1:a,2:b},blk,**k)
def C(v,a,b,blk,fp=FP_C,**k): return add('C','Device','C_Small',v,fp,{1:a,2:b},blk,**k)
def L(v,a,b,blk,fp=FP_L,**k): return add('L','Device','L_Small',v,fp,{1:a,2:b},blk,**k)
def FB(a,b,blk): return add('FB','Device','FerriteBead_Small','220R@100MHz',FP_L6,{1:a,2:b},blk,mpn='Murata BLM18PG221SN1D (DCR <= 0.1 ohm)')
def LED(v,a,k,blk): return add('D','Device','LED_Small',v,FP_LED,{1:k,2:a},blk)
def UFL(name,sig,blk): return add('J','Connector','Conn_Coaxial',name,FP_UFL,{1:sig,2:'GND'},blk,mpn='Hirose U.FL-R-SMT-1(80)')
def JP2(name,a,b,blk): return add('JP','Jumper','Jumper_2_Bridged',name,FP_H2,{1:a,2:b},blk,note='fit shunt; remove to measure current')
def TP(name,net,blk): return add('TP','Connector','TestPoint',name,FP_TP,{1:net},blk)
def bypass(net,blk,vals=('100n',)):
    for v in vals:
        fp=FP_C8 if v in('10u','22u') else FP_C6 if v in('1u','2u2') else FP_C
        C(v,net,'GND',blk,fp=fp)

# ---------------------------------------------------------------- POWER
B='1 Power input'
add('J','Connector','USB_C_Receptacle_PowerOnly_6P','USB-C 5V in','Connector_USB:USB_C_Receptacle_GCT_USB4125-xx-x_6P_TopMnt_Horizontal',
    {'A5':'USB_CC1','B5':'USB_CC2','A9':'VUSB','B9':'VUSB','A12':'GND','B12':'GND','S1':'GND'},B,mpn='GCT USB4125-GF-A',ref='J1')
_cnt['J']=1
R('5k1','USB_CC1','GND',B); R('5k1','USB_CC2','GND',B)
add('J','Connector','Screw_Terminal_01x02','5V EXT','TerminalBlock_Phoenix:TerminalBlock_Phoenix_PT-1,5-2-3.5-H_1x02_P3.50mm_Horizontal',
    {1:'VEXT',2:'GND'},B,mpn='Phoenix PT 1,5/2-3,5-H')
add('JP','Jumper','Jumper_3_Bridged12','SRC USB/EXT',FP_H3,{1:'VUSB',2:'VIN',3:'VEXT'},B,note='1-2 = USB-C, 2-3 = bench supply')
add('F','Device','Polyfuse_Small','1.5A hold','Fuse:Fuse_1812_4532Metric',{1:'VIN',2:'+5V'},B,mpn='PTC resettable 1.5A/6V 1812')
bypass('+5V',B,('22u','22u','100n'))
R('2k2','+5V','LED_PWR_A',B); LED('PWR green','LED_PWR_A','GND',B)
# rails with current-measurement jumpers
JP2('I_DEMOD','+5V','5V_DEMOD_J',B); FB('5V_DEMOD_J','5V_DEMOD',B); bypass('5V_DEMOD',B,('10u',))
JP2('I_LNA','+5V','5V_LNA_J',B); FB('5V_LNA_J','5V_LNA',B); bypass('5V_LNA',B,('10u',))
def ldo(rail,blkname):
    j=JP2('I_'+rail[4:],'+5V',rail+'_IN',B)
    add('U','Regulator_Linear','LP5907MFX-3.3','LP5907MFX-3.3','Package_TO_SOT_SMD:SOT-23-5',
        {1:rail+'_IN',2:'GND',3:rail+'_IN',4:NC,5:rail},B,mpn='TI LP5907MFX-3.3/NOPB')
    C('1u',rail+'_IN','GND',B,fp=FP_C6); C('1u',rail,'GND',B,fp=FP_C6)
for r in ('3V3_VCO','3V3_PLL','3V3_ADC','3V3_DIG'): ldo(r,B)
R('1k','3V3_DIG','LED_3V3_A',B); LED('3V3 green','LED_3V3_A','GND',B)

# ---------------------------------------------------------------- RF FRONT END
B='2 RF input, bias tee, LNA'
add('J','Connector','Conn_Coaxial','ANT (SMA)','Connector_Coaxial:SMA_Samtec_SMA-J-P-X-ST-EM1_EdgeMount',{1:'ANT',2:'GND'},B,mpn='Samtec SMA-J-P-H-ST-EM1')
# bias tee: +5V -> polyfuse -> P-FET -> 10R -> choke -> ANT
add('F','Device','Polyfuse_Small','100mA hold','Fuse:Fuse_0805_2012Metric',{1:'+5V',2:'BT_FUSED'},B,mpn='PTC resettable 100mA 0805')
add('Q','Transistor_FET','AO3401A','AO3401A','Package_TO_SOT_SMD:SOT-23',{1:'BT_GATE',2:'BT_FUSED',3:'BT_SW'},B,mpn='AOS AO3401A')
add('Q','Transistor_FET','2N7002','2N7002','Package_TO_SOT_SMD:SOT-23',{1:'BIAS_EN',2:'GND',3:'BT_GATE'},B,mpn='2N7002')
R('100k','BT_FUSED','BT_GATE',B); R('100k','BIAS_EN','GND',B)
R('10R','BT_SW','BT_FILT',B); C('100n','BT_FILT','GND',B); C('1n','BT_FILT','GND',B)
L('470nH','BT_FILT','ANT',B,fp=FP_L6,mpn='0603 wirewound, SRF > 600 MHz')
R('1k','BT_SW','LED_BT_A',B); LED('BIAS-T amber','LED_BT_A','GND',B)
C('1n','ANT','RF_A',B,note='DC block, C0G')
L('470nH','RF_A','GND',B,fp=FP_L6,note='static bleed')
add('D','Diode','BAV99','BAV99','Package_TO_SOT_SMD:SOT-23',{1:'GND',2:'GND',3:'RF_A'},B,mpn='BAV99',note='antiparallel clamp')
# break point 1
UFL('BP1 ANT side','RF_A',B); R('0R','RF_A','LNA_IN',B,note='BP1 link'); UFL('BP1 LNA side','LNA_IN',B)
C('1n','LNA_IN','LNA_G',B)
add('U','RF_Amplifier','PGA-103','PGA-103+','Package_TO_SOT_SMD:SOT-89-3',{1:'LNA_G',2:'GND',3:'LNA_D'},B,mpn='Mini-Circuits PGA-103+')
L('470nH','5V_LNA','LNA_D',B,fp=FP_L6,mpn='0603 wirewound, >= 150 mA'); bypass('5V_LNA',B,('100n','1n'))
C('1n','LNA_D','LNA_OUT',B)
UFL('BP2 LNA side','LNA_OUT',B); R('0R','LNA_OUT','FILT_IN',B,note='BP2 link'); UFL('BP2 filter side','FILT_IN',B)

B='3 Band filters'
def sw(rfc,rf1,rf2,blk,ctrl='BAND_SEL'):
    add('U','CSC_R1','PE4259','PE4259','Package_TO_SOT_SMD:SOT-363_SC-70-6',
        {1:rf1,2:'GND',3:rf2,4:ctrl,5:rfc,6:'3V3_DIG'},blk,mpn='pSemi PE4259-63')
    C('100n','3V3_DIG','GND',blk)
sw('FILT_IN','F70_IN','F2_IN',B)       # CTRL high -> RFC-RF1 (70 cm), low -> RFC-RF2 (2 m)
def ladder(pfx,nin,nout,hp,lp,blk):
    # HPF: Cs, Lp, Cs, Lp, Cs  then LPF: C, L, C, L, C
    n=[nin]+[f'{pfx}_{i}' for i in range(1,5)]+[f'{pfx}_M']
    C(hp[0],n[0],n[1],blk); L(hp[1],n[1],'GND',blk); C(hp[2],n[1],n[2],blk); L(hp[3],n[2],'GND',blk); C(hp[4],n[2],n[5],blk)
    m=[n[5],f'{pfx}_5',nout]
    C(lp[0],m[0],'GND',blk); L(lp[1],m[0],m[1],blk); C(lp[2],m[1],'GND',blk); L(lp[3],m[1],m[2],blk); C(lp[4],m[2],'GND',blk)
ladder('F2','F2_IN','F2_OUT',('47p','39nH','15p','39nH','47p'),('10p','68nH','33p','68nH','10p'),B)
ladder('F70','F70_IN','F70_OUT',('15p','15nH','4p7','15nH','15p'),('3p9','22nH','12p','22nH','3p9'),B)
sw('FILT_OUT','F2_OUT','F70_OUT',B,ctrl='BAND_SEL_N')   # mirrored pinout: driven by inverted control
add('Q','Transistor_FET','2N7002','2N7002','Package_TO_SOT_SMD:SOT-23',{1:'BAND_SEL',2:'GND',3:'BAND_SEL_N'},B,mpn='2N7002',note='inverter for output switch')
R('10k','BAND_SEL_N','3V3_DIG',B)
R('100k','BAND_SEL','GND',B)
UFL('BP3 filter side','FILT_OUT',B); R('0R','FILT_OUT','DEM_RF',B,note='BP3 link'); UFL('BP3 demod side','DEM_RF',B)

# ---------------------------------------------------------------- DEMODULATOR
B='4 I/Q demodulator'
C('DNP','DEM_RF','GND',B,dnp=True,note='match pad'); R('0R','DEM_RF','DEM_RF_M',B,note='match series pad'); C('DNP','DEM_RF_M','GND',B,dnp=True,note='match pad')
add('T','Transformer','TC1-1-13M+','TC1-1-13M+','Transformer_SMD:Transformer_MiniCircuits_AT224-1A',
    {6:'DEM_RF_M',4:'GND',1:'RF_BAL_P',3:'RF_BAL_N'},B,mpn='Mini-Circuits TC1-1-13M+')
L('15nH','RF_BAL_P','RF_BAL_N',B,dnp=True,note='L6 per LTC5584 data sheet: 15nH @450MHz, 68nH @140MHz; confirm position against data sheet Figure 4')
C('10n','RF_BAL_P','DEM_RFP',B); C('10n','RF_BAL_N','DEM_RFN',B)
add('U','CSC_R1','LTC5584','LTC5584IUF','Package_DFN_QFN:QFN-24-1EP_4x4mm_P0.5mm_EP2.6x2.6mm_ThermalVias',
    {1:'TRIM_IP2Q',2:'TRIM_DCOQ',3:'TRIM_DCOI',4:'TRIM_IP2I',5:'DEM_RFP',6:'DEM_RFN',7:'DEMOD_EN',8:'GND',9:'DEM_VBIAS',
     10:'5V_DEMOD',11:'DEMOD_EDC',12:'DEMOD_EIP2',13:'GND',14:'GND',15:'LO_P',16:'LO_N',17:'DEM_VCAP',18:'DEM_CMQ',19:'DEM_CMI',
     20:'BB_QN',21:'BB_QP',22:'BB_IN',23:'BB_IP',24:'DEM_REF',25:'GND'},B,mpn='Analog Devices LTC5584IUF#PBF')
bypass('5V_DEMOD',B,('1u','10n'))
R('294R','DEM_VBIAS','GND',B,dnp=True,note='optional: 294R cuts ICC to ~145 mA (lower IIP3)')
C('100n','DEM_CMQ','DEM_VCAP',B); C('100n','DEM_CMI','DEM_VCAP',B); C('100n','DEM_REF','GND',B)
for n in ('BB_IP','BB_IN','BB_QP','BB_QN'): R('100R','5V_DEMOD',n,B,note='output pull-up, 1%')
R('100k','DEMOD_EN','GND',B)
# trim DAC
add('U','Analog_DAC','MCP4728','MCP4728','Package_SO:MSOP-10_3x3mm_P0.5mm',
    {1:'3V3_DIG',2:'SCL',3:'SDA',4:'GND',5:NC,6:'DAC_A',7:'DAC_B',8:'DAC_C',9:'DAC_D',10:'GND'},B,mpn='Microchip MCP4728-E/UN',note='I2C 0x60')
bypass('3V3_DIG',B,('100n','10u'))
for d,t in (('DAC_A','TRIM_IP2I'),('DAC_B','TRIM_DCOI'),('DAC_C','TRIM_DCOQ'),('DAC_D','TRIM_IP2Q')):   # channel order chosen for crossing-free routing
    R('330R',d,t,B); R('330R',t,'GND',B); C('100n',t,'GND',B)

B='5 Baseband and ADC'
for s,adc in (('I','ADC_IN1'),('Q','ADC_IN2')):
    R('100R',f'BB_{s}P',f'BBF_{s}P',B); R('100R',f'BB_{s}N',f'BBF_{s}N',B)
    C('2n2',f'BBF_{s}P',f'BBF_{s}N',B,note='C0G; ~240 kHz corner')
    C('1u',f'BBF_{s}P',adc+'P',B,fp=FP_C6,note='X7R 25V'); C('1u',f'BBF_{s}N',adc+'M',B,fp=FP_C6,note='X7R 25V')
add('J','Connector','Conn_01x06_Pin','BB TEST','Connector_PinHeader_2.54mm:PinHeader_1x06_P2.54mm_Vertical',
    {1:'BBF_IP',2:'BBF_IN',3:'GND',4:'BBF_QP',5:'BBF_QN',6:'GND'},B)
add('U','CSC_R1','TLV320ADC3140','TLV320ADC3140','Package_DFN_QFN:Texas_RTW_WQFN-24-1EP_4x4mm_P0.5mm_EP2.7x2.7mm_ThermalVias',
    {1:'3V3_ADC',2:'ADC_AREG',3:'ADC_VREF',4:'GND',5:NC,6:'ADC_IN1P',7:'ADC_IN1M',8:'ADC_IN2P',9:'ADC_IN2M',10:NC,11:NC,12:NC,13:NC,
     14:'ADC_SHDNZ',15:'GND',16:'GND',17:'SCL',18:'SDA',19:'3V3_DIG',20:'ADC_MCLK',21:'ADC_SDOUT',22:'ADC_BCLK',23:'ADC_FSYNC',
     24:'ADC_DREG',25:'GND'},B,mpn='TI TLV320ADC3140IRTWR',note='I2C 0x4C')
bypass('3V3_ADC',B,('10u','100n')); bypass('ADC_AREG',B,('10u','100n')); bypass('ADC_VREF',B,('1u',))
bypass('ADC_DREG',B,('10u','100n')); bypass('3V3_DIG',B,('100n',))
R('100k','ADC_SHDNZ','GND',B)
R('33R','ADC_SDOUT','I2S_DOUT',B); R('33R','ADC_BCLK','I2S_BCLK',B); R('33R','ADC_FSYNC','I2S_FSYNC',B)
add('Y','Oscillator','ASE-xxxMHz','24.576MHz','Oscillator:Oscillator_SMD_Abracon_ASE-4Pin_3.2x2.5mm',
    {1:'3V3_MCLK',2:'GND',3:'MCLK_OSC_R',4:'3V3_MCLK'},B,mpn='Abracon ASE-24.576MHZ-LC-T (3.3 V CMOS)')
FB('3V3_DIG','3V3_MCLK',B); bypass('3V3_MCLK',B,('100n',)); R('33R','MCLK_OSC_R','MCLK_OSC',B)
add('JP','Jumper','Jumper_3_Bridged12','MCLK OSC/CTRL',FP_H3,{1:'MCLK_OSC',2:'ADC_MCLK',3:'XD6'},B,note='1-2 = on-board oscillator, 2-3 = controller D6')

# ---------------------------------------------------------------- SYNTHESIZER
B='6 Local oscillator'
add('U','Timer_PLL','ADF4351','ADF4351BCPZ','Package_CSP:LFCSP-32-1EP_5x5mm_P0.5mm_EP3.1x3.1mm_ThermalVias',
    {1:'PLL_CLK',2:'PLL_DATA',3:'PLL_LE',4:'PLL_CE',5:NC,6:'3V3_PLL',7:'PLL_CP',8:'GND',9:'GND',10:'3V3_PLL',11:'GND',
     12:'LOA_P',13:'LOA_N',14:'LOB_P',15:'LOB_N',16:'3V3_VCO',17:'3V3_VCO',18:'GND',19:'PLL_TEMP',20:'PLL_VTUNE',21:'GND',
     22:'PLL_RSET',23:'PLL_VCOM',24:'PLL_VREF',25:'PLL_LD',26:'PLL_PDBRF',27:'GND',28:'3V3_PLL',29:'PLL_REFIN',30:'PLL_MUXOUT_R',
     31:'GND',32:'3V3_PLL',33:'GND'},B,mpn='Analog Devices ADF4351BCPZ')
for i in range(4): bypass('3V3_PLL',B,('100n','10p'))
bypass('3V3_PLL',B,('10u',)); bypass('3V3_VCO',B,('100n','10p','100n','10p','10u'))
R('5k1','PLL_RSET','GND',B,note='1%'); C('100n','PLL_TEMP','GND',B); C('100n','PLL_VCOM','GND',B); C('100n','PLL_VREF','GND',B)
R('10k','PLL_CE','3V3_DIG',B); R('10k','PLL_PDBRF','3V3_DIG',B); TP('PDBRF','PLL_PDBRF',B)
# loop filter (verify with ADIsimPLL): Icp 2.5 mA, ~20 kHz
C('18n','PLL_CP','GND',B,note='C0G/X7R loop C1'); R('180R','PLL_CP','PLL_LF2',B); C('120n','PLL_LF2','GND',B,fp=FP_C6,note='loop C2')
R('390R','PLL_CP','PLL_VTUNE',B); C('1n5','PLL_VTUNE','GND',B,note='loop C3'); TP('VTUNE','PLL_VTUNE',B)
# reference
add('Y','Oscillator','TG2520SMN-xx.xxxxxxMhz-xxxxNM','25MHz TCXO','Oscillator:Oscillator_SMD_SeikoEpson_TG2520SMN-xxx-xxxxxx-4Pin_2.5x2.0mm',
    {1:NC,2:'GND',3:'TCXO_OUT',4:'3V3_TCXO'},B,mpn='25 MHz TCXO 2.5x2.0 mm, 3.3 V, <= 2 ppm (Epson TG2520SMN family or equiv.)')
FB('3V3_PLL','3V3_TCXO',B); bypass('3V3_TCXO',B,('100n',)); C('1n','TCXO_OUT','PLL_REFIN',B)
# LO to demodulator (differential), pull-ups to VVCO
R('51R','3V3_VCO','LOA_P',B,note='or 27 nH choke for more LO power'); R('51R','3V3_VCO','LOA_N',B,note='or 27 nH choke for more LO power')
C('10n','LOA_P','LO_N',B,note='polarity swapped on purpose: no crossing'); C('10n','LOA_N','LO_P',B)
# aux output = LO monitor
R('51R','3V3_VCO','LOB_P',B); R('51R','3V3_VCO','LOB_N',B)
C('1n','LOB_P','LO_MON',B); UFL('LO MON out','LO_MON',B)
R('33R','PLL_MUXOUT_R','PLL_MUXOUT',B)
R('2k2','PLL_LD','LED_LOCK_A',B); LED('LOCK green','LED_LOCK_A','GND',B); TP('LD','PLL_LD',B)

# ---------------------------------------------------------------- CONTROL
B='7 Controller and control'
add('J','Connector','Conn_01x07_Socket','XIAO left','Connector_PinSocket_2.54mm:PinSocket_1x07_P2.54mm_Vertical',
    {1:'XD0',2:'XD1',3:'XD2',4:'XD3',5:'XD4',6:'XD5',7:'XD6'},B,note='XIAO D0..D6, USB end at pin 1')
add('J','Connector','Conn_01x07_Socket','XIAO right','Connector_PinSocket_2.54mm:PinSocket_1x07_P2.54mm_Vertical',
    {1:'XIAO_5V',2:'GND',3:'XIAO_3V3',4:'XD10',5:'XD9',6:'XD8',7:'XD7'},B,note='XIAO 5V,GND,3V3,D10..D7, USB end at pin 1')
add('J','Connector_Generic','Conn_02x10_Odd_Even','CTRL','Connector_PinHeader_2.54mm:PinHeader_2x10_P2.54mm_Vertical',
    {1:'I2S_BCLK',2:'XD0',3:'I2S_FSYNC',4:'XD1',5:'I2S_DOUT',6:'XD2',7:'PLL_MUXOUT',8:'XD3',9:'SDA',10:'XD4',11:'SCL',12:'XD5',
     13:'PLL_CLK',14:'XD8',15:'PLL_LE',16:'XD9',17:'PLL_DATA',18:'XD10',19:'GND',20:'GND'},B,
    note='odd = kit signal, even = XIAO pin; fit 9 shunts for XIAO, or wire an external controller to the odd pins')
add('J','Connector','Conn_01x06_Pin','AUX','Connector_PinHeader_2.54mm:PinHeader_1x06_P2.54mm_Vertical',
    {1:'XD6',2:'XD7',3:'XIAO_3V3',4:'3V3_DIG',5:'+5V',6:'GND'},B)
JP2('XIAO 5V','XIAO_5V_D','XIAO_5V',B)
add('D','Device','D_Schottky_Small','B5819W','Diode_SMD:D_SOD-123',{2:'+5V',1:'XIAO_5V_D'},B,mpn='B5819W / MBR0520',note='kit 5V feeds XIAO, no back-feed')
R('4k7','SDA','3V3_DIG',B); R('4k7','SCL','3V3_DIG',B)
add('U','Interface_Expansion','TCA9534','TCA9534PWR','Package_SO:TSSOP-16_4.4x5mm_P0.65mm',
    {1:'GND',2:'GND',3:'GND',4:'BAND_SEL',5:'BIAS_EN',6:'DEMOD_EN',7:'DEMOD_EDC',8:'GND',9:'DEMOD_EIP2',10:'ADC_SHDNZ',11:'PLL_CE',
     12:'LED_USER_K',13:'EXP_INT',14:'SCL',15:'SDA',16:'3V3_DIG'},B,mpn='TI TCA9534PWR',note='I2C 0x20')
bypass('3V3_DIG',B,('100n',)); R('10k','EXP_INT','3V3_DIG',B); TP('INT','EXP_INT',B)
R('1k','3V3_DIG','LED_USER_A',B); LED('USER red','LED_USER_A','LED_USER_K',B)
for i in range(3): add('TP','Connector','TestPoint','GND','TestPoint:TestPoint_Loop_D1.80mm_Drill1.0mm_Beaded',{1:'GND'},B)
for i in range(4): add('H','Mechanical','MountingHole_Pad','M3','MountingHole:MountingHole_3.2mm_M3_Pad_Via',{1:'GND'},B)

def nets():
    n=defaultdict(list)
    for p in parts.values():
        for pin,net in p['pins'].items():
            if net!=NC: n[net].append((p['ref'],pin))
    return n
if __name__=='__main__':
    n=nets(); print(len(parts),'parts',len(n),'nets')
    for k,v in sorted(n.items()):
        if len(v)<2: print('SINGLE-PIN NET',k,v)
