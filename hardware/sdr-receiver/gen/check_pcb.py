"""Independent parity check: board pads vs the schematic netlist exported by KiCad (not vs design.py)."""
import sys, pcbnew; sys.setrecursionlimit(100000)
from sexp import parse, find, first
t=parse(open('../kicad/rt.net').read())
sch={}
for n in find(first(t,'nets'),'net'):
    name=first(n,'name')[1].lstrip('/')
    for x in find(n,'node'): sch[(first(x,'ref')[1],first(x,'pin')[1])]=name
comps={first(c,'ref')[1]:first(c,'footprint')[1] for c in find(first(t,'components'),'comp')}
b=pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb'); bad=0; seen=set()
for f in b.GetFootprints():
    r=f.GetReference(); seen.add(r)
    fp=str(f.GetFPID().GetLibNickname())+':'+str(f.GetFPID().GetLibItemName())
    if comps.get(r)!=fp: bad+=1; print('footprint differs',r,comps.get(r),fp)
    for p in f.Pads():
        k=(r,p.GetNumber()); s=sch.get(k)
        s=None if (s is None or s.startswith('unconnected')) else s
        pn=p.GetNetname() or None
        if p.GetNumber()=='' : continue
        if s!=pn: bad+=1; print('net differs',k,'sch',s,'pcb',pn)
for r in comps:
    if r not in seen: bad+=1; print('missing on board',r)
trk=[t_ for t_ in b.GetTracks()]
print('footprints',len(seen),'schematic parts',len(comps),'pad/net mismatches',bad)
print('tracks',sum(1 for x in trk if x.GetClass()=='PCB_TRACK'),'vias',sum(1 for x in trk if x.GetClass()=='PCB_VIA'),'min track mm',min(pcbnew.ToMM(x.GetWidth()) for x in trk if x.GetClass()=='PCB_TRACK'))
on={}
for x in trk:
    if x.GetClass()=='PCB_TRACK': on[b.GetLayerName(x.GetLayer())]=on.get(b.GetLayerName(x.GetLayer()),0)+1
print('tracks per layer',on)
