#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

check_node_version() {
  local candidate="$1"
  [ -x "$candidate" ] || return 1
  local ver major minor patch
  ver="$("$candidate" -v 2>/dev/null | sed 's/^v//')"
  IFS='.' read -r major minor patch <<< "${ver%%-*}"
  if [ "${major:-0}" -gt 22 ] || { [ "${major:-0}" -eq 22 ] && [ "${minor:-0}" -gt 23 ]; } || { [ "${major:-0}" -eq 22 ] && [ "${minor:-0}" -eq 23 ] && [ "${patch:-0}" -ge 4 ]; }; then
    local node_dir
    node_dir="$(dirname "$candidate")"
    export PATH="$node_dir:$PATH"
    return 0
  fi
  return 1
}

pick_node24() {
  local current_node
  if current_node="$(command -v node 2>/dev/null)"; then
    if check_node_version "$current_node"; then
      return 0
    fi
  fi
  local candidate
  for candidate in /workspace/node24/bin/node "$HOME"/.local/node/*/bin/node; do
    if check_node_version "$candidate"; then
      return 0
    fi
  done
  return 1
}

echo "=== Rust station (fmt, clippy, tests) ==="
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace

if [ -f deny.toml ]; then
  if command -v cargo-deny >/dev/null 2>&1; then
    echo "=== Rust advisories and licenses (cargo-deny) ==="
    cargo deny check
  else
    echo "error: deny.toml is present but cargo-deny is not installed" >&2
    exit 1
  fi
fi

echo "=== Shell scripts and Python checks ==="
bash -n docs/start.sh scripts/*.sh
command -v shellcheck >/dev/null 2>&1 || { echo "error: shellcheck is required" >&2; exit 1; }
shellcheck docs/start.sh scripts/*.sh
(cd hardware/solar-face/gen && python3 test_drc.py)
python3 hardware/rideshare-flatsat/test_lab.py
python3 hardware/rideshare-flatsat-ground/test_ground.py

echo "=== Firmware builds ==="
bash scripts/firmware.sh

echo "=== Committed CAD and print files are up to date ==="
cargo run --locked --release -p cubesat-cad -- archive hardware
cargo run --locked --release -p amsat-kit -- self-test
cargo run --locked --release -p amsat-kit
cargo clippy --locked --release -p cubesat-cad --all-targets -- -D warnings
cargo test --locked --release -p cubesat-cad
cargo run --locked --release -p cubesat-cad -- build hardware/rev-c
hardware_status="$(git status --porcelain -- hardware)"
if [ -n "$hardware_status" ]; then
  printf '%s\n' "$hardware_status"
  echo "Regenerate the hardware files (see hardware/README.md) and commit them." >&2
  exit 1
fi

echo "=== Site and Worker ==="
if ! pick_node24; then
  echo "error: Node.js >= 22.23.4 is required for site/ and mobile/" >&2
  exit 1
fi
echo "Using node $(node -v)"
(
  cd site
  npm ci
  npm run build
  npm test
  npm run build:github
)
docs_status="$(git status --porcelain -- docs)"
if [ -n "$docs_status" ]; then
  printf '%s\n' "$docs_status"
  echo "Run 'npm run build:github' in site/ and commit docs/." >&2
  exit 1
fi

echo "=== Mobile app ==="
(
  cd mobile
  npm ci
  npm run typecheck
  npm run lint
)

echo "=== Quest check ==="
command -v quest >/dev/null 2>&1 || { echo "error: quest binary not found (set QUEST_BIN or install quest 0.23.1)" >&2; exit 1; }
quest --root . check
