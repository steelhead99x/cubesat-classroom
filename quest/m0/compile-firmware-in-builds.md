# [M] Compile the firmware in every build

## Goal

Every build that checks a pull request also compiles the firmware, so a firmware change that doesn't build can't look green. That means the Rust Nano 33 BLE Sense firmware (`firmware/nano33-cubesat`) for `thumbv7em-none-eabihf`, and the Arduino sketches for each board in the CI matrix. PrairieDog runs it as well as GitHub CI, and so can anyone on their own computer.

## Plan

### Why this matters

The Rust firmware is its own Cargo project (`exclude` in the root `Cargo.toml`), so the workspace checks never compile it. GitHub CI has a `firmware-rust` job and an Arduino `firmware` matrix in `.github/workflows/ci.yml`, but GitHub Actions hasn't run since 22 November (the billing problem). PrairieDog only runs the host workspace. The Rev C review fixes in `66a94df` (the I2C RAM buffer, the HS3003 maths, the USB zero-length packet) went into PR #37 without anyone compiling them for the board.

### Progress

Steps 1 and 3 are done: `scripts/firmware.sh` exists, and PrairieDog runs it from `scripts/ci.sh` (the list lives there; `.prairiedog.toml` points at it). The script doesn't add the Rust target itself; it stops and prints the `rustup target add` command, so the "target removed" check below still holds. Steps 2 and 4 to 6 are still open.

### Steps

1. Add `scripts/firmware.sh`. It runs the same commands as the `firmware-rust` job: `rustup target add thumbv7em-none-eabihf`, then `cargo fmt --check`, `cargo clippy --locked --release -- -D warnings` and `cargo build --locked --release` in `firmware/nano33-cubesat`. It then compiles each Arduino sketch with `arduino-cli compile`, using the boards, cores and libraries from the CI `firmware` matrix. If a toolchain is missing it stops with an error. It never skips a board quietly.
2. Point the `firmware-rust` job in `ci.yml` at `bash scripts/firmware.sh --rust`, so GitHub CI and the script can't drift apart. Keep the Arduino matrix, or switch it to `--arduino <fqbn>`, whichever keeps one board per job.
3. Find out where PrairieDog reads its build commands. The [Rev C quest](/quest/m0/rev-c-printed-frame.md) names `scripts/ci.sh`, but that file isn't in the repository yet. Add `scripts/firmware.sh` to that list, and write down in this quest where the list lives.
4. Prove that a broken build fails. On a scratch branch, remove the `TWIM_TX.init([0; 16])` argument in `firmware/nano33-cubesat/src/main.rs` and push it. The firmware step must go red. Delete the branch afterwards.
5. Run the firmware build on PR #37's head commit and post the result on the PR. That closes the "not compiled" note from the review.
6. Once it builds, update the **Status** line in `firmware/nano33-cubesat/README.md` to name the commit that was compiled. It still says "not run on a board", which stays true until someone flashes one.

### How to check

- [ ] `bash scripts/firmware.sh` exits 0 on a clean checkout with the Rust target and `arduino-cli` installed.
- [ ] With the `thumbv7em-none-eabihf` target removed, the script exits non-zero and names the missing target.
- [ ] `shellcheck scripts/firmware.sh` passes. The CI `scripts` job already runs shellcheck on `scripts/*.sh`.
- [ ] The scratch branch from step 4 turned PrairieDog's build red, and this quest records the link.
- [ ] PR #37 has a comment showing `nano33-cubesat` built in release mode, with the commit it built.
- [ ] `quest --root . check` exits 0.
- [ ] The pull request is open and was not merged automatically.

## Required

- [Add the Rev C printed 1U frame and Nano 33 firmware, in Rust](/quest/m0/rev-c-printed-frame.md) - the firmware this quest compiles arrives with that quest.
