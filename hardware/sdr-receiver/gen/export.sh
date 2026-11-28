#!/bin/bash
# Exports the fab package from the finished board. Run after build.sh.
# CSC_BOARD picks the board (default csc-r1, the dev kit); see boardcfg.py.
set -euo pipefail
cd "$(dirname "$0")"
proj=$(python3 boardcfg.py proj); kicad=$(python3 boardcfg.py kicad); fab=$(python3 boardcfg.py fab)
render_box=$(python3 boardcfg.py render)
# Stage exports and intermediates separately from the checked-in CAD files.
mkdir -p "$fab"
stage=$(mktemp -d "$(cd "$fab/.." && pwd)/.fab-stage.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
export CSC_NETLIST="$stage/$proj.net"
# Refuse to export a board that fails its checks, even when run on its own.
kicad-cli sch export netlist --format kicadsexpr -o "$CSC_NETLIST" "$kicad/$proj.kicad_sch" >/dev/null
python3 check_net.py | tail -1
python3 check_pcb.py | grep -E "mismatches|differs|missing"
python3 drc.py gate "$stage/drc-report.txt"
mkdir -p "$stage/gerbers"
# The pedal placement file is for the assembler's top SMT operation. The BOM
# still lists hand-fit hardware and DNP parts with their fit notes.
placement_options=()
if [[ "$proj" == csc-r1p || "$proj" == csc-r1p-ctl ]]; then
  placement_options=(--smd-only --exclude-dnp)
fi
( cd "$kicad"
  kicad-cli pcb export gerbers -o "$stage/gerbers/" --layers F.Cu,In1.Cu,In2.Cu,B.Cu,F.Paste,B.Paste,F.SilkS,B.SilkS,F.Mask,B.Mask,Edge.Cuts "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export drill -o "$stage/gerbers/" --format excellon --drill-origin absolute --excellon-units mm "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export pos -o "$stage/$proj-cpl-top.csv" --format csv --units mm --side front "${placement_options[@]}" "$proj.kicad_pcb" >/dev/null
  kicad-cli pcb export pdf -o "$proj-layout-top.pdf" --layers F.Cu,F.SilkS,Edge.Cuts "$proj.kicad_pcb" >/dev/null
  kicad-cli sch export pdf -o "$proj-schematic.pdf" "$proj.kicad_sch" >/dev/null )
python3 bom.py "$stage"
# shellcheck disable=SC2086
python3 render.py F.Cu,F.SilkS,Edge.Cuts $render_box "$kicad/$proj-top.png" 300
# Every step succeeded: swap the new package in.
items="gerbers $proj-cpl-top.csv $proj-bom.csv drc-report.txt"
if [[ "$proj" == csc-r1p || "$proj" == csc-r1p-ctl ]]; then
  python3 pedal_cam.py write "$stage" "$kicad" "$proj"
  items="$items kicad-cam.json"
fi
for item in $items; do
  test -e "$stage/$item" || { echo "export: $item missing from the new package" >&2; exit 1; }
done
for item in $items; do
  rm -rf -- "${fab:?}/$item"
  mv -- "$stage/$item" "$fab/$item"
done
echo exported
