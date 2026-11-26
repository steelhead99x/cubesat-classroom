import pcbnew, sys, re, collections
b=pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
if len(sys.argv)>1 and sys.argv[1]=='fill':
    pcbnew.ZONE_FILLER(b).Fill(b.Zones())
pcbnew.WriteDRCReport(b,'/tmp/drc.rpt',pcbnew.EDA_UNITS_MILLIMETRES,True)
t=open('/tmp/drc.rpt').read()
c=collections.Counter(re.findall(r'^\[(\w+)\]',t,re.M)); print(dict(c))
m=re.search(r'Found (\d+) unconnected',t); print('unconnected:',m.group(1) if m else '?')
show=sys.argv[2:] or []
blocks=re.split(r'\n(?=\[)',t)
for k in show:
    n=0
    for bl in blocks:
        if bl.startswith('['+k+']'):
            print(bl.strip().replace('\n',' | ')[:330]); n+=1
            if n>=40: break
