"""Minimal S-expression reader/writer for KiCad files."""
import re
class Sym(str):
    pass
_tok = re.compile(r'\s*(?:(\()|(\))|"((?:[^"\\]|\\.)*)"|([^\s()"]+))')
def parse(text):
    stack=[[]]; pos=0; n=len(text)
    while True:
        m=_tok.match(text,pos)
        if not m: break
        pos=m.end()
        if m.group(1): stack.append([])
        elif m.group(2):
            l=stack.pop(); stack[-1].append(l)
        elif m.group(3) is not None:
            stack[-1].append(m.group(3).replace('\\"','"').replace('\\\\','\\').replace('\\n','\n'))
        else: stack[-1].append(Sym(m.group(4)))
    return stack[0][0]
def dump(x,ind=0):
    if isinstance(x,list):
        simple=all(not isinstance(i,list) for i in x)
        if simple: return '('+' '.join(dump(i) for i in x)+')'
        out='('+dump(x[0])
        for i in x[1:]:
            if isinstance(i,list): out+='\n'+'  '*(ind+1)+dump(i,ind+1)
            else: out+=' '+dump(i)
        return out+'\n'+'  '*ind+')'
    if isinstance(x,Sym): return str(x)
    if isinstance(x,(int,float)):
        return ('%.4f'%x).rstrip('0').rstrip('.') if isinstance(x,float) else str(x)
    return '"'+str(x).replace('\\','\\\\').replace('"','\\"').replace('\n','\\n')+'"'
def find(l,name): return [i for i in l if isinstance(i,list) and i and i[0]==name]
def first(l,name):
    r=find(l,name); return r[0] if r else None
