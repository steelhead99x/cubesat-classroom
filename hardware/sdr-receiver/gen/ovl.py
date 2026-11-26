"""quick courtyard overlap check (bounding boxes) + list"""
import pcbnew,sys
b=pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb'); OX,OY=60,50
rs=[]
for f in b.GetFootprints():
    bb=None
    for l in (pcbnew.F_CrtYd,pcbnew.B_CrtYd):
        c=f.GetCourtyard(l)
        if c.OutlineCount():
            bb=c.BBox(); break
    if bb is None: bb=f.GetBoundingBox(False,False)
    T=pcbnew.ToMM
    rs.append((f.GetReference(),T(bb.GetLeft())-OX,T(bb.GetTop())-OY,T(bb.GetRight())-OX,T(bb.GetBottom())-OY))
n=0
for i,a in enumerate(rs):
    for c in rs[i+1:]:
        ox=min(a[3],c[3])-max(a[1],c[1]); oy=min(a[4],c[4])-max(a[2],c[2])
        if ox>0.005 and oy>0.005:
            n+=1; print(f"{a[0]}({(a[1]+a[3])/2:.1f},{(a[2]+a[4])/2:.1f}) x {c[0]}({(c[1]+c[3])/2:.1f},{(c[2]+c[4])/2:.1f}) overlap {ox:.2f}x{oy:.2f}")
print('overlaps:',n)
if len(sys.argv)>1:
    for r in rs:
        if r[0] in sys.argv[1:]: print(r[0],'w',round(r[3]-r[1],2),'h',round(r[4]-r[2],2))
