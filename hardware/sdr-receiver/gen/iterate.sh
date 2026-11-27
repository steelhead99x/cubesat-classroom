#!/bin/bash
# Try failed nets first on the next pass and preserve the best board in this run.
set -euo pipefail
cd "$(dirname "$0")"
PCB=$(python3 boardcfg.py pcb)
run=$(mktemp -d)
trap 'rm -rf -- "$run"' EXIT
best=
for ((i=1; i<=${1:-$(python3 boardcfg.py passes)}; i++)); do
  result="$run/pass-$i.json"
  python3 route.py "$run/priority.json" "$result"
  n=$(cat "$result")
  if [[ ! "$n" =~ ^[0-9]+$ ]]; then
    echo "invalid routing result: $n" >&2
    exit 1
  fi
  if [[ -z "$best" || "$n" -lt "$best" ]]; then
    best=$n
    cp "$PCB" "$run/best.kicad_pcb"; cp "$result.nets" "$run/best.nets.json"
  fi
  if [[ "$n" -eq 0 ]]; then break; fi
done
if [[ -z "$best" ]]; then
  echo "no routing passes ran" >&2
  exit 1
fi
cp "$run/best.kicad_pcb" "$PCB"
echo "best: $best failed"
# Finishing passes (pedal boards): keep the best board and route only its unfinished nets.
if [[ "$best" -gt 0 && "$(python3 boardcfg.py proj)" != csc-r1 ]]; then
  cp "$run/best.nets.json" "$run/finish.json"
  for ((j=1; j<=4 && best>0; j++)); do
    cp "$PCB" "$run/start.kicad_pcb"
    ROUTE_FINISH="$run/start.kicad_pcb" ROUTE_FINISH_NETS="$(cat "$run/finish.json")" \
      python3 route.py "$run/finish-priority.json" "$run/finish-result.json"
    cp "$run/finish-result.json.nets" "$run/finish.json"
    n=$(cat "$run/finish-result.json")
    echo "finishing pass $j: $n nets left"
    best=$n
  done
fi
[[ "$best" -eq 0 ]] || { echo "routing incomplete: $best nets failed" >&2; exit 1; }
