#!/bin/bash
# Exports the fab package from the finished board. Run after build.sh.
# CSC_BOARD picks the board (default csc-r1, the dev kit); see boardcfg.py.
set -euo pipefail
cd "$(dirname "$0")"
proj=$(python3 boardcfg.py proj); kicad=$(python3 boardcfg.py kicad); fab=$(python3 boardcfg.py fab)
net=$(python3 boardcfg.py net); render_box=$(python3 boardcfg.py render)
# Refuse to export a board that fails its checks, even when run on its own.
kicad-cli sch export netlist --format kicadsexpr -o "$net" "$kicad/$proj.kicad_sch" >/dev/null
python3 check_net.py | tail -1
python3 check_pcb.py | grep -E "mismatches|differs|missing"
report=$(mktemp)
# Build the whole package in a staging folder; the fab folder is replaced only when every step
# has succeeded, so a failed export never leaves a half-written or empty package behind.
mkdir -p "$fab"
stage=$(mktemp -d "$(cd "$fab/.." && pwd)/.fab-stage.XXXXXX")
trap 'rm -f -- "$report"; rm -rf -- "$stage"' EXIT
python3 drc.py gate "$report"
PROJ="$proj" KICAD="$kicad" python3 - <<'PY'
import json, os, re
k, n = os.environ['KICAD'], os.environ['PROJ']
p = json.load(open(f'{k}/{n}.kicad_pro'))
root = re.search(r'\(uuid "?([0-9a-f-]+)"?\)', open(f'{k}/{n}.kicad_sch').read()).group(1)
p['sheets'] = [[root, ""]]
json.dump(p, open(f'{k}/{n}.kicad_pro', 'w'), indent=1)
PY
mkdir -p "$stage/gerbers"
( cd "$kicad"
  kicad-cli pcb export gerbers -o "$stage/gerbers/" --layers F.Cu,In1.Cu,In2.Cu,B.Cu,F.Paste,B.Paste,F.SilkS,B.SilkS,F.Mask,B.Mask,Edge.Cuts "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export drill -o "$stage/gerbers/" --format excellon --drill-origin absolute --excellon-units mm "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export pos -o "$stage/$proj-cpl-top.csv" --format csv --units mm --side front "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export pdf -o "$proj-layout-top.pdf" --layers F.Cu,F.SilkS,Edge.Cuts "$proj.kicad_pcb" >/dev/null
  kicad-cli sch export pdf -o "$proj-schematic.pdf" "$proj.kicad_sch" >/dev/null )
python3 bom.py "$stage"
# shellcheck disable=SC2086
python3 render.py F.Cu,F.SilkS,Edge.Cuts $render_box "$kicad/$proj-top.png" 300
cp "$report" "$stage/drc-report.txt"
# Every step succeeded: swap the new package in.
items="gerbers $proj-cpl-top.csv $proj-bom.csv drc-report.txt"
for item in $items; do
  test -e "$stage/$item" || { echo "export: $item missing from the new package" >&2; exit 1; }
done
for item in $items; do
  rm -rf -- "${fab:?}/$item"
  mv -- "$stage/$item" "$fab/$item"
done
rm -f "$net"
echo exported
