#!/bin/bash
# Exports the fab package from the finished board. Run after build.sh.
set -euo pipefail
cd "$(dirname "$0")"
# Refuse to export a board that fails its checks, even when run on its own.
( cd ../kicad && kicad-cli sch export netlist --format kicadsexpr -o rt.net csc-r1.kicad_sch >/dev/null )
python3 check_net.py | tail -1
python3 check_pcb.py | grep -E "mismatches|differs|missing"
report=$(mktemp)
# Build the whole package in a staging folder; fab/ is replaced only when every step has
# succeeded, so a failed export never leaves a half-written or empty package behind.
stage=$(mktemp -d "$PWD/../.fab-stage.XXXXXX")
trap 'rm -f -- "$report"; rm -rf -- "$stage"' EXIT
python3 drc.py gate "$report"
python3 - <<'PY'
import json, re
p = json.load(open('../kicad/csc-r1.kicad_pro'))
root = re.search(r'\(uuid "?([0-9a-f-]+)"?\)', open('../kicad/csc-r1.kicad_sch').read()).group(1)
p['sheets'] = [[root, ""]]
json.dump(p, open('../kicad/csc-r1.kicad_pro', 'w'), indent=1)
PY
mkdir -p "$stage/gerbers"
( cd ../kicad
  kicad-cli pcb export gerbers -o "$stage/gerbers/" --layers F.Cu,In1.Cu,In2.Cu,B.Cu,F.Paste,B.Paste,F.SilkS,B.SilkS,F.Mask,B.Mask,Edge.Cuts csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export drill -o "$stage/gerbers/" --format excellon --drill-origin absolute --excellon-units mm csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export pos -o "$stage/csc-r1-cpl-top.csv" --format csv --units mm --side front csc-r1.kicad_pcb >/dev/null
  kicad-cli pcb export pdf -o csc-r1-layout-top.pdf --layers F.Cu,F.SilkS,Edge.Cuts csc-r1.kicad_pcb >/dev/null
  kicad-cli sch export pdf -o csc-r1-schematic.pdf csc-r1.kicad_sch >/dev/null )
python3 bom.py "$stage"
python3 render.py F.Cu,F.SilkS,Edge.Cuts 0 0 120 78 ../kicad/csc-r1-top.png 300
cp "$report" "$stage/drc-report.txt"
# Every step succeeded: swap the new package in.
for item in gerbers csc-r1-cpl-top.csv csc-r1-bom.csv drc-report.txt; do
  test -e "$stage/$item" || { echo "export: $item missing from the new package" >&2; exit 1; }
done
mkdir -p ../fab
for item in gerbers csc-r1-cpl-top.csv csc-r1-bom.csv drc-report.txt; do
  rm -rf -- "../fab/$item"
  mv -- "$stage/$item" "../fab/$item"
done
rm -f ../kicad/rt.net
echo exported
