#!/bin/bash
# Rebuilds both solar faces from gen/design.py and gen/cells.py. Needs KiCad 7 (kicad-cli + pcbnew Python).
# Any failed check stops the build before anything is exported.
set -e
cd "$(dirname "$0")"
python3 footprints.py
for v in si tj; do
  python3 gen_sch.py "$v"
  ( cd "../kicad/solar-face-$v" && kicad-cli sch export netlist --format kicadsexpr -o rt.net "solar-face-$v.kicad_sch" >/dev/null )
  python3 check_net.py "$v"      # schematic netlist == design
  python3 gen_pcb.py "$v"        # placement and routing
  python3 check_pcb.py "$v"      # board pads == schematic netlist, mechanical rules
  python3 drc.py "$v"            # KiCad DRC: no errors, nothing unconnected
  python3 render.py "$v"
  python3 export.py "$v"         # Gerbers only when the cell land pattern is verified
  rm -f "../kicad/solar-face-$v/rt.net" "../kicad/solar-face-$v/"*.kicad_prl
done
python3 figures.py
