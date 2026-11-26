//! The kit's three joints, all printed as part of the plate or the end piece: nothing loose.
//!
//! - **Snap pin**, plate to frame: a pin with a 45 degree head on each column; two curved flex
//!   fingers in the plate round it, with teeth that spring in under the head.
//! - **Snap hook**, board to plate: a straight flex finger cut into the plate beside a board edge,
//!   with a post on its tip and a hook over the board's edge.
//! - **Fixed lip**, board to plate: a rigid post with a lip the board's opposite edge tucks under.
//!
//! A board goes in by tucking one edge under its lips and pressing the other edge down until the
//! hooks click. Every finger bends in the plate's own plane, along its printed strands.

use manifold_csg::{CrossSection, Manifold};

use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{EPS, FACETS, arc_band, bx, cone, cyl, rect, round_corners, uz_prism};

// ---------- snap pin ----------

/// Snap pin standing on a shoulder, for a part `part_t` thick. The head's 45 degree underside
/// starts `pin_bite` below the part's top face, so the teeth meet it there.
pub fn pin(d: &Design, x: f64, y: f64, z_shoulder: f64, part_t: f64) -> Manifold {
    let z_c = z_shoulder + part_t - d.pin_bite;
    let flank = d.pin_head_r - d.pin_r;
    let (r, hr) = (d.pin_r, d.pin_head_r);
    Manifold::batch_union(&[
        cyl(
            2.0 * r,
            z_c - z_shoulder + 2.0 * EPS,
            x,
            y,
            z_shoulder - EPS,
        ),
        cone(2.0 * r, 2.0 * hr, flank, x, y, z_c),
        cyl(2.0 * hr, d.pin_cap + 2.0 * EPS, x, y, z_c + flank - EPS),
        cone(
            2.0 * hr,
            2.0 * (hr - d.pin_lead),
            d.pin_lead,
            x,
            y,
            z_c + flank + d.pin_cap,
        ),
    ])
}

pub fn pin_top(d: &Design, z_shoulder: f64, part_t: f64) -> f64 {
    z_shoulder + part_t - d.pin_bite + (d.pin_head_r - d.pin_r) + d.pin_cap + d.pin_lead
}

pub fn locating_pin(d: &Design, x: f64, y: f64, z_shoulder: f64, part_t: f64) -> Manifold {
    let top = z_shoulder + part_t + d.locate_above;
    cyl(
        2.0 * d.pin_r,
        top - 0.4 - z_shoulder + 2.0 * EPS,
        x,
        y,
        z_shoulder - EPS,
    )
    .union(&cone(
        2.0 * d.pin_r,
        2.0 * d.pin_r - 0.8,
        0.4,
        x,
        y,
        top - 0.4,
    ))
}

/// Where the teeth meet the head's 45 degree face: at the part's top face.
pub fn contact_r(d: &Design) -> f64 {
    d.pin_r + d.pin_bite
}

/// Head clearance, finger outer radius, slit outer radius.
pub fn finger_radii(d: &Design) -> (f64, f64, f64) {
    let r_clear = d.pin_head_r + d.finger_clear;
    (
        r_clear,
        r_clear + d.finger_w,
        r_clear + d.finger_w + d.finger_slit,
    )
}

pub fn finger_length(d: &Design) -> f64 {
    let (r_clear, r_out, _) = finger_radii(d);
    (r_clear + r_out) / 2.0 * d.finger_span.to_radians()
}

/// How far each tooth springs out while the head passes.
pub fn finger_pass(d: &Design) -> f64 {
    d.pin_head_r + 0.1 - d.tooth_r
}

pub fn finger_strain(d: &Design) -> f64 {
    1.5 * d.finger_w * finger_pass(d) / finger_length(d).powi(2)
}

/// Pull to lift a part off one snap pin, N: both teeth pushed out to the head's full radius
/// along its 45 degree face, against friction. A straight-beam estimate for curved fingers.
pub fn pin_retention(d: &Design, part_t: f64) -> f64 {
    let k = 3.0 * d.pla_e * (part_t * d.finger_w.powi(3) / 12.0) / finger_length(d).powi(3);
    let radial = k * (d.pin_head_r - contact_r(d));
    2.0 * radial * (45f64.to_radians() + d.friction.atan()).tan()
}

/// Cut round a snap pin: the head's clearance, the slits that free two curved fingers and the
/// gaps at their tips, less the teeth. `tooth_r` places the teeth where the checks want them.
pub fn finger_cut(d: &Design, cx: f64, cy: f64, a0: f64, tooth_r: Option<f64>) -> CrossSection {
    let tooth_r = tooth_r.unwrap_or(d.tooth_r);
    let (r_clear, r_out, r_slit) = finger_radii(d);
    let mut cut = CrossSection::circle(r_clear, FACETS);
    let mut teeth = CrossSection::empty();
    for k in 0..2 {
        let root = a0 + 180.0 * f64::from(k);
        let tip = root + d.finger_span;
        cut = cut
            .union(&arc_band(r_out, r_slit, root, tip + d.finger_gap))
            .union(&arc_band(r_clear - EPS, r_slit, tip, tip + d.finger_gap));
        teeth = teeth.union(&arc_band(tooth_r, r_clear + EPS, tip - d.tooth_span, tip));
    }
    round_corners(&cut, 0.5)
        .difference(&teeth)
        .translate(cx, cy)
}

/// Room the fingers need: nothing else may stand on them.
pub fn finger_zone(d: &Design, cx: f64, cy: f64, z0: f64, h: f64) -> Manifold {
    cyl(2.0 * finger_radii(d).2 + 1.0, h, cx, cy, z0)
}

// ---------- board edges ----------

/// (angle, origin, s) for a joint on a board edge. Local u points out from the edge, v along it;
/// s is +1 if v runs the same way as `along`, -1 if opposite.
pub fn edge_frame(board: Rect, edge: &Edge) -> (f64, [f64; 2], f64) {
    let [x0, y0, w, h] = board;
    match edge.side {
        Side::Right => (0.0, [x0 + w, y0 + edge.along], 1.0),
        Side::Top => (90.0, [x0 + edge.along, y0 + h], -1.0),
        Side::Left => (180.0, [x0, y0 + edge.along], -1.0),
        Side::Bottom => (270.0, [x0 + edge.along, y0], 1.0),
    }
}

fn place(local: &Manifold, board: Rect, edge: &Edge, flip: bool) -> Manifold {
    let (angle, [x, y], _) = edge_frame(board, edge);
    let local = if flip {
        local.mirror([0.0, 1.0, 0.0])
    } else {
        local.clone()
    };
    local.rotate(0.0, 0.0, angle).translate(x, y, 0.0)
}

fn flipped(board: Rect, edge: &Edge) -> bool {
    f64::from(edge.root) * edge_frame(board, edge).2 < 0.0
}

// ---------- snap hook ----------

/// The finger that bends: 0.2 mm wider than its post.
fn hook_finger_w(d: &Design) -> f64 {
    d.hook_w + 0.2
}

/// Finger strain while the hook rides over the board edge.
pub fn hook_strain(d: &Design) -> f64 {
    1.5 * hook_finger_w(d) * (d.hook_reach + 0.1) / d.hook_len.powi(2)
}

/// Downward force on the board from one hook, N (bending only; no friction).
pub fn hook_preload_force(d: &Design) -> f64 {
    let k = 3.0 * d.pla_e * (d.plate_t * hook_finger_w(d).powi(3) / 12.0) / d.hook_len.powi(3);
    k * d.hook_preload / d.hook_angle.to_radians().tan()
}

/// (u, z) points of the hook: root underside, tip underside, tip top, and the post's top height.
fn hook_profile(d: &Design, board_top: f64) -> ([[f64; 2]; 3], f64) {
    let (g, tip) = (d.hook_gap, -d.hook_reach);
    let t = d.hook_angle.to_radians().tan();
    let z_tip_under = board_top + (-d.hook_preload - tip) * t;
    let z_root_under = board_top - (g + d.hook_preload) * t;
    let z_top = z_tip_under + d.hook_lip_t + (g - tip); // 45 degree lead-in on top
    (
        [
            [g, z_root_under],
            [tip, z_tip_under],
            [tip, z_tip_under + d.hook_lip_t],
        ],
        z_top,
    )
}

pub fn hook_top(d: &Design, board_top: f64) -> f64 {
    hook_profile(d, board_top).1
}

/// Post and hook on a finger's tip, in place. `deflect` bends the finger outward.
pub fn hook_post(d: &Design, board: Rect, edge: &Edge, board_top: f64, deflect: f64) -> Manifold {
    let ([root_under, tip_under, tip_over], z_top) = hook_profile(d, board_top);
    let g = d.hook_gap;
    let profile = [
        [g, d.plate_t - EPS],
        [g + d.hook_w, d.plate_t - EPS],
        [g + d.hook_w, z_top],
        [g, z_top],
        tip_over,
        tip_under,
        [g, root_under[1]],
    ];
    let local = uz_prism(&profile, d.hook_post_w).translate(deflect, 0.0, 0.0);
    place(&local, board, edge, flipped(board, edge))
}

/// The finger in local (u, v): 0.1 mm wider than its post all round, so no faces line up.
fn finger_span(d: &Design) -> (f64, f64, f64, f64) {
    let (g, w, pw) = (d.hook_gap, d.hook_w, d.hook_post_w);
    (
        g - 0.1,
        g + w + 0.1,
        -pw / 2.0 - 0.1,
        -pw / 2.0 - 0.1 + d.hook_len,
    )
}

/// The slits through the plate that free a hook's finger.
pub fn hook_cut(d: &Design, board: Rect, edge: &Edge) -> Manifold {
    let (u0, u1, v_tip, v_root) = finger_span(d);
    let v0 = v_tip - d.hook_tip_gap;
    let cut = rect(u0 - d.hook_slit_in, v0, u0, v_root)
        .union(&rect(u1, v0, u1 + d.hook_slit_out, v_root))
        .union(&rect(u0 - d.hook_slit_in, v0, u1 + d.hook_slit_out, v_tip));
    let local = round_corners(&cut, 0.5)
        .extrude(d.plate_t + 2.0)
        .translate(0.0, 0.0, -1.0);
    place(&local, board, edge, flipped(board, edge))
}

/// The finger and its slits, from z0 up h: nothing but its own post may stand there.
pub fn hook_zone(d: &Design, board: Rect, edge: &Edge, z0: f64, h: f64) -> Manifold {
    let (u0, u1, v_tip, v_root) = finger_span(d);
    let local = bx(
        u0 - d.hook_slit_in,
        v_tip - d.hook_tip_gap,
        z0,
        u1 + d.hook_slit_out,
        v_root,
        z0 + h,
    );
    place(&local, board, edge, flipped(board, edge))
}

// ---------- fixed lip ----------

/// Rigid post with a lip over the board's edge. The lip's underside is at 45 degrees, so it
/// prints without supports and guides the board's edge under it.
pub fn lip(d: &Design, board: Rect, edge: &Edge, board_top: f64) -> Manifold {
    let (g, reach) = (d.hook_gap, d.lip_reach);
    let under = |u: f64| board_top + d.lip_gap - u;
    let z_top = under(-reach) + d.lip_t;
    let profile = [
        [g, d.plate_t - EPS],
        [g + d.lip_post_t, d.plate_t - EPS],
        [g + d.lip_post_t, z_top],
        [-reach, z_top],
        [-reach, under(-reach)],
        [g, under(g)],
    ];
    place(&uz_prism(&profile, d.lip_w), board, edge, false)
}
