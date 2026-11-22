# Continuous integration and deploy

GitHub Actions is disabled and replaced by PrairieDog (`pdog.bitprairie.com`, monitored in `config/repos.toml` under `steelhead99x/cubesat-classroom`).

## CI on PrairieDog runners

PrairieDog dispatches pull request and default-branch CI jobs to runners carrying `ci` and `x86_64` (the Alienware and MSI pool), which run `bash scripts/ci.sh`:

1. **Rust station:** `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`.
2. **Website and Worker (`site/`):** `npm ci`, `npm run build`, Worker checks (`npm test` when present, else worker module import), and `npm run build:github` with a clean `git status --porcelain -- docs` check.
3. **Mobile (`mobile/`):** `npm ci`, `npm run typecheck`, `npm run lint`.
4. **Installer and scripts:** `bash -n`, `shellcheck`, and `python3 -m py_compile`.

## Cloudflare Worker deploy (approval required)

`scripts/deploy-worker.sh` builds `site/` and runs `wrangler deploy`. It is **never** called from `scripts/ci.sh` and must not auto-deploy on push:

- Fails closed (`exit 1`) when `CLOUDFLARE_API_TOKEN` or `CLOUDFLARE_ACCOUNT_ID` is unset.
- Preserves `GUIDE_LIMITS_KV_ID` when provided without leaving `site/wrangler.jsonc` modified.
- Intended to run only after explicit owner approval (email approval to `kyle@hikd.ai` once PrairieDog M4 deploy approvals are live, or manual operator invocation).
