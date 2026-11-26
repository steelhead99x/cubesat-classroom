//! Small solid-modeling helpers on top of the Manifold kernel. Dimensions in mm.

use manifold_csg::{CrossSection, Manifold};

/// Segments for a full circle. Fine enough for 3 mm holes, cheap enough for fast checks.
pub const FACETS: i32 = 48;
/// Overlap between stacked features so unions fuse into one solid instead of touching.
pub const EPS: f64 = 0.05;

/// Axis-aligned box from two opposite corners. Empty if any side has no length,
/// so a parameter change that squeezes a feature away leaves nothing, not an error.
pub fn cuboid(min: [f64; 3], max: [f64; 3]) -> Manifold {
    if (0..3).any(|i| max[i] <= min[i]) {
        return Manifold::empty();
    }
    Manifold::cube(max[0] - min[0], max[1] - min[1], max[2] - min[2], false)
        .translate(min[0], min[1], min[2])
}

/// Cylinder along +Z from `z`, optionally tapered to `d_top`.
pub fn cylinder(d: f64, h: f64, at: [f64; 3], d_top: Option<f64>) -> Manifold {
    Manifold::cylinder(h, d / 2.0, d_top.unwrap_or(d) / 2.0, FACETS, false)
        .translate(at[0], at[1], at[2])
}

/// Prism from a 2D outline in XY, extruded along +Z from `z`.
pub fn prism(outline: &[[f64; 2]], z: f64, h: f64) -> Manifold {
    Manifold::extrude(&CrossSection::from_simple_polygon(outline), h).translate(0.0, 0.0, z)
}

/// Rectangle in XY with every corner rounded to `r`, extruded along +Z.
pub fn rounded_rect(min: [f64; 2], max: [f64; 2], r: f64, z: f64, h: f64) -> Manifold {
    let corners = [
        [min[0] + r, min[1] + r],
        [max[0] - r, min[1] + r],
        [max[0] - r, max[1] - r],
        [min[0] + r, max[1] - r],
    ];
    let discs: Vec<Manifold> = corners
        .iter()
        .map(|c| cylinder(2.0 * r, h, [c[0], c[1], z], None))
        .collect();
    Manifold::batch_hull(&discs)
}

/// Union of many solids in one batch (faster than chaining).
pub fn union_all(parts: &[Manifold]) -> Manifold {
    Manifold::batch_union(parts)
}

/// A hole along Z for a part that prints with Z lying flat: a circle with a
/// 45 degree point towards `up` (a direction in XY: the way up on the print bed),
/// so its roof prints without supports.
pub fn teardrop_hole_z(d: f64, length: f64, center: [f64; 3], up: [f64; 2]) -> Manifold {
    let r = d / 2.0;
    let mut pts: Vec<[f64; 2]> = (0..FACETS)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * f64::from(i) / f64::from(FACETS);
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    let n = up[0].hypot(up[1]);
    let tip = r * std::f64::consts::SQRT_2 / n;
    pts.push([up[0] * tip, up[1] * tip]);
    Manifold::extrude(&CrossSection::hull_simple_polygon(&pts), length).translate(
        center[0],
        center[1],
        center[2] - length / 2.0,
    )
}

/// Bounding box as (min, max).
pub fn bounds(m: &Manifold) -> ([f64; 3], [f64; 3]) {
    let b = m.bounding_box().expect("bounding box of a non-empty solid");
    (b.min(), b.max())
}

/// Volume shared by two solids.
pub fn overlap(a: &Manifold, b: &Manifold) -> f64 {
    a.intersection(b).volume()
}

/// A cone that widens the root of a post standing on a face at `z`, pointing
/// `dir` (+1 or -1 in Z): the post's diameter `d` at the tip, `d + 2c` at the root.
pub fn root_chamfer(d: f64, c: f64, at: [f64; 2], z: f64, dir: f64) -> Manifold {
    let h = c + EPS;
    if dir > 0.0 {
        cylinder(d + 2.0 * c, h, [at[0], at[1], z - EPS], Some(d))
    } else {
        cylinder(d, h, [at[0], at[1], z - c], Some(d + 2.0 * c))
    }
}
