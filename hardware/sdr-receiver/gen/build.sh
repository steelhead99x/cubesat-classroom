#!/bin/bash
# Rebuilds every output from gen/design.py. Needs KiCad 7 (kicad-cli + pcbnew Python).
set -euo pipefail                       # any failed check stops the build before export
cd "$(dirname "$0")"
python3 design.py                       # netlist sanity (no single-pin nets)
python3 footprints.py                   # project footprints (LTC UF24)
python3 gen_sch.py                      # schematic + project symbol library
( cd ../kicad && kicad-cli sch export netlist --format kicadsexpr -o rt.net csc-r1.kicad_sch >/dev/null && kicad-cli sch export pdf -o csc-r1-schematic.pdf csc-r1.kicad_sch >/dev/null )
python3 check_net.py | tail -1          # schematic netlist == design
python3 fpaudit.py | tail -1            # every pin has a pad
python3 rf_checks.py                    # loop filter, band filters, 50 ohm width (desk calculations)
python3 gen_pcb.py | tail -1            # placement + hand-routed RF
python3 ovl.py | tail -1                # courtyard overlaps
./iterate.sh 8                          # maze router, until 0 failures
python3 finish.py                       # ground pours (once, on the routed board)
python3 check_pcb.py                    # board pads == schematic netlist
python3 drc.py gate                     # KiCad DRC: fails unless only the documented J3 edge items remain
./export.sh                             # Gerbers, drill, placement, BOM, PDFs, render
