import numpy as np, math
E24=[1.0,1.1,1.2,1.3,1.5,1.6,1.8,2.0,2.2,2.4,2.7,3.0,3.3,3.6,3.9,4.3,4.7,5.1,5.6,6.2,6.8,7.5,8.2,9.1]
E12=E24[::2]
def nearest(v,series=E12):
    d=math.floor(math.log10(v)); m=v/10**d
    b=min(series+[10.0],key=lambda s:abs(math.log(s/m)))
    return b*10**d
g=[0.618,1.618,2.0,1.618,0.618]  # Butterworth n=5
def lpf(fc):  # shunt C, series L, shunt C, series L, shunt C
    w=2*math.pi*fc; return [('C',g[0]/(w*50)),('L',g[1]*50/w),('C',g[2]/(w*50)),('L',g[3]*50/w),('C',g[4]/(w*50))]
def hpf(fc):  # series C, shunt L, series C, shunt L, series C
    w=2*math.pi*fc; return [('Cs',1/(g[0]*w*50)),('Lp',50/(g[1]*w)),('Cs',1/(g[2]*w*50)),('Lp',50/(g[3]*w)),('Cs',1/(g[4]*w*50))]
def s21(net,f,QL=40,QC=200):
    w=2*math.pi*f; M=np.eye(2,dtype=complex)
    for k,v in net:
        if k in('C','Cs'): z=1/(1j*w*v)+ (1/(w*v))/QC
        else: z=1j*w*v + w*v/QL
        if k in('C','Lp'): A=np.array([[1,0],[1/z,1]])
        else: A=np.array([[1,z],[0,1]])
        M=M@A
    A,B,C,D=M.ravel(); return 20*math.log10(abs(2/(A+B/50+C*50+D)))
def rnd(net): return [(k,nearest(v)) for k,v in net]
def fmt(k,v): return f"{v*1e12:.3g}pF" if k[0]=='C' else f"{v*1e9:.3g}nH"
out={}
for name,fh,fl,probes in [('2m',118e6,180e6,[88e6,100e6,108e6,144e6,146e6,437e6]),('70cm',360e6,540e6,[108e6,146e6,300e6,435e6,438e6,900e6,1311e6])]:
    net=rnd(hpf(fh))+rnd(lpf(fl)); out[name]=net
    print(name,' '.join(f"{k}:{fmt(k,v)}" for k,v in net))
    print('   ',' '.join(f"{p/1e6:.0f}MHz:{s21(net,p):.1f}dB" for p in probes))
# PLL loop filter (3rd order passive), ADF4351
Kv=2*math.pi*40e6; Icp=2.5e-3; Kphi=Icp/(2*math.pi)
for N,label in [(139.84,'70cm N=139.84'),(93.3,'2m N=93.3')]:
    fc=20e3; pm=math.radians(50); wc=2*math.pi*fc
    T1=(1/math.cos(pm)-math.tan(pm))/wc; T2=1/(wc*wc*T1)
    C1=(Kphi*Kv/(N*wc*wc))*(T1/T2)*math.sqrt((1+(wc*T2)**2)/(1+(wc*T1)**2))
    C2=C1*(T2/T1-1); R2=T2/C2
    print(label,f"C1={C1*1e9:.2f}nF C2={C2*1e9:.2f}nF R2={R2:.0f}ohm")
