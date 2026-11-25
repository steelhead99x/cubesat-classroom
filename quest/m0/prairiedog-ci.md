# [S] Run cubesat-classroom CI on PrairieDog runners

## Goal

`steelhead99x/cubesat-classroom` runs its Rust, site, Worker, mobile, script, and quest checks through PrairieDog CI (`scripts/ci.sh` and `.prairiedog.toml`) instead of disabled GitHub Actions workflows.

## Plan

1. Add `.prairiedog.toml` at the repository root pointing `ci_command` at `bash scripts/ci.sh` on `["ci", "x86_64"]` runners.
2. Add `scripts/ci.sh` covering the Rust workspace (`cargo fmt --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`), `cargo deny check` when `deny.toml` is present, shell/Python script checks, `site/` build + Worker tests + `docs/` sync, `mobile/` typecheck + lint, and `quest --root . check`.
3. Run `bash scripts/ci.sh` locally and verify PrairieDog CI runs cleanly on the open pull request.

### How to check

- [ ] `bash scripts/ci.sh` exits 0.
- [ ] `shellcheck docs/start.sh scripts/*.sh` exits 0.
- [ ] `quest --root . check` prints `documents ok` and exits 0.
- [ ] `.prairiedog.toml` configures `ci_command = "bash scripts/ci.sh"` on `["ci", "x86_64"]`.
- [ ] The pull request is open and was not merged automatically.
