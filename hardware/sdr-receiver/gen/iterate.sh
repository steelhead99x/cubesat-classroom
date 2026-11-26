#!/bin/bash
# Try failed nets first on the next pass and preserve the best board in this run.
set -euo pipefail
cd "$(dirname "$0")"
run=$(mktemp -d)
trap 'rm -rf -- "$run"' EXIT
best=
for ((i=1; i<=${1:-8}; i++)); do
  result="$run/pass-$i.json"
  python3 route.py "$run/priority.json" "$result"
  n=$(cat "$result")
  if [[ ! "$n" =~ ^[0-9]+$ ]]; then
    echo "invalid routing result: $n" >&2
    exit 1
  fi
  if [[ -z "$best" || "$n" -lt "$best" ]]; then
    best=$n
    cp ../kicad/csc-r1.kicad_pcb "$run/best.kicad_pcb"
  fi
  if [[ "$n" -eq 0 ]]; then break; fi
done
if [[ -z "$best" ]]; then
  echo "no routing passes ran" >&2
  exit 1
fi
cp "$run/best.kicad_pcb" ../kicad/csc-r1.kicad_pcb
echo "best: $best failed"
[[ "$best" -eq 0 ]] || { echo "routing incomplete: $best nets failed" >&2; exit 1; }
