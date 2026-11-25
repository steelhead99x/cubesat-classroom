# mission-budget

Design-review budgets for CubeSat Classroom. **Work in progress.** These are planning numbers for a classroom review, not measurements or flight predictions.

| Module | CLI | Replaces |
| --- | --- | --- |
| `link` | `cargo run --locked -- link-budget hardware/rf-example.json [--altitude-km 550] [--elevation-deg 15] [--radial-speed-km-s 7] [--json]` | `scripts/link_budget.py` |
| `energy` | `cargo run --locked -- energy-budget [--json]` | `tools/energy_budget.py` |

## Link budget

This is a first-order, one-way downlink sensitivity exercise:

- slant range on a spherical Earth (R = 6371 km)
- free-space path loss
- EIRP = PA − feed loss + TX gain
- C/N0 = EIRP + G/T − FSPL − other losses − k (k = −228.599 dBW/K/Hz)
- Eb/N0 = C/N0 − 10·log10(bit rate)
- margin = Eb/N0 − required Eb/N0

The Doppler column comes from a separately assumed radial speed, not from the geometry. The model leaves out the uplink, interference, antenna patterns, real pass profiles and duty cycle. The values in `hardware/rf-example.json` are invented for sensitivity studies. They are not a channel plan: any transmission needs a licensed operator and the applicable spectrum authorization. Amateur use stays open, with no encryption (FCC Part 97).

## Energy budget

The budget covers lithium AA packs: 3000 mAh at 1.4 V per cell, with 85 % regulator efficiency (an estimate), a 25 % cold reserve (an estimate) and a 2× run-time margin. Every load names its source. Replace the estimates with your own USB power-meter readings, then edit `src/energy.rs`.

## Tests

`cargo test -p mission-budget` runs unit tests and golden tests. The golden tests compare the output byte for byte with what the original Python tools printed (`tests/golden/`). The crate forbids unsafe code and holds itself to clippy's pedantic lints (see `Cargo.toml`).
