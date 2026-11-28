# [M] Export latch meshes without zero-area triangles

## Goal

The outdoor latch coupon meshes that amsat-kit writes contain no zero-area triangles, the standalone latch STLs are watertight after coincident vertices merge, and both outdoor 3MF packages are regenerated from the fixed exporter. The fix lands on [PR 116](https://github.com/steelhead99x/cubesat-classroom/pull/116) (`fix/latch-coupon-box-label`).

## Plan

Review of PR 116 at `c9350c5` found 16,908 zero-area triangles across the latch box, lid and their accents in each outdoor 3MF, while the Bambu metadata reports zero degenerate facets. `coupon-latch-box.stl` and `coupon-latch-lid.stl` fail a watertightness check after merging coincident vertices.

1. Fetch PR 116 and read its latest commits first. Another session is working on this finding; if it has pushed a fix, verify it instead of writing a second one. Never force-push.
2. Add a check to `crates/amsat-kit/src/outdoor_checks.rs` (or the exporter's tests) that loads each serialized mesh, counts triangles whose area is below a small epsilon, and checks edge manifoldness after merging vertices within a tolerance. It must fail on `c9350c5`.
3. Fix the source of the degenerate triangles in the mesh or text code (`design.rs`, `text.rs`, `parts.rs` or `bambu.rs`), and make the 3MF metadata's degenerate-facet count come from the real mesh.
4. Regenerate `hardware/outdoor-station` and `hardware/outdoor-xiao` packages together, and confirm the new check passes on every latch STL and 3MF object.
5. Push to the PR 116 branch.
