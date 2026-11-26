# [M] Add the Rev C printed 1U frame and Nano 33 firmware, in Rust

## Goal

The classroom kit has a CDS-checked Rev C printed 1U frame (`crates/cubesat-cad`, `hardware/rev-c/`) and Arduino Nano 33 BLE Sense firmware (`firmware/nano33-cubesat`), with the host Rust workspace green on PrairieDog CI and `url` at 2.5.9 or newer.

## Plan

1. Keep the Rev C CAD, CDS checks, and print files in `crates/cubesat-cad` and `hardware/rev-c/`, and the Nano 33 firmware in `firmware/nano33-cubesat`.
2. Run `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, and `cargo test --locked --workspace` so `crates/cubesat-cad` and `crates/cubesat-mission` are covered by the same workspace checks PrairieDog runs in `scripts/ci.sh`.
3. Keep `url` in `Cargo.lock` at `2.5.9` or newer so `RUSTSEC-2026-0592` stays clear.
4. Run `quest --root . check`.

### How to check

- [ ] `rg -A1 '^name = "url"' Cargo.lock` shows `2.5.9` or newer.
- [ ] `cargo fmt --all --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, and `cargo test --locked --workspace` exit 0.
- [ ] `quest --root . check` exits 0.
- [ ] The pull request is open and was not merged automatically.
