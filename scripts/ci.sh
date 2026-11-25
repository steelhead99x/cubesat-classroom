#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

pick_node24() {
  local candidate
  for candidate in "$(command -v node 2>/dev/null || true)" /workspace/node24/bin/node "$HOME"/.local/node/*/bin/node; do
    [ -x "$candidate" ] || continue
    local ver major minor patch
    ver="$("$candidate" -v 2>/dev/null | sed 's/^v//')"
    IFS='.' read -r major minor patch <<< "${ver%%-*}"
    if [ "${major:-0}" -gt 22 ] || { [ "${major:-0}" -eq 22 ] && [ "${minor:-0}" -gt 23 ]; } || { [ "${major:-0}" -eq 22 ] && [ "${minor:-0}" -eq 23 ] && [ "${patch:-0}" -ge 4 ]; }; then
      local node_dir
      node_dir="$(dirname "$candidate")"
      export PATH="$node_dir:$PATH"
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

echo "=== Scripts and Python tools ==="
bash -n docs/start.sh scripts/*.sh
if command -v shellcheck >/dev/null 2>&1; then
  shellcheck docs/start.sh scripts/*.sh
else
  echo "note: shellcheck not installed; skipping shellcheck"
fi
python3 -m py_compile scripts/*.py tools/*.py hardware/*.py hardware/p1s/*.py
python3 tools/test_serial_image.py

echo "=== Site and Worker ==="
if ! pick_node24; then
  echo "error: Node.js >= 22.23.4 is required for site/ and mobile/ (found: $(node -v 2>/dev/null || echo none))" >&2
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
if [ -n "$(git status --porcelain -- docs)" ]; then
  git status --short -- docs
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
if command -v quest >/dev/null 2>&1; then
  quest --root . check
else
  echo "note: quest CLI not installed; skipping quest check"
fi
