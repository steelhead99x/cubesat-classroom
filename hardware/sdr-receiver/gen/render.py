import subprocess, sys, boardcfg
from pathlib import Path
from tempfile import TemporaryDirectory
from PIL import Image
Image.MAX_IMAGE_PIXELS=None
def render(layers,box,out,dpi=600,scale=1):
    x0,y0,x1,y1=[v/25.4 for v in (box[0]+60,box[1]+50,box[2]+60,box[3]+50)]
    with TemporaryDirectory() as directory:
        prefix = Path(directory) / 'layout'
        pdf = prefix.with_suffix('.pdf')
        subprocess.run(
            ['kicad-cli', 'pcb', 'export', 'pdf', '-o', str(pdf), '--layers', layers, boardcfg.PCB],
            check=True,
        )
        subprocess.run(
            ['pdftoppm', '-r', str(dpi), '-x', str(int(x0 * dpi)), '-y', str(int(y0 * dpi)),
             '-W', str(int((x1 - x0) * dpi)), '-H', str(int((y1 - y0) * dpi)),
             '-png', str(pdf), str(prefix)],
            check=True,
        )
        with Image.open(prefix.with_name('layout-1.png')) as image:
            image.save(out)
if __name__=='__main__':
    a=sys.argv; render(a[1],[float(v) for v in a[2:6]],a[6],int(a[7]) if len(a)>7 else 600)
