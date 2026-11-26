import os, copy
from sexp import *
SYMDIR='/usr/share/kicad/symbols'
_cache={}
def lib(name, path=None):
    if name not in _cache:
        p=path or f'{SYMDIR}/{name}.kicad_sym'
        t=parse(open(p).read())
        _cache[name]={s[1]:s for s in find(t,'symbol')}
    return _cache[name]
def get(libname,symname):
    """Return flattened symbol (extends resolved), named symname."""
    L=lib(libname); s=L[symname]
    ext=first(s,'extends')
    if not ext: return copy.deepcopy(s)
    par=get(libname,ext[1]); pn=ext[1]
    out=[Sym('symbol'),symname]
    props={p[1]:p for p in find(s,'property')}
    for it in par[2:]:
        if isinstance(it,list) and it[0]=='property':
            out.append(copy.deepcopy(props.pop(it[1],it)))
        elif isinstance(it,list) and it[0]=='symbol':
            u=copy.deepcopy(it); u[1]=symname+u[1][len(pn):]; out.append(u)
        else: out.append(copy.deepcopy(it))
    for p in props.values(): out.append(copy.deepcopy(p))
    return out
def pins(sym):
    """list of dict(num,name,type,x,y,ang,unit,hide)"""
    r=[]
    for u in find(sym,'symbol'):
        unit=int(u[1].rsplit('_',2)[1])
        for p in find(u,'pin'):
            at=first(p,'at'); 
            r.append(dict(num=first(p,'number')[1],name=first(p,'name')[1],type=str(p[1]),
                          x=float(at[1]),y=float(at[2]),ang=int(float(at[3])) if len(at)>3 else 0,unit=unit,
                          hide=any(i=='hide' for i in p)))
    return r
def prop(sym,name):
    for p in find(sym,'property'):
        if p[1]==name: return p[2]
    return ''
