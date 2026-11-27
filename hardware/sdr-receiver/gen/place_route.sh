#!/bin/bash
# Placement, overlap check and routing for one board, stopping at the first failure.
set -euo pipefail
cd "$(dirname "$0")"
python3 "$(python3 boardcfg.py placer)" | tail -1
python3 ovl.py | tail -1
./iterate.sh "$@"
