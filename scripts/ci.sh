#!/usr/bin/env bash
# Repository CI entrypoint for PrairieDog runners (ci + x86_64 pool).
# Replaces .github/workflows/ci.yml. Never deploys; deploys run only via
# scripts/deploy-worker.sh after explicit owner approval.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Ensure Node >= 22.23.4 is first on PATH when /usr/local/bin carries Node 24.
if [ -x /usr/local/bin/node ]; then
  export PATH="/usr/local/bin:$PATH"
fi

echo "==> [1/4] Rust station (fmt, clippy, test)"
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked

echo "==> [2/4] Website and Worker (site/)"
(
  cd "$ROOT/site"
  npm ci
  npm run build
  if node -e "const p=require('./package.json'); process.exit(p.scripts && p.scripts.test ? 0 : 1)"; then
    npm test
  else
    node --input-type=module -e "await import('./src/worker.mjs')"
  fi
  npm run build:github
)
if ! git diff --exit-code --quiet -- docs || [ -n "$(git ls-files --others --exclude-standard -- docs)" ]; then
  git status --short -- docs
  echo "Run 'npm run build:github' in site/ and commit docs/." >&2
  exit 1
fi

echo "==> [3/4] Mobile app (mobile/)"
if [ "${SKIP_MOBILE:-0}" = "1" ]; then
  echo "SKIP_MOBILE=1 set; skipping mobile typecheck and lint."
else
  (
    cd "$ROOT/mobile"
    npm ci
    npm run typecheck
    npm run lint
  )
fi

echo "==> [4/4] Installer and scripts"
bash -n docs/start.sh scripts/*.sh
shellcheck docs/start.sh scripts/*.sh
python3 -m py_compile scripts/*.py tools/*.py hardware/*.py hardware/p1s/*.py

echo "cubesat-classroom CI passed."
