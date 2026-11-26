"""Small grid maze router (A*, 45-degree moves, through vias) written for this board.
Routes every net except GND and the hand-routed RF nets on F.Cu, In2.Cu and B.Cu. In1.Cu is kept as an unbroken ground plane."""
import pcbnew, math, heapq, sys, time, numpy as np
OUT='../kicad'; PROJ='csc-r1'; OX,OY=60.0,50.0
G=0.1; CLR=0.16; VIA_D=0.6; VIA_DRL=0.3; EDGE=0.5
b=pcbnew.LoadBoard(f'{OUT}/base.kicad_pcb'); T=pcbnew.ToMM; mm=pcbnew.FromMM
bb=b.GetBoardEdgesBoundingBox(); W=T(bb.GetWidth()); H=T(bb.GetHeight())
NX=int(round(W/G))+1; NY=int(round(H/G))+1
LAYERS=[pcbnew.F_Cu,pcbnew.In1_Cu,pcbnew.In2_Cu,pcbnew.B_Cu]; ROUTE=[0,2,3]; NL=4
WIDTHS={'sig':0.2,'pwr':0.3,'p5':0.5}
P5={'+5V','VIN','VUSB','VEXT'}
def cls(net,ncname): return 'p5' if net in P5 else 'pwr' if ncname=='Power' else 'sig'
KEYS=['sig','pwr','p5','via']; RAD={k:WIDTHS[k]/2+CLR for k in WIDTHS}; RAD['via']=VIA_D/2+CLR
own={k:np.zeros((NL,NY,NX),dtype=np.int32) for k in KEYS}
novia=np.zeros((NY,NX),dtype=bool)
HOLE_GAP=0.3
def rast_hole(x,y,d):
    R=d/2+HOLE_GAP+VIA_DRL/2; y0,y1,x0,x1=win(x,y,R+0.1); novia[y0:y1,x0:x1]|=np.hypot(xs[y0:y1,x0:x1]*G-x,ys[y0:y1,x0:x1]*G-y)<R
ys,xs=np.mgrid[0:NY,0:NX]; 
def mark(arr,mask,net,win):
    y0,y1,x0,x1=win; a=arr[y0:y1,x0:x1]
    a[mask&(a==0)]=net; a[mask&(a!=0)&(a!=net)]=-1
def win(x,y,r):
    return max(0,int((y-r)/G)),min(NY,int((y+r)/G)+2),max(0,int((x-r)/G)),min(NX,int((x+r)/G)+2)
def rast_rect(cx,cy,w,h,ang,net,layers,extra_novia=None):
    R=math.hypot(w,h)/2+RAD['via']+0.1; y0,y1,x0,x1=win(cx,cy,R)
    X=xs[y0:y1,x0:x1]*G-cx; Y=ys[y0:y1,x0:x1]*G-cy; c,s=math.cos(math.radians(ang)),math.sin(math.radians(ang))
    u=X*c-Y*s; v=X*s+Y*c
    dx=np.maximum(np.abs(u)-w/2,0); dy=np.maximum(np.abs(v)-h/2,0); d=np.hypot(dx,dy)
    for k in KEYS:
        m=d<RAD[k]
        for l in layers: mark(own[k][l],m,net,(y0,y1,x0,x1))
    if extra_novia is not None: novia[y0:y1,x0:x1]|=(d<extra_novia)
def rast_seg(ax,ay,bx,by,w,net,layer):
    R=w/2+RAD['via']+0.1; y0,y1,x0,x1=max(0,int((min(ay,by)-R)/G)),min(NY,int((max(ay,by)+R)/G)+2),max(0,int((min(ax,bx)-R)/G)),min(NX,int((max(ax,bx)+R)/G)+2)
    X=xs[y0:y1,x0:x1]*G; Y=ys[y0:y1,x0:x1]*G; vx,vy=bx-ax,by-ay; L2=vx*vx+vy*vy
    t=np.clip(((X-ax)*vx+(Y-ay)*vy)/L2,0,1) if L2>0 else 0
    d=np.hypot(X-(ax+t*vx),Y-(ay+t*vy))
    for k in KEYS: mark(own[k][layer],d<w/2+RAD[k],net,(y0,y1,x0,x1))
def rast_via(x,y,net):
    R=VIA_D/2+RAD['via']+0.1; y0,y1,x0,x1=win(x,y,R); d=np.hypot(xs[y0:y1,x0:x1]*G-x,ys[y0:y1,x0:x1]*G-y)
    for k in KEYS:
        for l in range(NL): mark(own[k][l],d<VIA_D/2+RAD[k],net,(y0,y1,x0,x1))
    rast_hole(x,y,VIA_DRL)
# ---- obstacles: board edge, pads, existing tracks
for k in KEYS:
    e=int(round((EDGE+ (WIDTHS[k]/2 if k!='via' else VIA_D/2))/G))
    for l in range(NL):
        own[k][l][:e,:]=-1; own[k][l][-e:,:]=-1; own[k][l][:,:e]=-1; own[k][l][:,-e:]=-1
pads_by_net={}
for f in b.GetFootprints():
    for p in f.Pads():
        pos=p.GetPosition(); x,y=T(pos.x)-OX,T(pos.y)-OY; sz=p.GetSize(); w,h=T(sz.x),T(sz.y)
        ang=p.GetOrientationDegrees(); net=p.GetNetCode() or -1
        tht=p.GetAttribute() in (pcbnew.PAD_ATTRIB_PTH,pcbnew.PAD_ATTRIB_NPTH)
        if tht: lay=[0,1,2,3]
        else: lay=[0] if p.IsOnLayer(pcbnew.F_Cu) else []
        if p.IsOnLayer(pcbnew.B_Cu) and not tht: lay.append(3)
        if not lay: continue
        if tht:
            d=p.GetDrillSize(); w=max(w,T(d.x)+0.3); h=max(h,T(d.y)+0.3); rast_hole(x,y,max(T(d.x),T(d.y)))
        oc=max(0.0,T(p.GetOwnClearance(pcbnew.F_Cu))-0.15); w+=2*oc; h+=2*oc
        rast_rect(x,y,w,h,-ang,net,lay,extra_novia=0.25 if not tht else 0.0)
        if p.GetNetCode(): pads_by_net.setdefault(p.GetNetname(),[]).append(dict(x=x,y=y,w=w,h=h,ang=-ang,layers=[l for l in lay if l in ROUTE],ref=f.GetReference(),num=p.GetNumber(),tht=tht))
for t in b.GetTracks():
    if t.GetClass()=='PCB_TRACK':
        rast_seg(T(t.GetStart().x)-OX,T(t.GetStart().y)-OY,T(t.GetEnd().x)-OX,T(t.GetEnd().y)-OY,T(t.GetWidth()),t.GetNetCode(),LAYERS.index(t.GetLayer()))
# keep-out rule areas that come with footprints (U.FL, TCXO ...)
for f in b.GetFootprints():
    for z in f.Zones():
        if z.GetIsRuleArea() and (z.GetDoNotAllowTracks() or z.GetDoNotAllowVias()):
            zb=z.GetBoundingBox(); x0=T(zb.GetLeft())-OX; y0=T(zb.GetTop())-OY; x1=T(zb.GetRight())-OX; y1=T(zb.GetBottom())-OY; poly=z.Outline()
            for k in KEYS:
                rr=RAD[k]-CLR
                for iy in range(max(0,int((y0-rr)/G)),int((y1+rr)/G)+2):
                    for ix in range(max(0,int((x0-rr)/G)),int((x1+rr)/G)+2):
                        if poly.Contains(pcbnew.VECTOR2I(mm(OX+ix*G),mm(OY+iy*G)),-1,mm(rr)):
                            for li,L in enumerate(LAYERS):
                                if z.IsOnLayer(L): own[k][li][iy,ix]=-1
routed_nets={t.GetNetname() for t in b.GetTracks()}
gcells=set()
def astar_local(src,tflag,pb,centre,radius):
    """Dijkstra on the top layer inside a window; returns list of cell ids (layer 0)"""
    cx,cy=centre; g={}; par={}; pq=[]
    for n in src: g[n]=0.0; heapq.heappush(pq,(0.0,n))
    moves=((1,0,1.0),(-1,0,1.0),(0,1,1.0),(0,-1,1.0),(1,1,SQ2),(1,-1,SQ2),(-1,1,SQ2),(-1,-1,SQ2))
    while pq:
        f,n=heapq.heappop(pq)
        if f>g[n]+1e-9: continue
        if tflag[n]:
            path=[n]
            while n in par: n=par[n]; path.append(n)
            return path[::-1]
        y=n//NX; x=n-y*NX
        for dx,dy,c in moves:
            xx=x+dx; yy=y+dy
            if abs(xx-cx)>radius or abs(yy-cy)>radius or xx<0 or yy<0 or xx>=NX or yy>=NY: continue
            m=yy*NX+xx
            if not pb[m]: continue
            if dx and dy and not (pb[y*NX+xx] and pb[yy*NX+x]): continue
            ng=f+c
            if ng<g.get(m,1e18): g[m]=ng; par[m]=n; heapq.heappush(pq,(ng,m))
    return None
new_tracks=[]; new_vias=[]; failed=[]
# ---- escape stubs for fine-pitch IC pads (reserve each pin's exit before any routing), GND pins tied to the exposed pad
def free(x,y,code,k='sig'):
    ix=int(round(x/G)); iy=int(round(y/G)); v=own[k][0][iy,ix]; return v==0 or v==code
for f in b.GetFootprints():
    ref=f.GetReference()
    if not ref.startswith('U') or f.GetPadCount()<6: continue
    fc=f.GetPosition(); fx,fy=T(fc.x)-OX,T(fc.y)-OY
    for p in f.Pads():
        if p.GetAttribute()!=pcbnew.PAD_ATTRIB_SMD or not p.GetNetCode(): continue
        sz=p.GetSize(); w,h=T(sz.x),T(sz.y)
        if min(w,h)>0.45 or max(w,h)>1.2: continue
        net=p.GetNetname(); code=p.GetNetCode(); px,py=T(p.GetPosition().x)-OX,T(p.GetPosition().y)-OY
        a=math.radians(-p.GetOrientationDegrees()); ux,uy=(math.cos(a),-math.sin(a)) if w>=h else (math.sin(a),math.cos(a))
        if ux*(px-fx)+uy*(py-fy)<0: ux,uy=-ux,-uy
        half=max(w,h)/2
        if net=='GND':
            if ref in('U8','U10','U11'):
                ex,ey=px-ux*(half+0.55),py-uy*(half+0.55)
                new_tracks.append((code,0,(px,py),(ex,ey),0.2)); rast_seg(px,py,ex,ey,0.2,code,0)
            continue
        if net in routed_nets: continue
        best=0
        for L in (0.7,0.6,0.5,0.4):
            if all(free(px+ux*(half+s),py+uy*(half+s),code) for s in np.arange(0.36,L+0.11,0.05)): best=L; break
        if not best: continue
        ex,ey=px+ux*(half+best),py+uy*(half+best)
        new_tracks.append((code,0,(px,py),(ex,ey),0.2)); rast_seg(px,py,ex,ey,0.2,code,0)
        for d in pads_by_net[net]:
            if d['ref']==ref and d['num']==p.GetNumber(): d.update(x=ex,y=ey,w=0.24,h=0.24,ang=0)
PARTIAL={'3V3_VCO','3V3_PLL','PLL_TEMP','PLL_VTUNE','PLL_RSET','PLL_VCOM','PLL_VREF','PLL_REFIN','BBF_IP','BBF_IN','BBF_QP','BBF_QN','TRIM_IP2Q','TRIM_DCOQ','TRIM_DCOI','TRIM_IP2I'}
SKIP=({'GND'}|routed_nets)-PARTIAL
NN=NX*NY
# existing (hand-drawn) tracks per net: used to seed the routing tree of partly routed nets
hand={}
for t in b.GetTracks():
    if t.GetClass()=='PCB_TRACK' and t.GetLayer()==pcbnew.F_Cu:
        hand.setdefault(t.GetNetname(),[]).append(((T(t.GetStart().x)-OX,T(t.GetStart().y)-OY),(T(t.GetEnd().x)-OX,T(t.GetEnd().y)-OY)))
def inpad(p,pt):
    X=pt[0]-p['x']; Y=pt[1]-p['y']; c,s=math.cos(math.radians(p['ang'])),math.sin(math.radians(p['ang']))
    return abs(X*c-Y*s)<=p['w']/2+0.01 and abs(X*s+Y*c)<=p['h']/2+0.01
def groups(name,pads):
    """union pads and hand tracks into connected groups; returns list of (pad list, track cell list)"""
    segs=hand.get(name,[]); n=len(pads)+len(segs); par=list(range(n))
    def fd(i):
        while par[i]!=i: par[i]=par[par[i]]; i=par[i]
        return i
    def un(i,j): par[fd(i)]=fd(j)
    for si,(a,c) in enumerate(segs):
        for pi,p in enumerate(pads):
            if 0 in p['layers'] and (inpad(p,a) or inpad(p,c)): un(len(pads)+si,pi)
        for sj,(a2,c2) in enumerate(segs[:si]):
            if any(math.hypot(u[0]-v[0],u[1]-v[1])<0.02 for u in (a,c) for v in (a2,c2)): un(len(pads)+si,len(pads)+sj)
            else:
                for u,(v,w_) in ((a,(a2,c2)),(c,(a2,c2)),(a2,(a,c)),(c2,(a,c))):
                    vx,vy=w_[0]-v[0],w_[1]-v[1]; L2=vx*vx+vy*vy
                    if L2>0:
                        t_=max(0,min(1,((u[0]-v[0])*vx+(u[1]-v[1])*vy)/L2))
                        if math.hypot(u[0]-(v[0]+t_*vx),u[1]-(v[1]+t_*vy))<0.03: un(len(pads)+si,len(pads)+sj)
    out={}
    for pi,p in enumerate(pads): out.setdefault(fd(pi),([],[]))[0].append(p)
    for si,(a,c) in enumerate(segs):
        L=math.hypot(c[0]-a[0],c[1]-a[1]); cells=set()
        for k_ in range(int(L/0.05)+1):
            x=a[0]+(c[0]-a[0])*k_*0.05/max(L,1e-9); y=a[1]+(c[1]-a[1])*k_*0.05/max(L,1e-9); cells.add(int(round(y/G))*NX+int(round(x/G)))
        out.setdefault(fd(len(pads)+si),([],[]))[1].extend(cells)
    return list(out.values())
def pad_cells(p,shrink=0.03):
    R=math.hypot(p['w'],p['h'])/2; y0,y1,x0,x1=win(p['x'],p['y'],R)
    X=xs[y0:y1,x0:x1]*G-p['x']; Y=ys[y0:y1,x0:x1]*G-p['y']; c,s=math.cos(math.radians(p['ang'])),math.sin(math.radians(p['ang']))
    u=X*c-Y*s; v=X*s+Y*c; m=(np.abs(u)<=max(p['w']/2-shrink,0.02))&(np.abs(v)<=max(p['h']/2-shrink,0.02))
    iy,ix=np.nonzero(m); return [(int(a)+y0)*NX+int(c_)+x0 for a,c_ in zip(iy,ix)]
SQ2=1.41421356
VIA_COST=30.0; LCOST={0:1.0,2:1.15,3:1.05}
def astar(src,tgt_flag,passb,viaok,tb):
    """src: list of node ids; tgt_flag: bytearray over NL*NN; passb: dict layer->bytes; returns node path"""
    tx0,ty0,tx1,ty1=tb
    def hf(n):
        r=n%NN; y=r//NX; x=r-y*NX
        dx=tx0-x if x<tx0 else x-tx1 if x>tx1 else 0; dy=ty0-y if y<ty0 else y-ty1 if y>ty1 else 0
        return (dx+dy)-0.5858*min(dx,dy)
    g={}; par={}; pq=[]
    for n in src: g[n]=0.0; heapq.heappush(pq,(hf(n),n))
    moves=((1,0,1.0),(-1,0,1.0),(0,1,1.0),(0,-1,1.0),(1,1,SQ2),(1,-1,SQ2),(-1,1,SQ2),(-1,-1,SQ2))
    pops=0
    while pq:
        f,n=heapq.heappop(pq); gn=g[n]
        if f>gn+hf(n)+1e-6: continue
        if tgt_flag[n]:
            path=[n]
            while n in par: n=par[n]; path.append(n)
            return path[::-1]
        pops+=1
        if pops>450000: return None
        l=n//NN; r=n-l*NN; y=r//NX; x=r-y*NX; pb=passb[l]; lc=LCOST[l]
        for dx,dy,c in moves:
            xx=x+dx; yy=y+dy
            if xx<0 or yy<0 or xx>=NX or yy>=NY: continue
            r2=yy*NX+xx
            if not pb[r2]: continue
            if dx and dy and not (pb[y*NX+xx] and pb[yy*NX+x]): continue
            m=l*NN+r2; ng=gn+c*lc
            if ng<g.get(m,1e18): g[m]=ng; par[m]=n; heapq.heappush(pq,(ng+hf(m),m))
        if viaok[r]:
            for l2 in ROUTE:
                if l2!=l and passb[l2][r]:
                    m=l2*NN+r; ng=gn+VIA_COST
                    if ng<g.get(m,1e18): g[m]=ng; par[m]=n; heapq.heappush(pq,(ng+hf(m),m))
    return None
def commit(path,net,code,w,first_pad,last_pad=None):
    isufl=lambda p_: p_ is not None and (not p_['tht']) and p_['ref'].startswith('J') and abs(max(p_['w'],p_['h'])-2.2)<0.2
    if isufl(first_pad): first_pad=None
    if isufl(last_pad): last_pad=None
    pts=[]
    for n in path:
        l=n//NN; r=n-l*NN; y=r//NX; x=r-y*NX; pts.append((l,x*G,y*G))
    segs=[]
    def addseg(l,a,bp):
        if abs(a[0]-bp[0])<1e-6 and abs(a[1]-bp[1])<1e-6: return
        new_tracks.append((code,l,a,bp,w)); rast_seg(a[0],a[1],bp[0],bp[1],w,code,l)
    if first_pad is not None: addseg(pts[0][0],(first_pad['x'],first_pad['y']),(pts[0][1],pts[0][2]))
    i=0
    while i<len(pts)-1:
        l,x,y=pts[i]; l2,x2,y2=pts[i+1]
        if l2!=l:
            new_vias.append((code,x,y)); rast_via(x,y,code); i+=1; continue
        d=(round((x2-x)/G),round((y2-y)/G)); j=i+1
        while j<len(pts)-1 and pts[j+1][0]==l and (round((pts[j+1][1]-pts[j][1])/G),round((pts[j+1][2]-pts[j][2])/G))==d: j+=1
        addseg(l,(x,y),(pts[j][1],pts[j][2])); i=j
    if last_pad is not None: addseg(pts[-1][0],(pts[-1][1],pts[-1][2]),(last_pad['x'],last_pad['y']))
necked=[]
def route_net(name):
    pads=pads_by_net[name]; code=b.FindNet(name).GetNetCode(); k=cls(name,b.FindNet(name).GetNetClassName()); w=WIDTHS[k]
    passb={l:((own[k][l]==0)|(own[k][l]==code)).tobytes() for l in ROUTE}
    vo=np.ones((NY,NX),dtype=bool)
    for l in range(NL): vo&=(own['via'][l]==0)|(own['via'][l]==code)
    vo&=~novia; viaok=vo.tobytes()
    cx=sum(p['x'] for p in pads)/len(pads); cy=sum(p['y'] for p in pads)/len(pads)
    pads=sorted(pads,key=lambda p:math.hypot(p['x']-cx,p['y']-cy))
    def nodes(p): 
        cs=pad_cells(p); return [l*NN+c for l in p['layers'] for c in cs]
    grp=groups(name,pads); gof={}
    for gi,(gp,gc) in enumerate(grp):
        for p in gp: gof[id(p)]=gi
    cellpad={}
    def gnodes(gi):
        ns=[]
        for p in grp[gi][0]:
            for n in nodes(p): ns.append(n); cellpad[n]=p
        ps=passb[0]
        ns+=[c for c in grp[gi][1] if ps[c]]
        return ns
    g0=gof[id(pads[0])]; tree=set(gnodes(g0)); done={g0}; todo=[p for p in pads if gof[id(p)]!=g0]
    ok=True
    while todo:
        # nearest remaining pad to tree bbox
        tflag=bytearray(NL*NN); txs=[];tys=[]
        for n in tree:
            tflag[n]=1; r=n%NN; tys.append(r//NX); txs.append(r%NX)
        tb=(min(txs),min(tys),max(txs),max(tys))
        def dist(p):
            x=p['x']/G;y=p['y']/G; dx=max(tb[0]-x,0,x-tb[2]); dy=max(tb[1]-y,0,y-tb[3]); return dx+dy
        todo.sort(key=dist); p=todo.pop(0); gi=gof[id(p)]
        if gi in done: continue
        src=nodes(p)
        if any(tflag[n] for n in src): tree.update(gnodes(gi)); done.add(gi); continue
        path=astar(src,tflag,passb,viaok,tb); ww=w
        if path is None and k!='sig':
            ps={l:((own['sig'][l]==0)|(own['sig'][l]==code)).tobytes() for l in ROUTE}
            path=astar(src,tflag,ps,viaok,tb); ww=WIDTHS['sig']
            if path is not None: necked.append((name,p['ref'],p['num']))
        if path is None: failed.append((name,p['ref'],p['num'])); ok=False; done.add(gi); continue
        commit(path,name,code,ww,p,cellpad.get(path[-1]))
        passb={l:((own[k][l]==0)|(own[k][l]==code)).tobytes() for l in ROUTE}
        tree.update(path); tree.update(gnodes(gi)); done.add(gi)
    return ok
if __name__=='__main__':
    names=[n for n in pads_by_net if n not in SKIP and len(pads_by_net[n])>1]
    def span(n):
        ps=pads_by_net[n]; return (max(p['x'] for p in ps)-min(p['x'] for p in ps))+(max(p['y'] for p in ps)-min(p['y'] for p in ps))
    names.sort(key=span)
    import os, json
    pri=json.load(open('/tmp/priority.json')) if os.path.exists('/tmp/priority.json') else []
    names=[n for n in pri if n in names]+[n for n in names if n not in pri]
    only=[]
    # ---------------- ground: give every surface-mount GND pad a short track to a via into the planes, then stitch
    gcode=b.FindNet('GND').GetNetCode(); gtree=bytearray(NL*NN); nvia0=len(new_vias); gfail=[]
    def gmasks():
        ps={l:((own['sig'][l]==0)|(own['sig'][l]==gcode)).tobytes() for l in ROUTE}
        vo=np.ones((NY,NX),dtype=bool)
        for l in range(NL): vo&=(own['via'][l]==0)|(own['via'][l]==gcode)
        vo&=~novia; return ps,vo
    gp=[p for p in pads_by_net['GND'] if not p['tht'] and not (p['ref'] in('U8','U10','U11'))]
    gp.sort(key=lambda p:(p['x'],p['y']))
    for p in gp:
        ps,vo=gmasks(); cs=pad_cells(p,0.16 if min(p['w'],p['h'])>=0.7 else 0.03); src=[c for c in cs]
        if not src: gfail.append((p['ref'],p['num'])); continue
        # breadth-limited search on the top layer only
        tflag=bytearray(vo.astype(np.uint8).tobytes())
        # also accept joining an earlier ground track
        for n in gcells: tflag[n]=1
        x0=int(p['x']/G); y0=int(p['y']/G)
        path=astar_local(src,tflag,ps[0],(x0,y0),40)
        if path is None: gfail.append((p['ref'],p['num'])); continue
        end=path[-1]; commit(path,'GND',gcode,0.25,p,None)
        if end not in gcells:
            y=end//NX; x=end-y*NX; new_vias.append((gcode,x*G,y*G)); rast_via(x*G,y*G,gcode)
        gcells.update(path)
    npadvia=len(new_vias)-nvia0
    t0=time.time()
    for i,n in enumerate(names):
        if only and n not in only: continue
        t=time.time(); ok=route_net(n)
        pass
    print('routed nets:',len(names),'tracks',len(new_tracks),'vias',len(new_vias),'failed',len(failed),failed,'necked',necked,f'{time.time()-t0:.0f}s')
    fn=[]
    for n,_,_ in failed:
        if n not in fn: fn.append(n)
    json.dump(fn+[n for n in pri if n not in fn],open('/tmp/priority.json','w')); json.dump(len(failed),open('/tmp/nfail.json','w'))
    # stitching vias on a grid (denser beside the RF path)
    gcode=b.FindNet('GND').GetNetCode(); ps,vo=gmasks(); nst=0
    for yy in np.arange(2.0,H-1.9,2.5):
        for xx in np.arange(2.0,W-1.9,2.5):
            dense=abs(yy-52.0)<9.0 or xx>80
            if not dense and (round(xx/2.5)%2 or round(yy/2.5)%2): continue
            ix=int(round(xx/G)); iy=int(round(yy/G)); ps,vo=gmasks()
            if vo[iy,ix]: new_vias.append((gcode,ix*G,iy*G)); rast_via(ix*G,iy*G,gcode); nst+=1
    print('ground: pad vias',npadvia,'stitching vias',nst,'pads left to the pour',len(gfail),gfail[:40])
    def V(x,y): return pcbnew.VECTOR2I(mm(OX+x),mm(OY+y))
    for code,l,a,bp,w in new_tracks:
        t=pcbnew.PCB_TRACK(b); t.SetStart(V(*a)); t.SetEnd(V(*bp)); t.SetWidth(mm(w)); t.SetLayer(LAYERS[l]); t.SetNetCode(code); b.Add(t)
    for code,x,y in new_vias:
        v=pcbnew.PCB_VIA(b); v.SetPosition(V(x,y)); v.SetWidth(mm(VIA_D)); v.SetDrill(mm(VIA_DRL)); v.SetViaType(pcbnew.VIATYPE_THROUGH); v.SetLayerPair(pcbnew.F_Cu,pcbnew.B_Cu); v.SetNetCode(code); b.Add(v)
    pcbnew.SaveBoard(f'{OUT}/{PROJ}.kicad_pcb',b)
