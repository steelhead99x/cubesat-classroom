import pcbnew, sys, boardcfg
design=boardcfg.design()
FPDIR='/usr/share/kicad/footprints'
def load(fp):
    lib,name=fp.split(':')
    d=boardcfg.KICAD if lib=='CSC_R1' else FPDIR
    return pcbnew.FootprintLoad(f'{d}/{lib}.pretty',name)
bad=0; seen={}
for r,p in design.parts.items():
    f=load(p['fp'])
    if f is None: print('NO FOOTPRINT',r,p['fp']); bad+=1; continue
    pads=set(pd.GetNumber() for pd in f.Pads() if pd.GetNumber()!='')
    pins=set(p['pins'])
    if pads!=pins and p['fp'] not in seen:
        print(r,p['fp'].split(':')[1],'pads-not-in-design:',sorted(pads-pins),'pins-without-pad:',sorted(pins-pads)); 
        if pins-pads: bad+=1
    seen[p['fp']]=1
print('problems:',bad)
if bad: sys.exit(1)
if len(sys.argv)>1:
    for name in ['Connector_Coaxial:SMA_Samtec_SMA-J-P-X-ST-EM1_EdgeMount','Connector_Coaxial:U.FL_Hirose_U.FL-R-SMT-1_Vertical','Package_TO_SOT_SMD:SOT-89-3','Package_TO_SOT_SMD:SOT-363_SC-70-6','Transformer_SMD:Transformer_MiniCircuits_AT224-1A','Connector_USB:USB_C_Receptacle_GCT_USB4125-xx-x_6P_TopMnt_Horizontal','Package_DFN_QFN:QFN-24-1EP_4x4mm_P0.5mm_EP2.6x2.6mm_ThermalVias']:
        f=load(name); print('==',name.split(':')[1])
        for pd in f.Pads():
            if pd.GetNumber() in('','25') : continue
            print('   pad',pd.GetNumber(),'at',pcbnew.ToMM(pd.GetPosition().x),pcbnew.ToMM(pd.GetPosition().y),'size',pcbnew.ToMM(pd.GetSize().x),pcbnew.ToMM(pd.GetSize().y),'layers',pd.GetLayerSet().FmtHex()[-9:] if False else ('THT' if pd.GetAttribute()==pcbnew.PAD_ATTRIB_PTH else 'SMD' if pd.GetAttribute()==pcbnew.PAD_ATTRIB_SMD else 'NPTH'), 'layer', 'B' if pd.IsOnLayer(pcbnew.B_Cu) and not pd.IsOnLayer(pcbnew.F_Cu) else 'F')
        bb=f.GetBoundingBox(False,False); print('   bbox',pcbnew.ToMM(bb.GetLeft()),pcbnew.ToMM(bb.GetTop()),pcbnew.ToMM(bb.GetRight()),pcbnew.ToMM(bb.GetBottom()))
