//! The 3000 mAh battery in the classroom kit: a keyed plate that snaps onto a second set of
//! columns on the AMSAT top piece. Press the cell straight down between two side guides until a
//! snap hook clicks over each end: the boards' hook, with a sturdier post because it stands the
//! full height of the cell.
//!
//! The cell is sized from MakerFocus's listing, 67 x 36 x 12 mm +-2 mm, and every check uses
//! the largest cell that allows. A smaller cell sits loose by up to 2 mm each way: a strip of
//! foam tape under it takes that up.

use manifold_csg::{CrossSection, Manifold};

use crate::design::{Design, Edge, Side};
use crate::geom::{EPS, FACETS, bx, offset, rect, union_all};
use crate::joints::{hook_cut, hook_post, hook_zone};
use crate::parts::{cable_notch, column_cuts, plate_outline};

/// One hook over each end of the cell. The wire leaves the top end beside its hook.
pub const CELL_HOOKS: [Edge; 2] = [
    Edge::new(Side::Bottom, 12.0, 1),
    Edge::new(Side::Top, 26.0, -1),
];
/// Side guides: how tall, how thick, and the gap to the cell.
const GUIDE_H: f64 = 6.0;
const GUIDE_T: f64 = 2.0;
pub const GUIDE_GAP: f64 = 0.5;

/// The board joint, scaled for a cell: thicker posts, wider lips.
pub fn joint(d: &Design) -> Design {
    let mut j = d.clone();
    j.hook_w = 2.4;
    j.hook_post_w = 8.0;
    j.lip_w = 10.0;
    j.lip_post_t = 3.0;
    j
}

/// The top of the largest cell, lying on the plate.
pub fn cell_top(d: &Design) -> f64 {
    d.plate_t + d.cell_thick()
}

/// The largest cell, on the plate (plate coordinates).
pub fn cell(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.cell_rect();
    bx(x0, y0, d.plate_t, x0 + w, y0 + h, cell_top(d))
}

/// Guides along both long sides of the cell.
fn guides(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.cell_rect();
    let (z0, z1) = (d.plate_t - EPS, d.plate_t + GUIDE_H);
    union_all(&[
        bx(
            x0 - GUIDE_GAP - GUIDE_T,
            y0 + 4.0,
            z0,
            x0 - GUIDE_GAP,
            y0 + h - 4.0,
            z1,
        ),
        bx(
            x0 + w + GUIDE_GAP,
            y0 + 4.0,
            z0,
            x0 + w + GUIDE_GAP + GUIDE_T,
            y0 + h - 4.0,
            z1,
        ),
    ])
}

/// Hooks and side guides. `deflect` bends the hooks outward, for the checks.
pub fn features(d: &Design, deflect: f64) -> Manifold {
    let j = joint(d);
    let r = d.cell_rect();
    let mut parts = vec![guides(d)];
    for e in &CELL_HOOKS {
        parts.push(hook_post(&j, r, e, cell_top(d), deflect));
    }
    union_all(&parts)
}

pub fn hook_cuts(d: &Design) -> Manifold {
    let j = joint(d);
    union_all(
        &CELL_HOOKS
            .iter()
            .map(|e| hook_cut(&j, d.cell_rect(), e))
            .collect::<Vec<_>>(),
    )
}

pub fn hook_zones(d: &Design, z0: f64, h: f64) -> Vec<Manifold> {
    let j = joint(d);
    CELL_HOOKS
        .iter()
        .map(|e| hook_zone(&j, d.cell_rect(), e, z0, h))
        .collect()
}

/// The battery plate: the payload plate's outline and column cuts, opened up everywhere the
/// cell, the joints and the branding don't need plate.
pub fn battery_plate(d: &Design, tooth_r: Option<f64>, deflect: f64) -> Result<Manifold, String> {
    let outline = plate_outline(d);
    let [x0, y0, w, h] = d.cell_rect();
    let cuts = column_cuts(d, tooth_r)
        .union(&hook_cuts(d))
        .union(&cable_notch(d));
    let feats = features(d, 0.0);
    let keep = outline
        .difference(&outline.offset(-3.0, manifold_csg::JoinType::Miter, 2.0, 0))
        .union(&offset(
            &cuts.slice_to_cross_section(d.plate_t / 2.0),
            2.5,
            FACETS,
        ))
        .union(&offset(
            &feats.slice_to_cross_section(d.plate_t + 0.1),
            2.5,
            FACETS,
        ))
        // Rails under the cell's long edges and a rib across its middle carry it.
        .union(&rect(x0 + 2.0, y0, x0 + 8.0, y0 + h))
        .union(&rect(x0 + w - 8.0, y0, x0 + w - 2.0, y0 + h))
        .union(&rect(x0, y0 + h / 2.0 - 3.0, x0 + w, y0 + h / 2.0 + 3.0));
    let windows = offset(&offset(&outline.difference(&keep), -1.0, 16), 1.0, 16);
    let big: Vec<Vec<[f64; 2]>> = windows
        .to_polygons()
        .into_iter()
        .filter(|p| {
            CrossSection::from_polygons(std::slice::from_ref(p))
                .area()
                .abs()
                > 12.0
        })
        .collect();
    let slab = outline.extrude(d.plate_t).difference(&cuts).difference(
        &CrossSection::from_polygons(&big)
            .extrude(d.plate_t + 2.0)
            .translate(0.0, 0.0, -1.0),
    );
    Ok(slab.union(&features(d, deflect)))
}
