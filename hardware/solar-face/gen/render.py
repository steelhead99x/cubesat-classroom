"""PNG renders of one board, front and back, for the README."""
import subprocess, sys
VAR = sys.argv[1]; D = f'../kicad/solar-face-{VAR}'; P = f'{D}/solar-face-{VAR}.kicad_pcb'
for side, layers, extra in (('front', 'F.Cu,F.SilkS,F.Mask,Edge.Cuts', []), ('back', 'B.Cu,B.SilkS,B.Mask,Edge.Cuts', ['--mirror'])):
    pdf = f'/tmp/sf-{VAR}-{side}.pdf'
    subprocess.run(['kicad-cli', 'pcb', 'export', 'pdf', '-o', pdf, '--layers', layers, '--black-and-white'] + extra + [P], check=True, capture_output=True)
    out = f'{D}/solar-face-{VAR}-{side}'
    subprocess.run(['pdftoppm', '-r', '150', '-png', '-singlefile', '-cropbox', pdf, out], check=True)
from PIL import Image, ImageOps
for side in ('front', 'back'):
    p = f'{D}/solar-face-{VAR}-{side}.png'; im = Image.open(p).convert('L')
    x0, y0, x1, y1 = ImageOps.invert(im).getbbox()
    im.crop((x0 - 15, y0 - 15, x1 + 15, y1 + 15)).save(p)
print('rendered', VAR)
