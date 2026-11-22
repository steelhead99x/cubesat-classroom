# [S] Replace GitHub Actions workflows with PrairieDog CI and approval-gated deploy

## Goal

All CI checks run via `scripts/ci.sh` on PrairieDog runners (`ci` + `x86_64`), Worker deployment lives in fail-closed `scripts/deploy-worker.sh` (never auto-run on push), and `.github/workflows/` is removed.

## Plan

1. Verify `scripts/ci.sh` runs Rust station (`fmt`, `clippy`, `test`), Website/Worker (`build`, Worker check, `build:github` clean `docs/` check), Mobile (`typecheck`, `lint`), and installer/script checks (`bash -n`, `shellcheck`, `py_compile`).
2. Verify `scripts/deploy-worker.sh` exits non-zero when `CLOUDFLARE_API_TOKEN` or `CLOUDFLARE_ACCOUNT_ID` is unset, and is not invoked by `scripts/ci.sh`.
3. Confirm `.github/workflows/ci.yml` and `.github/workflows/deploy-worker.yml` are removed and `docs/CI.md` documents the PrairieDog runner and deploy-approval flow.

### How to check

- [ ] `bash scripts/ci.sh` exits 0.
- [ ] `bash scripts/deploy-worker.sh` without Cloudflare secrets exits non-zero with a clear error.
- [ ] `.github/workflows/ci.yml` and `.github/workflows/deploy-worker.yml` do not exist.
- [ ] A pull request is open and has not been merged automatically.
