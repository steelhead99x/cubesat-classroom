#!/bin/bash
# Proves the build refuses to export a board that fails its checks.
# Works on a throwaway copy of hardware/sdr-receiver; the real files are not touched.
# Needs KiCad 7 (kicad-cli + pcbnew Python). Run: gen/test_gates.sh
set -euo pipefail
src="$(cd "$(dirname "$0")/.." && pwd)"
# Exercise the parser without loading KiCad.
( cd "$src/gen" && python3 - <<'PY'
import drc
edge = ("[copper_edge_clearance]: Board edge clearance violation\n"
        "    @(62.1 mm, 99.1 mm): Pad 2 [GND] of J3 on F.Cu\n")
other = ("[copper_edge_clearance]: Board edge clearance violation\n"
         "    @(10 mm, 10 mm): Pad 1 [VUSB] of D1 on F.Cu\n")
clr = "[clearance]: Clearance violation\n    @(1 mm, 1 mm): Track [GND]\n"
def report(count, blocks, unconnected=0):
    return (f"** Found {count} DRC violations **\n" + blocks
            + f"\n** Found {unconnected} unconnected pads **\n"
            + "** Found 0 Footprint errors **\n** End of Report **\n")

assert drc.gate(report(5, edge * 5)) == []
assert drc.gate(report(6, edge * 6))
assert drc.gate(report(2, edge + other))
assert drc.gate(report(2, edge + clr))
assert drc.gate(report(1, edge, unconnected=2))
assert drc.gate(report(1, ''))
assert drc.gate(report(1, edge).split('** End')[0])
assert drc.gate(edge)
print("ok    DRC gate accepts only J3 edge items with 0 unconnected")
PY
)

# The routed pedal boards: build.sh must refuse before it changes any file. No KiCad needed.
# Runs on a throwaway copy with the override cleared, so a broken guard (or an override left
# set in the caller's shell) can't touch the real boards.
guard=$(mktemp -d)
cp -R "$src/." "$guard/"
for board in csc-r1p csc-r1p-ctl; do
  pcb="$guard/pedal/kicad/$board.kicad_pcb"
  before=$(sha256sum < "$pcb")
  # It must refuse for the right reason: any other failure (a missing tool, say) doesn't count.
  if out=$(cd "$guard/gen" && env -u CSC_REBUILD_ROUTED CSC_BOARD=$board ./build.sh 2>&1); then
    rm -rf -- "$guard"; echo "FAIL  build.sh rebuilt the routed $board board" >&2; exit 1
  fi
  if ! grep -q "$board is routed by hand" <<<"$out"; then
    rm -rf -- "$guard"; echo "FAIL  build.sh failed for $board without the routed-board refusal:" >&2; echo "$out" >&2; exit 1
  fi
  [[ "$(sha256sum < "$pcb")" == "$before" ]] || { rm -rf -- "$guard"; echo "FAIL  build.sh changed $board.kicad_pcb" >&2; exit 1; }
done
rm -rf -- "$guard"
echo "ok    build.sh refuses to rebuild the routed pedal boards and leaves them untouched"

if [[ "${1:-}" == "--report-only" ]]; then exit 0; fi
command -v kicad-cli >/dev/null || { echo "test_gates needs kicad-cli" >&2; exit 1; }
python3 -c 'import pcbnew'
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

fresh() {
  local dir
  dir=$(mktemp -d "$work/case.XXXXXX")
  cp -r "$src" "$dir/r"
  echo "$dir/r"
}

expect_refused() {
  local name=$1 dir=$2 reason=$3 log="$2/export.log"
  if (cd "$dir/gen" && ./export.sh >"$log" 2>&1); then
    echo "FAIL  $name: export.sh succeeded" >&2
    exit 1
  fi
  if ! grep -Fq "$reason" "$log"; then
    cat "$log" >&2
    echo "FAIL  $name: wrong reason for refusal" >&2
    exit 1
  fi
  diff -qr "$src/fab" "$dir/fab"
  echo "ok    $name: export refused, fab package untouched"
}

d=$(fresh)
if ! (cd "$d/gen" && ./export.sh >"$d/export.log" 2>&1); then
  cat "$d/export.log" >&2
  echo "FAIL  baseline export" >&2
  exit 1
fi
echo "ok    baseline export"

# The pedal boards use nested fab directories that must resolve from KiCad's working directory.
for spec in csc-r1p:main csc-r1p-ctl:controls; do
  board=${spec%:*}
  fab=${spec#*:}
  d=$(fresh)
  if ! (cd "$d/gen" && CSC_BOARD="$board" ./export.sh >"$d/export.log" 2>&1); then
    cat "$d/export.log" >&2
    echo "FAIL  $board export" >&2
    exit 1
  fi
  test -s "$d/pedal/fab/$fab/gerbers/$board-F_Cu.gtl"
  echo "ok    $board export"
done
# 1. A failing netlist checker.
d=$(fresh); printf 'import sys\nprint("mismatches: 1")\nsys.exit(1)\n' > "$d/gen/check_net.py"
expect_refused "failing netlist check" "$d" "mismatches: 1"

# 2. A pad on the wrong net (D1's pads swapped back the wrong way round).
d=$(fresh)
( cd "$d/gen" && python3 - <<'PY'
import pcbnew
b = pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
f = b.FindFootprintByReference('D1')
for p in f.Pads():
    p.SetNet(b.FindNet('GND' if p.GetNumber() == '1' else 'VUSB'))
assert pcbnew.SaveBoard('../kicad/csc-r1.kicad_pcb', b)
PY
)
expect_refused "pad/net mismatch" "$d" "net differs"

# 3. An unconnected net: delete every track on VUSB.
d=$(fresh)
( cd "$d/gen" && python3 - <<'PY'
import pcbnew
b = pcbnew.LoadBoard('../kicad/csc-r1.kicad_pcb')
tracks = [t for t in b.GetTracks() if t.GetNetname() == 'VUSB']
assert tracks, 'no VUSB tracks to remove'
for track in tracks:
    b.Remove(track)
assert pcbnew.SaveBoard('../kicad/csc-r1.kicad_pcb', b)
PY
)
expect_refused "unconnected net" "$d" "unconnected pads"

echo "all gate tests passed"
