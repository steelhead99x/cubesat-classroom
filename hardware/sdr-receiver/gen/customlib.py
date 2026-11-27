"""Generate project symbol library CSC_R1.kicad_sym from pin tables taken from manufacturer data sheets."""
from sexp import Sym, dump
S=Sym
def mk(name,ref,fp,ds,left,right,descr):
    n=max(len(left),len(right)); h=(n+1)*2.54; w=25.4
    top=h/2
    def pin(num,nm,typ,x,y,ang):
        return [S('pin'),S(typ),S('line'),[S('at'),x,y,ang],[S('length'),2.54],
                [S('name'),nm,[S('effects'),[S('font'),[S('size'),1.27,1.27]]]],
                [S('number'),str(num),[S('effects'),[S('font'),[S('size'),1.27,1.27]]]]]
    body=[S('symbol'),f'{name}_0_1',[S('rectangle'),[S('start'),-w/2,top],[S('end'),w/2,-top],
          [S('stroke'),[S('width'),0.254],[S('type'),S('default')]],[S('fill'),[S('type'),S('background')]]]]
    ps=[S('symbol'),f'{name}_1_1']
    for i,(num,nm,typ) in enumerate(left): ps.append(pin(num,nm,typ,-w/2-2.54,top-2.54*(i+1),0))
    for i,(num,nm,typ) in enumerate(right): ps.append(pin(num,nm,typ,w/2+2.54,top-2.54*(i+1),180))
    def prop(k,v,y,hide=False):
        e=[S('effects'),[S('font'),[S('size'),1.27,1.27]]]
        if hide: e.append(S('hide'))
        return [S('property'),k,v,[S('at'),0,y,0],e]
    return [S('symbol'),name,[S('in_bom'),S('yes')],[S('on_board'),S('yes')],
            prop('Reference',ref,top+2.54),prop('Value',name,top+5.08),prop('Footprint',fp,0,True),prop('Datasheet',ds,0,True),
            prop('ki_description',descr,0,True),body,ps]
P='passive'; I='input'; O='output'; W='power_in'; WO='power_out'; B='bidirectional'
syms=[
 mk('LTC5584','U','Package_DFN_QFN:QFN-24-1EP_4x4mm_P0.5mm_EP2.6x2.6mm','https://www.analog.com/media/en/technical-documentation/data-sheets/5584f.pdf',
    [(5,'RF+',I),(6,'RF-',I),(15,'LO+',I),(16,'LO-',I),(7,'EN',I),(11,'EDC',I),(12,'EIP2',I),(3,'DCOI',I),(2,'DCOQ',I),(4,'IP2I',I),(1,'IP2Q',I),(24,'REF',P),(9,'VBIAS',P)],
    [(10,'VCC',W),(23,'I+',O),(22,'I-',O),(21,'Q+',O),(20,'Q-',O),(19,'CMI',P),(18,'CMQ',P),(17,'VCAP',P),(8,'GND',W),(13,'GND',W),(14,'GND',W),(25,'GND(EP)',W)],
    '30MHz to 1.4GHz IQ demodulator, QFN-24 4x4'),
 mk('TLV320ADC3140','U','Package_DFN_QFN:Texas_RTW_WQFN-24-1EP_4x4mm_P0.5mm_EP2.7x2.7mm','https://www.ti.com/lit/gpn/tlv320adc3140',
    [(6,'IN1P_GPI1',I),(7,'IN1M_GPO1',I),(8,'IN2P_GPI2',I),(9,'IN2M_GPO2',I),(10,'IN3P_GPI3',I),(11,'IN3M_GPO3',I),(12,'IN4P_GPI4',I),(13,'IN4M_GPO4',I),
     (5,'MICBIAS',O),(3,'VREF',P),(2,'AREG',P),(24,'DREG',P),(14,'SHDNZ',I)],
    [(1,'AVDD',W),(19,'IOVDD',W),(17,'SCL_MOSI',I),(18,'SDA_SSZ',B),(16,'ADDR0_SCLK',I),(15,'ADDR1_MISO',B),(20,'GPIO1',B),(21,'SDOUT',O),(22,'BCLK',B),(23,'FSYNC',B),(4,'AVSS',W),(25,'VSS(EP)',W)],
    '4-channel 768 kHz audio ADC, WQFN-24 4x4'),
 mk('PE4259','U','Package_TO_SOT_SMD:SOT-363_SC-70-6','https://www.psemi.com/products/rf-switches/pe4259',
    [(5,'RFC',P),(4,'CTRL',I),(6,'VDD',W)],[(1,'RF1',P),(3,'RF2',P),(2,'GND',W)],'SPDT RF switch 10-3000 MHz, SC-70-6. CTRL high: RFC-RF1, low: RFC-RF2'),
]
def write(path):
    open(path,'w').write(dump([S('kicad_symbol_lib'),[S('version'),20220914],[S('generator'),S('csc_r1_gen')]]+syms)+'\n')
if __name__=='__main__':
    import boardcfg; write(f'{boardcfg.KICAD}/CSC_R1.kicad_sym')
