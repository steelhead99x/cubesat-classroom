"""Add ground pours on all four copper layers, fill, add board text, save."""
import pcbnew
import boardcfg
OUT=boardcfg.KICAD; PROJ=boardcfg.PROJ; OX,OY=60.0,50.0; mm=pcbnew.FromMM
b=pcbnew.LoadBoard(f'{OUT}/{PROJ}.kicad_pcb')
bb=b.GetBoardEdgesBoundingBox(); T=pcbnew.ToMM; W=round(T(bb.GetWidth()),1)-0.1; H=round(T(bb.GetHeight()),1)-0.1
gnd=b.FindNet('GND').GetNetCode()
import sys
if len(list(b.Zones())): sys.exit('board already has pours: run finish.py once, on the routed board from iterate.sh')
for layer in (pcbnew.F_Cu,pcbnew.In1_Cu,pcbnew.In2_Cu,pcbnew.B_Cu):
    z=pcbnew.ZONE(b); z.SetLayer(layer); z.SetNetCode(gnd); z.SetZoneName('GND_'+pcbnew.LayerName(layer))
    o=z.Outline(); o.NewOutline()
    for x,y in ((0,0),(W,0),(W,H),(0,H)): o.Append(mm(OX+x),mm(OY+y))
    z.SetLocalClearance(mm(0.25)); z.SetMinThickness(mm(0.2)); z.SetPadConnection(pcbnew.ZONE_CONNECTION_FULL)
    z.SetIslandRemovalMode(pcbnew.ISLAND_REMOVAL_MODE_ALWAYS); z.SetThermalReliefGap(mm(0.25)); z.SetThermalReliefSpokeWidth(mm(0.4))
    b.Add(z)
def text(s,x,y,size=1.2,layer=pcbnew.F_SilkS):
    t=pcbnew.PCB_TEXT(b); t.SetText(s); t.SetPosition(pcbnew.VECTOR2I(mm(OX+x),mm(OY+y))); t.SetLayer(layer)
    t.SetTextSize(pcbnew.VECTOR2I(mm(size),mm(size))); t.SetTextThickness(mm(size*0.15)); t.SetHorizJustify(pcbnew.GR_TEXT_H_ALIGN_LEFT); b.Add(t)
if PROJ=='csc-r1':
    text('CSC-R1 dev kit  Rev A',10,8,2.0); text('receive-only SDR, 2 m / 70 cm',10,11.5,1.3); text('cubesat-classroom   UNTESTED PROTOTYPE',10,14,1.1)
    text('ANT',1.5,46.5,1.0); text('5V USB-C',8.5,66.5,1.0); text('5V EXT + -',23,69.5,1.0)
else:
    for s_,x_,y_,size_ in boardcfg.design().SILK: text(s_,x_,y_,size_)
pcbnew.ZONE_FILLER(b).Fill(b.Zones())
pcbnew.SaveBoard(f'{OUT}/{PROJ}.kicad_pcb',b)
print('zones',len(b.Zones()),'tracks+vias',len(b.GetTracks()))
# the router can drop two vias of one net on the same spot; keep one (done on the saved text)
import re
txt=open(f'{OUT}/{PROJ}.kicad_pcb').read().split('\n'); seen=set(); out=[]; dup=0
for line in txt:
    if line.lstrip().startswith('(via ('):
        k=re.sub(r'\(tstamp [^)]*\)','',line).strip()
        if k in seen: dup+=1; continue
        seen.add(k)
    out.append(line)
open(f'{OUT}/{PROJ}.kicad_pcb','w').write('\n'.join(out)); print('duplicate vias removed',dup)
