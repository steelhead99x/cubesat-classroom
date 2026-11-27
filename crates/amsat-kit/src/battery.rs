//! The 3000 mAh battery in the classroom kit: a keyed plate that snaps onto a second set of
//! columns on the AMSAT top piece. Press the cell straight down between two side guides until a
//! snap hook clicks over each end.
//!
//! The cell is sized from MakerFocus's listing, 67 x 36 x 12 mm +-2 mm, and the plate holds
//! every cell in that range. Each hook's post stands on a long finger in the plate that springs
//! towards the cell: at rest the posts sit inside the shortest cell's ends, so every cell
//! pushes them apart and is held between them, with each hook's catch over its end. A thinner
//! cell can lift until it meets the catches (up to 4 mm), but can't come out.

use manifold_csg::Manifold;

use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{EPS, FACETS, bx, offset, rect, union_all};
use crate::joints::{hook_cut, hook_post, hook_zone};
use crate::parts::{cable_notch, column_cuts, isogrid_pockets, plate_outline};

/// One hook over each end of the cell. The wire leaves the top end beside its hook.
pub const CELL_HOOKS: [Edge; 2] = [
    Edge::new(Side::Bottom, 12.0, 1),
    Edge::new(Side::Top, 26.0, -1),
];
/// Side guides: how tall, how thick, and the gap to the cell.
const GUIDE_H: f64 = 6.0;
const GUIDE_T: f64 = 2.0;
pub const GUIDE_GAP: f64 = 0.5;

/// How far each post sits inside the shortest cell's end at rest: the least squeeze a cell gets.
pub const END_PRELOAD: f64 = 0.3;

/// The board joint, scaled for a cell: thicker posts, wider lips, and a much longer finger, so
/// the post can move out by the whole length range and then clear the catch.
pub fn joint(d: &Design) -> Design {
    let mut j = d.clone();
    j.hook_w = 2.4;
    j.hook_post_w = 8.0;
    j.hook_len = 33.0;
    // The post's face touches the cell's end, so the catch starts right at it.
    j.hook_gap = 0.0;
    j.hook_preload = 0.0;
    j.lip_w = 10.0;
    j.lip_post_t = 3.0;
    j
}

/// Lengths (y) of the shortest and longest cells the listing allows.
pub fn cell_lengths(d: &Design) -> (f64, f64) {
    (d.cell[0] - d.cell_tol, d.cell[0] + d.cell_tol)
}

/// Where the hooks are placed: the rectangle whose ends are the posts' faces at rest,
/// `END_PRELOAD` inside the shortest cell's ends.
pub fn hook_rect(d: &Design) -> Rect {
    let [x0, y0, w, h] = d.cell_rect();
    let s = d.cell_tol + END_PRELOAD;
    [x0, y0 + s, w, h - 2.0 * s]
}

/// How far each post is pushed out by a seated cell of length `len` (centred between them).
pub fn seated_deflect(d: &Design, len: f64) -> f64 {
    (len - cell_lengths(d).0) / 2.0 + END_PRELOAD
}

/// Finger strain as the longest cell goes in: pushed out by the seated deflection plus the
/// catch's reach.
pub fn hook_strain(d: &Design) -> f64 {
    let j = joint(d);
    let travel = seated_deflect(d, cell_lengths(d).1) + j.hook_reach + 0.1;
    1.5 * (j.hook_w + 0.2) * travel / j.hook_len.powi(2)
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

/// A cell of the given size, lying on the plate, centred along its length between the hooks
/// and against the low-x guide (`dx` moves it across).
pub fn cell_sized(d: &Design, len: f64, width: f64, thick: f64, dx: f64) -> Manifold {
    let [x0, y0, _, h] = d.cell_rect();
    let y = y0 + (h - len) / 2.0;
    bx(
        x0 + dx,
        y,
        d.plate_t,
        x0 + dx + width,
        y + len,
        d.plate_t + thick,
    )
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
    let r = hook_rect(d);
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
            .map(|e| hook_cut(&j, hook_rect(d), e))
            .collect::<Vec<_>>(),
    )
}

pub fn hook_zones(d: &Design, z0: f64, h: f64) -> Vec<Manifold> {
    let j = joint(d);
    CELL_HOOKS
        .iter()
        .map(|e| hook_zone(&j, hook_rect(d), e, z0, h))
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
    // Room for the logo with the words, where the top face is open.
    let free = outline
        .offset(-3.0, manifold_csg::JoinType::Miter, 2.0, 0)
        .difference(&offset(
            &cuts.slice_to_cross_section(d.plate_t / 2.0),
            2.5,
            FACETS,
        ))
        .difference(&offset(
            &feats.slice_to_cross_section(d.plate_t + 0.1),
            2.5,
            FACETS,
        ));
    let keep = keep.union(&crate::brand::pad(d, &free)?);
    let windows = isogrid_pockets(d, &outline, &keep);
    let slab = outline
        .extrude(d.plate_t)
        .difference(&cuts)
        .difference(&windows);
    Ok(slab.union(&features(d, deflect)))
}
