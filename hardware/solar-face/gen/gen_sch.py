"""Schematic for one solar face variant (si or tj), from design.py. Adapted from hardware/sdr-receiver/gen/gen_sch.py."""
import uuid, math, json, os
from sexp import Sym as S, dump, parse, find, first
import sys, symlib, design
VAR=sys.argv[1]; design.load(VAR)
PROJ=f'solar-face-{VAR}'; OUT=f'../kicad/{PROJ}'; os.makedirs(OUT,exist_ok=True)
U=lambda: str(uuid.uuid4()); ROOT={'si':'50a0f0ce-0000-4000-8000-000000000051','tj':'50a0f0ce-0000-4000-8000-0000000000f7'}[VAR]
CH=0.95  # label char pitch mm (font 1.0)
def snap(v): return round(v/1.27)*1.27
libsyms={}; items=[]
def getsym(lib,name):
    key=f'{lib}:{name}'
    if key not in libsyms:
        s=symlib.get(lib,name); libsyms[key]=s
    return libsyms[key]
def tf(sx,sy,rot,px,py):
    if rot==0: return (sx+px, sy-py)
    if rot==90: return (sx-py, sy-px)
    raise ValueError
def cell(p):
    s=getsym(p['lib'],p['sym']); pins=symlib.pins(s)
    rot=90 if all(abs(q['x'])<1e-6 for q in pins) and len(pins)==2 else 0
    pts=[]
    for q in pins:
        x,y=tf(0,0,rot,q['x'],q['y'])
        a=math.radians(q['ang']+180); ox,oy=tf(0,0,rot,q['x']+math.cos(a),q['y']+math.sin(a))
        dx,dy=ox-x,oy-y
        d=0 if dx>0.5 else 180 if dx<-0.5 else 90 if dy<-0.5 else 270
        net=p['pins'].get(q['num'])
        assert net is not None, (p['ref'],q['num'],'pin missing in design')
        ln=0 if net==design.NC else len(net)*CH+1
        pts.append((x,y,d,net,ln,q))
    for k in p['pins']: assert k in [q['num'] for q in pins], (p['ref'],k,'no such pin on symbol')
    L=max([-(x)+ln for x,y,d,n,ln,q in pts if d==180]+[0]); Rr=max([x+ln for x,y,d,n,ln,q in pts if d==0]+[0])
    T=max([-(y)+ln for x,y,d,n,ln,q in pts if d==90]+[0]); Bm=max([y+ln for x,y,d,n,ln,q in pts if d==270]+[0])
    xs=[x for x,*_ in pts]; ys=[y for _,y,*_ in pts]
    L=max(L,-min(xs)+1,4); Rr=max(Rr,max(xs)+1,4); T=max(T,-min(ys)+1,3)+5.5; Bm=max(Bm,max(ys)+1,3)+1
    txt=max(len(p['ref']),len(p['value']))*1.1
    Rr=max(Rr,txt-L+2) if rot==90 else max(Rr, txt/2); L=max(L,txt/2) if rot==0 else L
    return dict(rot=rot,pts=pts,L=L,R=Rr,T=T,B=Bm,sym=s)
def place(p,c,sx,sy):
    sx,sy=snap(sx),snap(sy); rot=c['rot']; s=c['sym']; libid=f"{p['lib']}:{p['sym']}"
    val=p['value']
    def prop(k,v,x,y,hide=False,just='left'):
        e=[S('effects'),[S('font'),[S('size'),1.0,1.0]],[S('justify'),S(just)]]
        if hide: e.append(S('hide'))
        return [S('property'),k,v,[S('at'),x,y,rot],e]
    top=sy-(c['T']-5.5)
    it=[S('symbol'),[S('lib_id'),libid],[S('at'),sx,sy,rot],[S('unit'),1],[S('in_bom'),S('yes')],[S('on_board'),S('yes')],
        [S('dnp'),S('yes' if p['dnp'] else 'no')],[S('uuid'),U()],
        prop('Reference',p['ref'],sx-c['L']+1 if rot==0 else sx-2.5,top-3.2),
        prop('Value',val,sx-c['L']+1 if rot==0 else sx-2.5,top-1.6),
        prop('Footprint',p['fp'],sx,sy,True),prop('Datasheet',symlib.prop(s,'Datasheet') or '~',sx,sy,True),
        prop('MPN',p['mpn'],sx,sy,True),prop('Note',p['note'],sx,sy,True)]
    seen=set()
    for q in symlib.pins(s):
        if q['num'] not in seen: it.append([S('pin'),q['num'],[S('uuid'),U()]]); seen.add(q['num'])
    it.append([S('instances'),[S('project'),PROJ,[S('path'),f'/{ROOT}',[S('reference'),p['ref']],[S('unit'),1]]]])
    items.append(it)
    done=set()
    for x,y,d,net,ln,q in c['pts']:
        X,Y=round(sx+x,4),round(sy+y,4)
        if (X,Y,net) in done: continue
        done.add((X,Y,net))
        if net==design.NC:
            items.append([S('no_connect'),[S('at'),X,Y],[S('uuid'),U()]]); continue
        just=[S('justify'),S('right'),S('bottom')] if d in(180,270) else [S('justify'),S('left'),S('bottom')]
        items.append([S('label'),net,[S('at'),X,Y,d],[S('effects'),[S('font'),[S('size'),1.0,1.0]],just],[S('uuid'),U()]])
# ---- flow layout per block
blocks={}
for p in design.parts.values(): blocks.setdefault(p['block'],[]).append(p)
PAGEW=420; X0=15; y=22; W=PAGEW-2*X0
frames=[]
for bname,plist in blocks.items():
    by0=y; y+=9; x=X0+4; rowh=0
    cells=[(p,cell(p)) for p in plist]
    for p,c in cells:
        w=c['L']+c['R']+3; h=c['T']+c['B']+2
        if x+w>X0+W-4: x=X0+4; y+=rowh; rowh=0
        place(p,c,x+c['L'],y+c['T']); x+=w; rowh=max(rowh,h)
    y+=rowh+3
    frames.append((bname,by0,y)); y+=6
PAGEH=y+25
for bname,a,b in frames:
    items.append([S('rectangle'),[S('start'),X0,a],[S('end'),X0+W,b],[S('stroke'),[S('width'),0.2],[S('type'),S('dash')]],[S('fill'),[S('type'),S('none')]],[S('uuid'),U()]])
    items.append([S('text'),bname,[S('at'),X0+3,a+6,0],[S('effects'),[S('font'),[S('size'),3,3],S('bold')],[S('justify'),S('left'),S('bottom')]],[S('uuid'),U()]])
ls=[S('lib_symbols')]
for key,s in libsyms.items():
    s=list(s); s[1]=key; ls.append(s)
sch=[S('kicad_sch'),[S('version'),20230121],[S('generator'),S('solar_face_gen')],[S('uuid'),ROOT],[S('paper'),'User',PAGEW,round(PAGEH)],
     [S('title_block'),[S('title'),design.variant['title']],[S('date'),'2026-11-26'],[S('rev'),'A'],[S('company'),'cubesat-classroom'],
      [S('comment'),1,'Nets are joined by label name. Generated from gen/design.py.'],[S('comment'),2,'UNBUILT, UNTESTED. Cell land patterns NOT VERIFIED (gen/cells.py).']],
     ls]+items+[[S('sheet_instances'),[S('path'),'/',[S('page'),'1']]]]
open(f'{OUT}/{PROJ}.kicad_sch','w').write(dump(sch)+'\n')
open(f'{OUT}/fp-lib-table','w').write('(fp_lib_table\n  (lib (name "solar_face")(type "KiCad")(uri "${KIPRJMOD}/../solar_face.pretty")(options "")(descr "1U solar face cells"))\n)\n')
if not os.path.exists(f'{OUT}/{PROJ}.kicad_pro'):
    json.dump({"meta":{"filename":f"{PROJ}.kicad_pro","version":1},"board":{},"schematic":{},"sheets":[[ROOT,""]]},open(f'{OUT}/{PROJ}.kicad_pro','w'),indent=1)
print('sheet',PAGEW,round(PAGEH),'symbols',len(design.parts))
