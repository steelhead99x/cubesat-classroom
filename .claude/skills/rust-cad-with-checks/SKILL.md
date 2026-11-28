---
name: rust-cad-with-checks
description: Use when building or changing parametric 3D-print CAD in Rust (manifold-csg), as in crates/amsat-kit: every design rule as a check, a broken-design self-test, deterministic STL and 3MF output, and honestly labelled results.
---

# Rust CAD with checks

CubeSat Classroom's printed parts are code: Rust with the `manifold-csg` crate (Manifold, the same kernel as Python's manifold3d). The reference is `crates/amsat-kit` in the cubesat-classroom repository: `cargo run --release -p amsat-kit [check|self-test]`.

## Structure

- **`design`:** every dimension is one named field with a comment saying where it came from: a datasheet, a measured print, or an assumption. Variants are `Design` clones with changed fields.
- **`joints`:** each joint once: snap pin, lips and hooks, push-down hooks. Parts call them, so a fix lands everywhere.
- **`parts` / case modules:** build solids in one frame of reference; flip them to print orientation only when writing files.
- **`checks`:**
  - `rules` are hand calculations: strain, preload, retention vs launch load, stresses.
  - `verify` covers geometry: seated parts don't overlap; parts lifted 0.3 mm do overlap, so they're held; insertion paths are clear with flex parts deflected.
- **`broken_designs()`:** labelled designs that must each be rejected. **`boundary_designs()`:** designs exactly at a limit that must pass.
- **`main`:**
  - `build` verifies, reports the key numbers, and writes STLs (each a single closed solid) and the print files.
  - `check` runs the checks only.
  - `self-test` runs the broken and boundary designs.
- **Tests:** cargo tests run the same checks, so CI catches regressions.

## Every rule a check, every check able to fail

- After adding a check, add a broken design that only that check should catch.
- **Read the reason it was rejected.** If another check catches it first, change the broken design until the new check is the one that fires. For example, take the ports out to prove a lid-height check.
- If a "broken" design passes, either the check is wrong or the design wasn't really broken. Find out which before changing either.

## Pitfalls met so far

- **Overlaps:** union parts that touch with an EPS overlap (about 0.05 mm). Coplanar faces leave seams or split solids.
- **Degenerate inputs:** return `Manifold::empty()` for a degenerate box. A negative size poisons every later union.
- **Heights vs tops:** `cyl(d, h, x, y, z)` takes a height, not a top. Passing a top height made a boss poke 1 mm into a battery.
- **Sealed voids:** an internal void shows up as an extra negative-volume shell in `decompose()`. Open it (rib bays, vents) instead of sealing air in.
- **Hull for slopes:** `Manifold::batch_hull` of two convex extrusions gives a printable 45° slope under a flange.
- **Threads:** `Manifold::extrude_with_options` with twist = 360° × turns makes a thread from its cross-section. Positive twist is a right-hand thread. Probe it: the crest's angle should rise 90° per quarter pitch going up.
- **Insertion paths:** check by moving the part along its path (lift steps, tilt steps about its lip edge, with the right sign) against the features with the flex parts deflected.
- **Fit searches:** for an orientation or position search, loop over candidates against the same rules the checks use, then pick the middle of the valid range.

## Output

- **Deterministic output:** same input, same bytes. Format numbers with fixed decimals, keep zip timestamps fixed, write files in a fixed order. Check it by running twice and comparing hashes. CI fails if committed files differ.
- **Every STL** is a closed single solid, standing on z = 0, in print orientation.
- **Check the files as written,** not the solid: read each STL back and each 3MF volume at its six decimals, merge equal corners, and fail on any collapsed or zero-area triangle or unpaired edge. Booleans leave coincident vertices that an index-based closed check misses; `Mesh::from_solid` simplifies, snaps to a grid and welds them away.
- **Report the numbers** a reviewer needs: strain, preload, retention vs load, volumes.

## Honesty

- Assumptions (board thickness, header height, material strength) are written as assumptions, with the coupon that will check them.
- Results are labelled design check, desk simulation or hardware bench. A passing check is not a measurement.
