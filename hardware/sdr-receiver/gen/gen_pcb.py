"""Stage 1: board, footprints, nets, placement, RF hand-routing. Writes out/csc-r1.kicad_pcb"""
import pcbnew, math, json, sys, os
import design
from design import parts
FPDIR='/usr/share/kicad/footprints'; OUT='../kicad'; PROJ='csc-r1'
OX,OY=60.0,50.0; W,H=116.0,78.0
mm=pcbnew.FromMM
def V(x,y): return pcbnew.VECTOR2I(mm(OX+x),mm(OY+y))
board=pcbnew.BOARD(); board.SetCopperLayerCount(4)
nets={}
for i,name in enumerate(sorted(design.nets())):
    ni=pcbnew.NETINFO_ITEM(board,name,i+1); board.Add(ni); nets[name]=ni
fps={}
for r,p in parts.items():
    lib,name=p['fp'].split(':'); f=pcbnew.FootprintLoad(f'{FPDIR}/{lib}.pretty',name)
    f.SetReference(r); f.SetValue(p['value']); f.SetFPID(pcbnew.LIB_ID(lib,name))
    for pd in f.Pads():
        n=p['pins'].get(pd.GetNumber())
        if n and n!=design.NC: pd.SetNet(nets[n])
    if p['dnp']: f.SetAttributes(f.GetAttributes()|pcbnew.FP_EXCLUDE_FROM_POS_FILES|pcbnew.FP_EXCLUDE_FROM_BOM) if hasattr(pcbnew,'FP_EXCLUDE_FROM_BOM') else None
    f.Reference().SetTextSize(pcbnew.VECTOR2I(mm(0.6),mm(0.6))); f.Reference().SetTextThickness(mm(0.1))
    f.SetPosition(V(-30,-30)); board.Add(f); fps[r]=f
placed=set()
def pos(ref,pin):
    ps=[pd.GetPosition() for pd in fps[ref].Pads() if pd.GetNumber()==str(pin)]
    return (sum(pcbnew.ToMM(q.x) for q in ps)/len(ps)-OX, sum(pcbnew.ToMM(q.y) for q in ps)/len(ps)-OY)
def put(ref,x,y,rot=0):
    f=fps[ref]; f.SetOrientationDegrees(rot); f.SetPosition(V(x,y)); placed.add(ref); return ref
def pin_at(ref,pin,x,y,rot=0):
    put(ref,0,0,rot); px,py=pos(ref,pin); put(ref,x-px,y-py,rot); return ref
def inline(ref,x,y,ang,pin='1'):
    """2-pad part: pad `pin` at (x,y), the other pad in screen direction ang (0=+x, 90=down)."""
    rot=-ang if str(pin)=='1' else 180-ang
    pin_at(ref,pin,x,y,rot)
    other='2' if str(pin)=='1' else '1'; ox,oy=pos(ref,other)
    d=math.degrees(math.atan2(oy-y,ox-x))%360
    assert abs((d-ang+180)%360-180)<1,(ref,d,ang)
    return (ox,oy)
def sel(prefix,*ns):
    out=[]
    for r,p in parts.items():
        if r.rstrip('0123456789')==prefix and all(n in p['pins'].values() for n in ns): out.append(r)
    return out
def one(prefix,*ns,i=0): return sel(prefix,*ns)[i]
def pinof(ref,net): return [k for k,v in parts[ref]['pins'].items() if v==net][0]
def row(refs,x,y,dx,dy,rot=0):
    for i,r in enumerate(refs): put(r,x+i*dx,y+i*dy,rot)
tracks=[]
def seg(net,a,b,w=0.35,layer=pcbnew.F_Cu):
    if math.hypot(a[0]-b[0],a[1]-b[1])<1e-4: return
    t=pcbnew.PCB_TRACK(board); t.SetStart(V(*a)); t.SetEnd(V(*b)); t.SetWidth(mm(w)); t.SetLayer(layer); t.SetNet(nets[net]); board.Add(t); tracks.append(t)
def connect(net,w=0.35,only=None):
    """straight-segment minimum spanning tree over the pads of a net"""
    pts=[]
    for r,p in parts.items():
        if only and r not in only: continue
        for k,v in p['pins'].items():
            if v==net: pts.append(pos(r,k))
    done=[pts[0]]; todo=pts[1:]
    while todo:
        best=min(((math.hypot(a[0]-b[0],a[1]-b[1]),a,b) for a in done for b in todo))
        seg(net,best[1],best[2],w); done.append(best[2]); todo.remove(best[2])

RFY=52.0
# ------------------------------------------------------------------ RF chain, left to right
sma=one('J','ANT'); pin_at(sma,'1',1.9,RFY,-90)   # footprint's 'PCB Edge' line lands on x=0
c_dc=one('C','ANT','RF_A'); e=inline(c_dc,7.4,RFY,0)
l_bt=one('L','BT_FILT','ANT'); inline(l_bt,5.9,RFY-1.5,270,pinof(l_bt,'ANT'))
l_st=one('L','RF_A','GND'); inline(l_st,e[0]-0.1,RFY+1.8,90,pinof(l_st,'RF_A'))
bav=one('D','RF_A'); pin_at(bav,'3',e[0]+2.9,RFY+2.4,90)
def breakpoint(n_a,n_b,x):
    ua=[r for r in sel('J',n_a) if parts[r]['sym']=='Conn_Coaxial' and 'BP' in parts[r]['value']][0]
    ub=[r for r in sel('J',n_b) if parts[r]['sym']=='Conn_Coaxial' and 'BP' in parts[r]['value']][0]
    pin_at(ua,'1',x,RFY,90)        # body above the line
    r0=one('R',n_a,n_b); inline(r0,x+3.25,RFY,0,pinof(r0,n_a))
    pin_at(ub,'1',x+7.4,RFY,270)   # body below the line
    assert pos(ua,'2')[1]<RFY and pos(ub,'2')[1]>RFY
    return x+7.4
x=breakpoint('RF_A','LNA_IN',e[0]+3.3)
c=one('C','LNA_IN','LNA_G'); e=inline(c,x+3.1,RFY,0,pinof(c,'LNA_IN'))
lna=one('U','LNA_G','LNA_D'); pin_at(lna,'1',e[0]+1.5,RFY,90)
p3=pos(lna,'3'); assert p3[0]>pos(lna,'1')[0] and abs(p3[1]-RFY)<0.01
l=one('L','5V_LNA','LNA_D'); e2=inline(l,p3[0]+0.2,RFY+1.7,90,pinof(l,'LNA_D'))
cs=sel('C','5V_LNA','GND'); cb=[r for r in cs if parts[r]['value'] in('100n','1n')]
inline(cb[0],e2[0]+1.3,e2[1],0,pinof(cb[0],'5V_LNA')); inline(cb[1],e2[0]-1.3,e2[1],180,pinof(cb[1],'5V_LNA'))
c=one('C','LNA_D','LNA_OUT'); e=inline(c,p3[0]+1.5,RFY,0,pinof(c,'LNA_D'))
x=breakpoint('LNA_OUT','FILT_IN',e[0]+3.1)
sw1=one('U','FILT_IN','F2_IN'); pin_at(sw1,'5',x+3.4,RFY,180)
xs=pos(sw1,'1')[0]+2.2
def ladder(pfx,nin,nout,y,out):
    n=[nin,f'{pfx}_1',f'{pfx}_2',f'{pfx}_M',f'{pfx}_5',nout]; xx=xs; ends=[]
    for a,b in zip(n[:-1],n[1:]):
        k='C' if b in(f'{pfx}_1',f'{pfx}_2',f'{pfx}_M') else 'L'
        r=one(k,a,b); e=inline(r,xx,y,0,pinof(r,a)); xx=e[0]+1.25; ends.append(e)
    for net,e in zip(n[1:],ends):
        for k in('L','C'):
            for r in sel(k,net,'GND'):
                if parts[r]['value']!='100n': inline(r,e[0]+0.6,y+out*1.15,90 if out>0 else 270,pinof(r,net))
    return ends[-1][0]
xe=max(ladder('F2','F2_IN','F2_OUT',RFY-2.4,-1),ladder('F70','F70_IN','F70_OUT',RFY+2.4,+1))
sw2=one('U','FILT_OUT','F2_OUT'); pin_at(sw2,'1',xe+2.6,RFY-0.65,0)
assert abs(pos(sw2,'5')[1]-RFY)<0.01 and pos(sw2,'3')[1]>RFY and pos(sw1,'3')[1]<RFY
x=breakpoint('FILT_OUT','DEM_RF',pos(sw2,'5')[0]+3.3)
cm=sel('C','DEM_RF','GND')[0]; r0=one('R','DEM_RF','DEM_RF_M'); cm2=sel('C','DEM_RF_M','GND')[0]
e=inline(r0,x+3.2,RFY,0,pinof(r0,'DEM_RF'))
inline(cm,x+3.2,RFY+1.1,90,pinof(cm,'DEM_RF')); inline(cm2,e[0],RFY+1.1,90,pinof(cm2,'DEM_RF_M'))
bal=one('T','DEM_RF_M'); pin_at(bal,'6',e[0]+2.2,RFY-1.27,-90)
b1=pos(bal,'1'); b3=pos(bal,'3'); assert b1[0]>pos(bal,'6')[0] and b1[1]<b3[1]
l6=one('L','RF_BAL_P','RF_BAL_N'); inline(l6,b1[0]+1.7,RFY-0.48,90,pinof(l6,'RF_BAL_P'))
cp=one('C','RF_BAL_P','DEM_RFP'); cn=one('C','RF_BAL_N','DEM_RFN')
ep=inline(cp,b1[0]+2.9,RFY-0.5,0,pinof(cp,'RF_BAL_P')); inline(cn,b1[0]+2.9,RFY+0.5,0,pinof(cn,'RF_BAL_N'))
dem=one('U','DEM_RFP','LO_P'); pin_at(dem,'5',ep[0]+2.4,RFY-0.25,0)
CX,CY=fps[dem].GetPosition(); CX=pcbnew.ToMM(CX)-OX; CY=pcbnew.ToMM(CY)-OY
print('demod centre',round(CX,2),round(CY,2))
# ------------------------------------------------------------------ LO: PLL right of demod, RF outputs facing left
pll=one('U','PLL_CP')
dn=pos(dem,'16'); dp=pos(dem,'15'); assert dn[1]<dp[1]
ca=one('C','LOA_P','LO_N'); cb_=one('C','LOA_N','LO_P')
ea=inline(ca,dn[0]+2.9,CY-0.5,0,pinof(ca,'LO_N')); inline(cb_,dn[0]+2.9,CY+0.5,0,pinof(cb_,'LO_P'))
ra=one('R','3V3_VCO','LOA_P'); rb=one('R','3V3_VCO','LOA_N')
inline(ra,ea[0],CY-1.75,270,pinof(ra,'LOA_P')); inline(rb,ea[0],CY+1.75,90,pinof(rb,'LOA_N'))
pin_at(pll,'12',ea[0]+2.5,CY-0.25,-90)
PX,PY=[pcbnew.ToMM(v) for v in fps[pll].GetPosition()]; PX-=OX; PY-=OY
assert abs(PY-CY)<0.01 and pos(pll,'13')[1]>pos(pll,'12')[1]
print('pll centre',round(PX,2),round(PY,2))

def at(ref,net,x,y,ang): return inline(ref,x,y,ang,pinof(ref,net))
def path(net,pts,w=0.35):
    for a,b in zip(pts[:-1],pts[1:]): seg(net,a,b,w)
# aux output B -> LO monitor U.FL; B- just terminated to VVCO
p14=pos(pll,'14'); p15=pos(pll,'15')
def path(net,pts,w=0.35):
    for a,b in zip(pts[:-1],pts[1:]): seg(net,a,b,w)
rbm=one('R','3V3_VCO','LOB_N'); at(rbm,'LOB_N',PX-3.6,CY+3.7,90)
path('LOB_N',[p15,(PX-3.6,p15[1]),(PX-3.6,CY+3.7)],0.25)
rbp=one('R','3V3_VCO','LOB_P'); at(rbp,'LOB_P',PX-5.3,CY+4.2,180)
cm_=one('C','LOB_P','LO_MON'); e=at(cm_,'LOB_P',PX-4.2,CY+5.7,90)
path('LOB_P',[p14,(PX-4.2,p14[1]),(PX-4.2,CY+5.7)],0.25); path('LOB_P',[(PX-4.2,CY+4.2),(PX-5.3,CY+4.2)],0.25)
um=one('J','LO_MON'); pin_at(um,'1',e[0],e[1]+1.9,270); path('LO_MON',[e,pos(um,'1')])
# ---------------- hand routes for the RF nets (straight MST between pads)
for n in ['ANT','RF_A','LNA_IN','LNA_G','LNA_D','LNA_OUT','FILT_IN','F2_IN','F2_1','F2_2','F2_M','F2_5','F2_OUT','F70_IN','F70_1','F70_2','F70_M','F70_5','F70_OUT',
          'FILT_OUT','DEM_RF','DEM_RF_M','RF_BAL_P','RF_BAL_N']:
    connect(n,0.35)
for n,cc,pn in (('DEM_RFP',cp,'5'),('DEM_RFN',cn,'6')):
    q=pos(dem,pn); path(n,[pos(cc,pinof(cc,n)),(q[0]-1.5,q[1]),q],0.2)
for sgn,na,nl,pa,pd in ((-1,'LOA_P','LO_N','12','16'),(1,'LOA_N','LO_P','13','15')):
    pp=pos(pll,pa); cpad=(ea[0],CY+sgn*0.5); rr=ra if sgn<0 else rb
    path(na,[pp,(ea[0]+0.49,pp[1]),cpad,pos(rr,pinof(rr,na))],0.22)
    dd_=pos(dem,pd); path(nl,[(dn[0]+2.9,CY+sgn*0.5),(dn[0]+2.3,dd_[1]),dd_],0.22)

# ---------------- demodulator support
offs={'IP':(-2.65,-1.55),'IN':(0.65,-0.45),'QP':(1.75,2.85),'QN':(5.05,3.95)}
for k,(xp,xs_) in offs.items():
    at(one('R','5V_DEMOD','BB_'+k),'BB_'+k,CX+xp,CY-5.0,270); at(one('R','BB_'+k,'BBF_'+k),'BB_'+k,CX+xs_,CY-5.0,270)
at(one('C','BBF_IP','BBF_IN'),'BBF_IP',CX-1.48,CY-7.4,0); at(one('C','BBF_QP','BBF_QN'),'BBF_QP',CX+2.92,CY-7.4,0)
for k,xx,adc in (('IP',-1.9,'ADC_IN1P'),('IN',-0.1,'ADC_IN1M'),('QP',2.5,'ADC_IN2P'),('QN',4.3,'ADC_IN2M')):
    cc_=one('C','BBF_'+k,adc); at(cc_,'BBF_'+k,CX+xx,CY-8.8,270)
    connect('BBF_'+k,0.2,only=[one('R','BB_'+k,'BBF_'+k),one('C','BBF_IP','BBF_IN') if k[0]=='I' else one('C','BBF_QP','BBF_QN'),cc_])
at(one('C','DEM_CMQ','DEM_VCAP'),'DEM_CMQ',CX+3.1,CY-1.5,0); at(one('C','DEM_CMI','DEM_VCAP'),'DEM_CMI',CX+2.7,CY-3.3,0)
at(one('C','DEM_REF','GND'),'DEM_REF',CX-2.3,CY-3.3,180)
dv=sel('C','5V_DEMOD','GND')   # 10u (power blk), 1u, 10n
c10u=[r for r in dv if parts[r]['value']=='10u'][0]; c1u=[r for r in dv if parts[r]['value']=='1u'][0]; c10n=[r for r in dv if parts[r]['value']=='10n'][0]
at(c10n,'5V_DEMOD',CX+0.25,CY+4.6,90); at(c1u,'5V_DEMOD',CX+0.25,CY+6.8,90)
at(one('R','DEM_VBIAS','GND'),'DEM_VBIAS',CX-1.1,CY+4.6,90); at(one('R','DEMOD_EN','GND'),'DEMOD_EN',CX-2.4,CY+4.6,90)
jd=one('JP','5V_DEMOD_J'); put(jd,CX-8.4,CY+13.0,90); fb=one('FB','5V_DEMOD_J','5V_DEMOD'); at(fb,'5V_DEMOD',CX-1.5,CY+10.2,180); at(c10u,'5V_DEMOD',CX+0.0,CY+10.2,0)
# trim DAC and dividers
dac=one('U','DAC_A'); put(dac,CX-19.2,CY-6.8,0)
for i,(d,t) in enumerate((('DAC_D','TRIM_IP2Q'),('DAC_C','TRIM_DCOQ'),('DAC_B','TRIM_DCOI'),('DAC_A','TRIM_IP2I'))):
    x0=CX-4.6-2.4*i; y0=CY-7.8
    rs_=one('R',d,t); at(rs_,t,x0,y0,180); at(one('R',t,'GND'),t,x0,y0+1.0,180); at(one('C',t,'GND'),t,x0,y0+2.0,180)
    connect(t,0.2,only=[rs_,one('R',t,'GND'),one('C',t,'GND')])
    q=pos(dac,pinof(dac,d)); pd_=pos(rs_,pinof(rs_,d)); xt=CX-14.2-0.4*(3-i)
    if i==3: path(d,[q,(xt,q[1]),(xt,pd_[1]),pd_],0.2)
    else:
        ly=y0-1.3-0.4*(2-i); path(d,[q,(xt,q[1]),(xt,ly),(pd_[0],ly),pd_],0.2)
dd=[r for r in sel('C','3V3_DIG','GND') if parts[r]['block'].startswith('4')]
at(dd[0],'3V3_DIG',CX-23.2,CY-8.4,270); put(dd[1],CX-23.8,CY-5.2,90)
# ---------------- ADC
AX,AY=CX+1.2,CY-15.5
adc=one('U','ADC_AREG'); put(adc,AX,AY,0)
va=sel('C','3V3_ADC','GND'); va=[r for r in va if parts[r]['block'].startswith('5')]
at([r for r in va if parts[r]['value']=='100n'][0],'3V3_ADC',AX-3.2,AY-2.3,180); at([r for r in va if parts[r]['value']=='10u'][0],'3V3_ADC',AX-6.0,AY-2.9,180)
ar=sel('C','ADC_AREG','GND'); at([r for r in ar if parts[r]['value']=='100n'][0],'ADC_AREG',AX-3.2,AY-1.2,180); at([r for r in ar if parts[r]['value']=='10u'][0],'ADC_AREG',AX-6.0,AY-0.6,180)
at(one('C','ADC_VREF','GND'),'ADC_VREF',AX-3.5,AY+1.3,180)
dr=sel('C','ADC_DREG','GND'); at([r for r in dr if parts[r]['value']=='100n'][0],'ADC_DREG',AX-1.9,AY-3.5,270); at([r for r in dr if parts[r]['value']=='10u'][0],'ADC_DREG',AX-3.4,AY-5.6,180)
io=[r for r in sel('C','3V3_DIG','GND') if parts[r]['block'].startswith('5')][0]; at(io,'3V3_DIG',AX+1.9,AY-3.5,270)
for n,xx in (('ADC_FSYNC',-1.75),('ADC_BCLK',-0.8),('ADC_SDOUT',0.15)): at(one('R',n),n,AX+xx,AY-5.6,270)
at(one('R','ADC_SHDNZ','GND'),'ADC_SHDNZ',AX+3.4,AY+0.75,0)
osc=one('Y','MCLK_OSC_R'); put(osc,AX+6.5,AY-2.0,0)
at(one('R','MCLK_OSC_R','MCLK_OSC'),'MCLK_OSC_R',AX+9.7,AY-2.8,270)
at(one('FB','3V3_DIG','3V3_MCLK'),'3V3_MCLK',AX+4.6,AY-4.7,0); at(one('C','3V3_MCLK','GND'),'3V3_MCLK',AX+6.0,AY+0.9,0)
put(one('JP','MCLK_OSC','ADC_MCLK'),AX+13.0,AY-4.8,0)
put(one('J','BBF_IP','BBF_QN'),CX-21.5,CY-11.8,90)
# ADC LDO
def ldo(rail,x,y):
    u=one('U',rail+'_IN',rail); put(u,x,y,0); jp=one('JP',rail+'_IN'); put(jp,x-4.6,y-1.27,0)
    at(one('C',rail+'_IN','GND'),rail+'_IN',x-2.4,y-0.95,180) if False else put(one('C',rail+'_IN','GND'),x-0.4,y+2.6,0)
    co=[r for r in sel('C',rail,'GND') if parts[r]['block'].startswith('1')][0]; put(co,x+2.9,y-0.4,90)
ldo('3V3_ADC',AX-15.7,AY-1.5)
# ---------------- PLL support
pl=[r for r in sel('C','3V3_PLL','GND')]; p100=[r for r in pl if parts[r]['value']=='100n']; p10=[r for r in pl if parts[r]['value']=='10p']; p10u=[r for r in pl if parts[r]['value']=='10u'][0]
at(p100[0],'3V3_PLL',PX-0.45,PY-4.4,270); at(p10[0],'3V3_PLL',PX-2.6,PY-4.5,270)          # VP
at(p100[1],'3V3_PLL',PX-3.8,PY-1.25,270); at(p10[1],'3V3_PLL',PX-3.8,PY-3.4,270)       # AVDD
at(p100[2],'3V3_PLL',PX+4.6,PY-1.9,270); at(p10[2],'3V3_PLL',PX+5.7,PY-1.9,270)        # SDVDD
at(p100[3],'3V3_PLL',PX+4.6,PY+1.0,90); at(p10[3],'3V3_PLL',PX+5.7,PY+1.0,90)          # DVDD
at(p10u,'3V3_PLL',PX+5.6,PY-6.6,0)
at(one('C','PLL_CP','GND'),'PLL_CP',PX-1.9,PY-7.2,180); at(one('R','PLL_CP','PLL_VTUNE'),'PLL_CP',PX-0.6,PY-7.2,0)
at(one('R','PLL_CP','PLL_LF2'),'PLL_CP',PX-1.9,PY-8.3,270); at(one('C','PLL_LF2','GND'),'PLL_LF2',PX-1.9,PY-10.5,270)
at(one('R','PLL_CE','3V3_DIG'),'PLL_CE',PX+3.0,PY-6.5,0)
vc=sel('C','3V3_VCO','GND'); v100=[r for r in vc if parts[r]['value']=='100n']; v10=[r for r in vc if parts[r]['value']=='10p']; v10u=[r for r in vc if parts[r]['value']=='10u'][0]
at(v100[0],'3V3_VCO',PX-2.0,PY+5.2,90); at(one('C','PLL_TEMP','GND'),'PLL_TEMP',PX-0.9,PY+5.2,90)
at(one('C','PLL_VTUNE','GND'),'PLL_VTUNE',PX+0.2,PY+5.2,90); at(one('R','PLL_RSET','GND'),'PLL_RSET',PX+1.3,PY+5.2,90)
at(one('C','PLL_VCOM','GND'),'PLL_VCOM',PX+2.4,PY+5.2,90); at(one('C','PLL_VREF','GND'),'PLL_VREF',PX+3.5,PY+5.2,90)
at(v10[0],'3V3_VCO',PX-0.9,PY+7.6,90); at(v100[1],'3V3_VCO',PX+0.2,PY+7.6,90); at(v10[1],'3V3_VCO',PX+1.3,PY+7.6,90); at(v10u,'3V3_VCO',PX+2.8,PY+8.0,0)
for n_,pn_ in (('3V3_VCO','17'),('PLL_TEMP','19'),('PLL_VTUNE','20'),('PLL_RSET','22'),('PLL_VCOM','23'),('PLL_VREF','24')):
    rr_=[r for r in sel('C',n_,'GND')+sel('R',n_,'GND') if abs(pos(r,pinof(r,n_))[1]-(PY+5.2))<0.01][0]; q=pos(pll,pn_); t_=pos(rr_,pinof(rr_,n_))
    path(n_,[q,(q[0],PY+3.3),(t_[0],PY+3.3+abs(t_[0]-q[0])) if abs(t_[0]-q[0])<1.5 else (t_[0],PY+4.9),t_],0.2)
q=pos(pll,'28'); t_=pos(p100[3],pinof(p100[3],'3V3_PLL')); path('3V3_PLL',[q,(PX+3.9,q[1]),t_],0.2)
q=pos(pll,'32'); t_=pos(p100[2],pinof(p100[2],'3V3_PLL')); path('3V3_PLL',[q,(PX+3.9,q[1]),t_],0.2)
q=pos(pll,'6'); t_=pos(p100[0],pinof(p100[0],'3V3_PLL')); path('3V3_PLL',[q,(q[0],PY-3.5),(q[0]+0.15,PY-4.15)],0.2)
q=pos(pll,'10'); t_=pos(p100[1],pinof(p100[1],'3V3_PLL')); path('3V3_PLL',[q,t_],0.2)
put(one('TP','PLL_VTUNE'),PX+5.6,PY+10.6); put(one('TP','PLL_LD'),PX+7.9,PY+10.6); put(one('TP','PLL_PDBRF'),PX+10.2,PY+10.6)
at(one('R','PLL_PDBRF','3V3_DIG'),'PLL_PDBRF',PX+9.6,PY+8.0,270)
at(one('R','PLL_LD','LED_LOCK_A'),'PLL_LD',PX+6.6,PY+3.4,0); at(one('D','LED_LOCK_A','GND'),'LED_LOCK_A',PX+6.6,PY+5.0,0)
tcxo=one('Y','TCXO_OUT'); put(tcxo,PX+8.8,PY-0.4,90)
at(one('C','TCXO_OUT','PLL_REFIN'),'PLL_REFIN',PX+4.5,PY-0.45,0)
q=pos(pll,'29'); cr_=one('C','TCXO_OUT','PLL_REFIN'); path('PLL_REFIN',[q,(PX+3.6,q[1]),pos(cr_,pinof(cr_,'PLL_REFIN'))],0.2)
at(one('FB','3V3_PLL','3V3_TCXO'),'3V3_TCXO',PX+8.4,PY-4.6,180); at(one('C','3V3_TCXO','GND'),'3V3_TCXO',PX+9.7,PY-4.6,90)
at(one('R','PLL_MUXOUT_R','PLL_MUXOUT'),'PLL_MUXOUT_R',PX+4.6,PY-3.9,0)
ldo('3V3_VCO',PX-5.5,PY+15.6); ldo('3V3_PLL',PX+6.0,PY+15.6)
# ---------------- controller area
HX=AX-12.2
ctrl=one('J','I2S_BCLK','XD0'); pin_at(ctrl,'1',HX,AY-9.5,90)
if pos(ctrl,'2')[1]>pos(ctrl,'1')[1]: pin_at(ctrl,'1',HX+22.86,AY-9.5,270)
XL=min(pos(ctrl,'1')[0],pos(ctrl,'19')[0])+3.8
xl=one('J','XD0','XD6'); pin_at(xl,'1',XL,3.6,0); xr=one('J','XIAO_5V'); pin_at(xr,'1',XL+15.24,3.6,0)
aux=one('J','XD6','XD7','XIAO_3V3'); pin_at(aux,'1',XL-6.5,4.5,0)
jx=one('JP','XIAO_5V_D'); put(jx,XL+21.2,9.0,0); dx=one('D','XIAO_5V_D'); put(dx,XL+21.2,16.8,90)
exp=one('U','EXP_INT'); put(exp,HX-9.0,AY-10.5,0)
at(one('R','SDA','3V3_DIG'),'SDA',HX-3.2,AY-14.5,270); at(one('R','SCL','3V3_DIG'),'SCL',HX-4.4,AY-14.5,270)
ex=[r for r in sel('C','3V3_DIG','GND') if parts[r]['block'].startswith('7')][0]; put(ex,HX-9.0,AY-14.5,0)
at(one('R','EXP_INT','3V3_DIG'),'EXP_INT',HX-14.4,AY-10.5,270); put(one('TP','EXP_INT'),HX-16.4,AY-11.0)
at(one('R','3V3_DIG','LED_USER_A'),'LED_USER_A',HX-14.4,AY-8.4,90); at(one('D','LED_USER_A','LED_USER_K'),'LED_USER_A',HX-16.2,AY-8.6,90)
ldo('3V3_DIG',HX-15.0,AY-18.0)
at(one('R','3V3_DIG','LED_3V3_A'),'3V3_DIG',HX-10.5,AY-18.5,0); at(one('D','LED_3V3_A','GND'),'LED_3V3_A',HX-10.5,AY-17.0,0)
# ---------------- bias tee (above the antenna input)
at(one('C','BT_FILT','GND',i=0),'BT_FILT',7.3,RFY-3.3,270); at(one('C','BT_FILT','GND',i=1),'BT_FILT',8.5,RFY-3.3,270)
at(one('R','BT_SW','BT_FILT'),'BT_FILT',5.9,RFY-4.6,270)
q1=one('Q','BT_GATE','BT_SW'); put(q1,5.2,RFY-8.6,90); q2=one('Q','BIAS_EN','BT_GATE'); put(q2,10.2,RFY-8.6,90)
at(one('R','BT_FUSED','BT_GATE'),'BT_GATE',7.7,RFY-9.6,90); at(one('R','BIAS_EN','GND'),'BIAS_EN',12.6,RFY-9.0,90)
put(one('F','BT_FUSED'),5.2,RFY-12.6,0)
at(one('R','BT_SW','LED_BT_A'),'BT_SW',9.2,RFY-5.6,0); at(one('D','LED_BT_A','GND'),'LED_BT_A',11.6,RFY-5.6,0)
# LNA rail
jl=one('JP','5V_LNA_J'); put(jl,pos(lna,'2')[0]+8.0,RFY+8.5,90); at(one('FB','5V_LNA_J','5V_LNA'),'5V_LNA',pos(lna,'2')[0]+1.5,RFY+6.4,180)
l10=[r for r in sel('C','5V_LNA','GND') if parts[r]['value']=='10u'][0]; at(l10,'5V_LNA',pos(lna,'2')[0]+3.1,RFY+6.4,0)
# switches' support
s1=pos(sw1,'2'); s2=pos(sw2,'2'); sc=[r for r in sel('C','3V3_DIG','GND') if parts[r]['block'].startswith('3')]
put(sc[0],s1[0]-0.9,RFY+6.6,0); put(sc[1],s2[0]+0.6,RFY+6.6,0)
at(one('R','BAND_SEL','GND'),'BAND_SEL',s1[0]-0.9,RFY+8.0,0)
qi=one('Q','BAND_SEL','BAND_SEL_N'); put(qi,s2[0]+0.6,RFY+9.6,90); at(one('R','BAND_SEL_N','3V3_DIG'),'BAND_SEL_N',s2[0]+3.2,RFY+8.4,90)
# ---------------- power input (bottom-left)
usb=parts['J1'] and 'J1'; put(usb,14.0,H-3.2,0)
at(one('R','USB_CC1','GND'),'USB_CC1',12.6,H-8.6,270); at(one('R','USB_CC2','GND'),'USB_CC2',15.4,H-8.6,270)
st=one('J','VEXT','GND'); put(st,27.0,H-4.6,0)
put(one('JP','VUSB','VIN','VEXT'),23.0,H-14.0,90); put(one('F','VIN','+5V'),33.0,H-12.6,0)
b5=[r for r in sel('C','+5V','GND')]; put(b5[0],37.4,H-12.6,90); put(b5[1],40.0,H-12.6,90); put(b5[2],42.2,H-12.6,90)
at(one('R','+5V','LED_PWR_A'),'+5V',36.4,H-8.8,0); at(one('D','LED_PWR_A','GND'),'LED_PWR_A',39.0,H-8.8,0)
gt=[r for r in sel('TP','GND')]; put(gt[0],46.0,H-5.0); put(gt[1],62.0,H-9.0); put(gt[2],40.0,8.0)
mh=sel('H','GND'); put(mh[0],4,4); put(mh[1],W-4,4); put(mh[2],4,H-4); put(mh[3],W-4,H-4)
missing=[r for r in parts if r not in placed]
print('unplaced:',missing)
# ---------------- outline, save
def edge(a,b):
    s=pcbnew.PCB_SHAPE(board); s.SetShape(pcbnew.SHAPE_T_SEGMENT); s.SetStart(V(*a)); s.SetEnd(V(*b)); s.SetLayer(pcbnew.Edge_Cuts); s.SetWidth(mm(0.1)); board.Add(s)
for a,b in (((0,0),(W,0)),((W,0),(W,H)),((W,H),(0,H)),((0,H),(0,0))): edge(a,b)
def ncl(name,w,cl=0.15):
    return {"name":name,"clearance":cl,"track_width":w,"via_diameter":0.6,"via_drill":0.3,"microvia_diameter":0.3,"microvia_drill":0.1,
            "diff_pair_width":0.2,"diff_pair_gap":0.25,"diff_pair_via_gap":0.25,"bus_width":12,"wire_width":6,"line_style":0,
            "pcb_color":"rgba(0, 0, 0, 0.000)","schematic_color":"rgba(0, 0, 0, 0.000)"}
power=['+5V','VIN','VUSB','VEXT','5V_DEMOD','5V_DEMOD_J','5V_LNA','5V_LNA_J','BT_FUSED','BT_SW','BT_FILT','XIAO_5V','XIAO_5V_D']+[r+s for r in ('3V3_VCO','3V3_PLL','3V3_ADC','3V3_DIG') for s in ('','_IN')]+['3V3_TCXO','3V3_MCLK']
pro={}
pro['net_settings']={"classes":[ncl('Default',0.2),ncl("Power",0.3)],"meta":{"version":3},"net_colors":None,"netclass_assignments":None,
                     "netclass_patterns":[{"netclass":"Power","pattern":n} for n in power]}
pro['board']={"design_settings":{"defaults":{},"meta":{"version":2},
  "rules":{"min_clearance":0.127,"min_track_width":0.127,"min_via_diameter":0.45,"min_through_hole_diameter":0.2,"min_via_annular_width":0.1,
           "min_hole_clearance":0.2,"min_hole_to_hole":0.25,"min_copper_edge_clearance":0.3,"min_microvia_diameter":0.2,"min_microvia_drill":0.1,
           "min_silk_clearance":0.0,"min_text_height":0.5,"min_text_thickness":0.08,"solder_mask_to_copper_clearance":0.0,"min_connection":0.0,
           "min_resolved_spokes":1,"allow_blind_buried_vias":False,"allow_microvias":False,"max_error":0.005},
  "rule_severities":{"silk_overlap":"ignore","silk_over_copper":"ignore","silk_edge_clearance":"ignore","lib_footprint_issues":"ignore",
                     "lib_footprint_mismatch":"ignore","text_height":"ignore","text_thickness":"ignore","solder_mask_bridge":"warning"},
  "track_widths":[0.0,0.2,0.35,0.4],"via_dimensions":[{"diameter":0.0,"drill":0.0},{"diameter":0.6,"drill":0.3}]}}
pcbnew.SaveBoard(f'{OUT}/{PROJ}.kicad_pcb',board)
full=json.load(open(f'{OUT}/{PROJ}.kicad_pro')); full['net_settings']=pro['net_settings']
full['board']['design_settings']['rules'].update(pro['board']['design_settings']['rules']); full['board']['design_settings']['rule_severities'].update(pro['board']['design_settings']['rule_severities'])
json.dump(full,open(f'{OUT}/{PROJ}.kicad_pro','w'),indent=1)
print('saved; tracks',len(tracks))

import shutil; shutil.copy(f'{OUT}/{PROJ}.kicad_pcb',f'{OUT}/base.kicad_pcb'); shutil.copy(f'{OUT}/{PROJ}.kicad_pro',f'{OUT}/base.kicad_pro')
