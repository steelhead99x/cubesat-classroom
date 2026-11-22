#!/usr/bin/env bash
# Deploy the Cloudflare Worker for cubesat-classroom (cubesatclass.org, cubesat.club,
# and hub domains). Replaces .github/workflows/deploy-worker.yml.
#
# Never call this script from scripts/ci.sh or auto-run it on push.
# Deploys must only run after explicit owner approval (PrairieDog deploy approval
# email to kyle@hikd.ai, or manual operator run with secrets loaded).
#
# Required environment variables:
#   CLOUDFLARE_API_TOKEN   API token with Edit Cloudflare Workers permissions
#   CLOUDFLARE_ACCOUNT_ID  Cloudflare account id owning cubesat-classroom
#
# Optional environment variables:
#   GUIDE_LIMITS_KV_ID     KV namespace id bound as GUIDE_LIMITS
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ -x /usr/local/bin/node ]; then
  export PATH="/usr/local/bin:$PATH"
fi

if [ -z "${CLOUDFLARE_API_TOKEN:-}" ] || [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
  echo "error: CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID must both be set; Worker was NOT deployed." >&2
  exit 1
fi

cd "$ROOT/site"
npm ci
npm run build

WRANGLER_CFG="$ROOT/site/wrangler.jsonc"
BACKUP_CFG=""
cleanup() {
  if [ -n "$BACKUP_CFG" ] && [ -f "$BACKUP_CFG" ]; then
    mv "$BACKUP_CFG" "$WRANGLER_CFG"
  fi
}
trap cleanup EXIT

if [ -n "${GUIDE_LIMITS_KV_ID:-}" ]; then
  BACKUP_CFG="$(mktemp)"
  cp "$WRANGLER_CFG" "$BACKUP_CFG"
  node --input-type=module -e "
    import { readFileSync, writeFileSync } from 'node:fs';
    const text = readFileSync('wrangler.jsonc', 'utf8').replace(/^\s*\/\/.*$/gm, '');
    const config = JSON.parse(text);
    config.kv_namespaces = [{ binding: 'GUIDE_LIMITS', id: process.env.GUIDE_LIMITS_KV_ID }];
    writeFileSync('wrangler.jsonc', JSON.stringify(config, null, 2));
  "
else
  echo "notice: GUIDE_LIMITS_KV_ID is unset; deploying without GUIDE_LIMITS binding (prewritten guide answers only)."
fi

npx --yes wrangler@4.166.0 deploy
