# [S] Patch the url advisory and add a dependency audit to the classroom Rust workspace

## Goal

The classroom Rust workspace (ground station, `crates/mission-budget`, `crates/downlink`) has no known-vulnerable dependency in `Cargo.lock`, and PrairieDog CI fails when a new advisory lands.

## Plan

1. Update `url` to `2.5.9` or newer in `Cargo.lock` (`cargo update -p url --precise 2.5.9`), clearing `RUSTSEC-2026-0592` (`Url::origin()` stack overflow on nested `blob:` URLs).
2. Add `deny.toml` modelled on `cubesat-classroom-rust`'s policy: explicit license allow-list covering the current tree, and reasoned ignores only for the two upstream unmaintained notices (`RUSTSEC-2026-0542` `humantime` via `moq-tokio`, and `RUSTSEC-2026-0543` `chrono` via `sgp4`).
3. Ensure `scripts/ci.sh` (PR #38) runs `cargo deny check` whenever `deny.toml` is present on PrairieDog CI runners.
4. Run `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`, `cargo deny check`, and `quest --root . check`.
5. Open a pull request with this quest file. Do not merge.

### How to check

- [ ] `cargo deny check` prints `advisories ok, bans ok, licenses ok, sources ok` and exits 0.
- [ ] `rg -A1 'name = "url"' Cargo.lock` shows `2.5.9` or newer.
- [ ] Clippy and all 87 workspace tests still pass.
- [ ] `quest --root . check` exits 0.
- [ ] The PR is open and was not merged automatically.
