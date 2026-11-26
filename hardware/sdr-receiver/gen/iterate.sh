#!/bin/bash
# run routing passes until no failures (failed nets go first on the next pass); keep the best board
best=999
for i in $(seq 1 ${1:-8}); do
  python3 route.py 2>&1 | grep "^routed" | cut -c1-420
  n=$(cat /tmp/nfail.json)
  if [ "$n" -lt "$best" ]; then best=$n; cp ../kicad/csc-r1.kicad_pcb /tmp/best.kicad_pcb; fi
  [ "$n" -eq 0 ] && break
done
cp /tmp/best.kicad_pcb ../kicad/csc-r1.kicad_pcb; echo "best: $best failed"
[ "$best" -eq 0 ] || { echo "routing incomplete: $best nets failed" >&2; exit 1; }
