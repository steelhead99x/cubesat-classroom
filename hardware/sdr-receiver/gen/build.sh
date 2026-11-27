#!/bin/bash
# Rebuilds every output of one board from its design source. Needs KiCad 7 (kicad-cli + pcbnew Python).
# CSC_BOARD picks the board: csc-r1 (dev kit, default), csc-r1p (pedal main), csc-r1p-ctl (pedal controls).
set -euo pipefail                       # any failed check stops the build before export
cd "$(dirname "$0")"
proj=$(python3 boardcfg.py proj); kicad=$(python3 boardcfg.py kicad); net=$(python3 boardcfg.py net)
if [ "$(python3 boardcfg.py hand_routed)" = 1 ] && [ "${CSC_REBUILD_ROUTED:-0}" != 1 ]; then
  echo "$proj is routed by hand; rebuilding replaces that copper with the maze router's." >&2
  echo "To check the board in Git, run: CSC_BOARD=$proj python3 drc.py gate" >&2
  echo "To rebuild anyway (in a separate checkout), set CSC_REBUILD_ROUTED=1." >&2
  exit 1
fi
python3 "$(python3 boardcfg.py design).py"   # netlist sanity (no single-pin nets)
python3 footprints.py                   # project footprints (LTC UF24)
python3 gen_sch.py                      # schematic + project symbol library
kicad-cli sch export netlist --format kicadsexpr -o "$net" "$kicad/$proj.kicad_sch" >/dev/null
kicad-cli sch export pdf -o "$kicad/$proj-schematic.pdf" "$kicad/$proj.kicad_sch" >/dev/null
python3 check_net.py | tail -1          # schematic netlist == design
python3 fpaudit.py | tail -1            # every pin has a pad
python3 rf_checks.py                    # loop filter, band filters, 50 ohm width (desk calculations)
python3 "$(python3 boardcfg.py placer)" | tail -1   # placement + hand-routed RF
python3 ovl.py | tail -1                # courtyard overlaps
./iterate.sh                            # maze router, until 0 failures
python3 finish.py                       # ground pours (once, on the routed board)
python3 check_pcb.py                    # board pads == schematic netlist
python3 drc.py gate                     # KiCad DRC: fails unless only the board's documented exceptions remain
./export.sh                             # Gerbers, drill, placement, BOM, PDFs, render
