#!/bin/bash
# Proves the build refuses to export a board that fails its checks.
# Works on a throwaway copy of hardware/sdr-receiver; the real files are not touched.
# Needs KiCad 7 (kicad-cli + pcbnew Python). Run: gen/test_gates.sh
set -uo pipefail
src="$(cd "$(dirname "$0")/.." && pwd)"
fails=0

fresh() {
  tmp=$(mktemp -d); cp -r "$src" "$tmp/r"; echo "$tmp/r"
}
expect_refused() {   # $1 = name, $2 = copy dir
  local before after
  before=$(sha256sum "$2/fab/gerbers/csc-r1-F_Cu.gtl" | cut -c1-16)
  if (cd "$2/gen" && ./export.sh >/tmp/gate-test.log 2>&1); then
    echo "FAIL  $1: export.sh succeeded"; fails=$((fails + 1)); return
  fi
  after=$(sha256sum "$2/fab/gerbers/csc-r1-F_Cu.gtl" | cut -c1-16)
  if [ "$before" != "$after" ]; then echo "FAIL  $1: Gerbers changed"; fails=$((fails + 1)); return; fi
  echo "ok    $1: export refused, Gerbers untouched"
}

# 1. A failing netlist checker.
d=$(fresh); printf 'import sys\nprint("mismatches: 1")\nsys.exit(1)\n' > "$d/gen/check_net.py"
expect_refused "failing netlist check" "$d"

# 2. A pad on the wrong net (D1's pads swapped back the wrong way round).
d=$(fresh)
( cd "$d/gen" && python3 - 2>/dev/null <<'PY'
import pcbnew
b = pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
f = b.FindFootprintByReference('D1')
for p in f.Pads():
    p.SetNet(b.FindNet('GND' if p.GetNumber() == '1' else 'VUSB'))
pcbnew.SaveBoard('../kicad/csc-r1.kicad_pcb', b)
PY
)
expect_refused "pad/net mismatch" "$d"

# 3. An unconnected net: delete every track on VUSB.
d=$(fresh)
( cd "$d/gen" && python3 - 2>/dev/null <<'PY'
import pcbnew
b = pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
for t in list(b.GetTracks()):
    if t.GetNetname() == 'VUSB':
        b.Remove(t)
pcbnew.SaveBoard('../kicad/csc-r1.kicad_pcb', b)
PY
)
expect_refused "unconnected net" "$d"

# 4. The DRC gate itself, on synthetic reports.
( cd "$src/gen" && python3 - <<'PY'
import drc
edge = ("[copper_edge_clearance]: Board edge clearance violation\n"
        "    @(62.1 mm, 99.1 mm): Pad 2 [GND] of J3 on F.Cu\n")
other = ("[copper_edge_clearance]: Board edge clearance violation\n"
         "    @(10 mm, 10 mm): Pad 1 [VUSB] of D1 on F.Cu\n")
clr = "[clearance]: Clearance violation\n    @(1 mm, 1 mm): Track [GND]\n"
ok = edge * 5 + "\n** Found 0 unconnected pads **\n"
assert drc.gate(ok) == [], drc.gate(ok)
assert drc.gate(edge * 6 + "\n** Found 0 unconnected pads **\n")
assert drc.gate(edge + other + "\n** Found 0 unconnected pads **\n")
assert drc.gate(edge + clr + "\n** Found 0 unconnected pads **\n")
assert drc.gate(edge + "\n** Found 2 unconnected pads **\n")
assert drc.gate(edge)
print("ok    DRC gate accepts only J3 edge items with 0 unconnected")
PY
) || fails=$((fails + 1))

[ "$fails" -eq 0 ] && echo "all gate tests passed" || { echo "$fails gate tests failed"; exit 1; }
