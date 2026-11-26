#!/bin/bash
# Exports the fab package from the finished board. Run after build.sh.
set -e
cd "$(dirname "$0")"
python3 - <<'PY'
import json, re
p = json.load(open('../kicad/csc-r1.kicad_pro'))
root = re.search(r'\(uuid "?([0-9a-f-]+)"?\)', open('../kicad/csc-r1.kicad_sch').read()).group(1)
p['sheets'] = [[root, ""]]
json.dump(p, open('../kicad/csc-r1.kicad_pro', 'w'), indent=1)
PY
rm -rf ../fab/gerbers && mkdir -p ../fab/gerbers
( cd ../kicad
  kicad-cli pcb export gerbers -o ../fab/gerbers/ --layers F.Cu,In1.Cu,In2.Cu,B.Cu,F.Paste,B.Paste,F.SilkS,B.SilkS,F.Mask,B.Mask,Edge.Cuts csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export drill -o ../fab/gerbers/ --format excellon --drill-origin absolute --excellon-units mm csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export pos -o ../fab/csc-r1-cpl-top.csv --format csv --units mm --side front csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export pdf -o csc-r1-layout-top.pdf --layers F.Cu,F.SilkS,Edge.Cuts csc-r1.kicad_pcb >/dev/null
  kicad-cli sch export pdf -o csc-r1-schematic.pdf csc-r1.kicad_sch >/dev/null )
python3 bom.py
python3 drc.py nofill >/dev/null && cp /tmp/drc.rpt ../fab/drc-report.txt
python3 render.py F.Cu,F.SilkS,Edge.Cuts 0 0 120 78 ../kicad/csc-r1-top.png 300
rm -f ../kicad/rt.net
echo exported
