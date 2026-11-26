//! The kit's three joints, all printed as part of the plate or the end piece: nothing loose.
//!
//! - **Snap pin**, plate to frame: a pin with two flats and a 45 degree head on each column. The
//!   plate has a keyed slot round it: rigid walls on the flats, and two straight flex fingers
//!   with teeth that spring in under the head.
//! - **Snap hook**, board to plate: a straight flex finger cut into the plate beside a board edge,
//!   with a post on its tip and a hook over the board's edge.
//! - **Fixed lip**, board to plate: a rigid post with a lip the board's opposite edge tucks under.
//!
//! A board goes in by tucking one edge under its lips and pressing the other edge down until the
//! hooks click. Every finger bends in the plate's own plane, along its printed strands.

use manifold_csg::{CrossSection, Manifold};

use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{EPS, FACETS, bx, cone, cyl, rect, round_corners, uz_prism};

// ---------- snap pin ----------
//
// Local frame of one snap pin: the pin at the origin, its two lobes (the round sides that carry
// the catch) facing +x and -x, its two flats facing +y and -y. In the plate, two straight flex
// fingers run along x = +-xi from their roots at -y up to their tips beside the head, each with a
// tooth facing in. Rigid walls of the plate face the flats with a small clearance, so the plate
// drops onto the pin one way only, centred, and can't turn. `angle` turns the whole joint.

/// The slot's half-width: the fingers' inner faces.
fn slot_x(d: &Design) -> f64 {
    d.pin_head_r + d.finger_clear
}

/// The rigid walls facing the flats, for a fit clearance.
fn wall_y(d: &Design, fit: f64) -> f64 {
    d.pin_flat + fit
}

/// The fingers' tips: just past their teeth, so the joint reaches little past the pin.
fn tip_y(d: &Design) -> f64 {
    d.tooth_half + 0.4
}

/// Height of the head's lead-in, from its full radius up to its tip.
fn lead_h(d: &Design) -> f64 {
    (d.pin_head_r - d.pin_tip_r) / d.pin_lead_angle.to_radians().tan()
}

fn place_pin(local: Manifold, x: f64, y: f64, angle: f64) -> Manifold {
    local.rotate(0.0, 0.0, angle).translate(x, y, 0.0)
}

/// Snap pin standing on a shoulder, for a part `part_t` thick. The head's 45 degree underside
/// starts `pin_bite` below the part's top face, so the teeth meet it there. The top is a gentle
/// lead-in cone, and two flats are cut down the whole pin.
pub fn pin(d: &Design, x: f64, y: f64, angle: f64, z_shoulder: f64, part_t: f64) -> Manifold {
    let z_c = z_shoulder + part_t - d.pin_bite;
    let flank = d.pin_head_r - d.pin_r;
    let (r, hr) = (d.pin_r, d.pin_head_r);
    let round = Manifold::batch_union(&[
        cyl(
            2.0 * r,
            z_c - z_shoulder + 2.0 * EPS,
            0.0,
            0.0,
            z_shoulder - EPS,
        ),
        cone(2.0 * r, 2.0 * hr, flank, 0.0, 0.0, z_c),
        cyl(2.0 * hr, d.pin_cap + 2.0 * EPS, 0.0, 0.0, z_c + flank - EPS),
        cone(
            2.0 * hr,
            2.0 * d.pin_tip_r,
            lead_h(d),
            0.0,
            0.0,
            z_c + flank + d.pin_cap,
        ),
    ]);
    let flats = bx(
        -10.0,
        -d.pin_flat,
        z_shoulder - 2.0 * EPS,
        10.0,
        d.pin_flat,
        z_c + 20.0,
    );
    place_pin(round.intersection(&flats), x, y, angle)
}

pub fn pin_top(d: &Design, z_shoulder: f64, part_t: f64) -> f64 {
    z_shoulder + part_t - d.pin_bite + (d.pin_head_r - d.pin_r) + d.pin_cap + lead_h(d)
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

/// Radius of the head's 45 degree face at the part's top face, where the teeth meet it.
pub fn contact_r(d: &Design) -> f64 {
    d.pin_r + d.pin_bite
}

/// The fingers' length, root to tip.
pub fn finger_length(d: &Design) -> f64 {
    d.finger_len
}

/// How far each finger bends to let the head pass: the tooth's face, shaped to the head at rest
/// plus preload, has to clear the head (plus 0.1 mm) all along its length.
pub fn finger_pass(d: &Design) -> f64 {
    let y = d.tooth_half;
    let preload = contact_r(d) - d.tooth_r;
    let head = d.pin_head_r + 0.1;
    (head * head - y * y).sqrt() - (contact_r(d).powi(2) - y * y).sqrt() + preload
}

pub fn finger_strain(d: &Design) -> f64 {
    1.5 * d.finger_w * finger_pass(d) / finger_length(d).powi(2)
}

/// Pull to lift a part off one snap pin, N: both teeth pushed out to the head's full radius
/// along its 45 degree face, against friction. A straight-beam estimate.
pub fn pin_retention(d: &Design, part_t: f64) -> f64 {
    let k = 3.0 * d.pla_e * (part_t * d.finger_w.powi(3) / 12.0) / finger_length(d).powi(3);
    let radial = k * (d.pin_head_r - contact_r(d));
    2.0 * radial * (45f64.to_radians() + d.friction.atan()).tan()
}

/// Cut through a part round a snap pin: the keyed slot, the slits that free the two fingers,
/// less the teeth. `tooth_r` is where each tooth's face crosses the lobe axis (`d.tooth_r` at
/// rest; the checks move it to model bent fingers). `fit` is the clearance on the flats.
pub fn finger_cut(
    d: &Design,
    cx: f64,
    cy: f64,
    angle: f64,
    tooth_r: f64,
    fit: f64,
) -> CrossSection {
    let (xi, w, s, si) = (slot_x(d), d.finger_w, d.finger_slit, d.finger_slit_in);
    let top = wall_y(d, fit);
    let tip = tip_y(d);
    let root = tip - d.finger_len;
    let g = d.finger_tip_gap;
    let mut cut = rect(-xi, -top, xi, top);
    for sx in [1.0, -1.0] {
        let band = |a: f64, b: f64, y0: f64, y1: f64| {
            let (u, v) = (sx * a, sx * b);
            rect(u.min(v), y0, u.max(v), y1)
        };
        cut = cut
            .union(&band(xi + w, xi + w + s, root, tip + g))
            .union(&band(xi - EPS, xi + w + s, tip, tip + g))
            .union(&band(xi - si, xi + EPS, root, -top + EPS));
    }
    let cut = round_corners(&cut, 0.3);
    // Each tooth's face is the head's contact circle, offset so it crosses the axis at tooth_r:
    // with the plate seated, it meets the 45 degree face along its whole length.
    let mut teeth = CrossSection::empty();
    let c = contact_r(d);
    for sx in [1.0, -1.0] {
        let face = CrossSection::circle(c, FACETS * 2).translate(sx * (tooth_r - c), 0.0);
        let x0 = if sx > 0.0 { 0.0 } else { -xi - EPS };
        let reach = rect(x0, -d.tooth_half, x0 + xi + EPS, d.tooth_half);
        teeth = teeth.union(&reach.difference(&face));
    }
    cut.difference(&teeth).rotate(angle).translate(cx, cy)
}

/// Room the fingers need: nothing else may stand on them.
pub fn finger_zone(
    d: &Design,
    cx: f64,
    cy: f64,
    angle: f64,
    fit: f64,
    z0: f64,
    h: f64,
) -> Manifold {
    let (tip, top) = (tip_y(d), wall_y(d, fit));
    let x = slot_x(d) + d.finger_w + d.finger_slit + 0.5;
    let y1 = (tip + d.finger_tip_gap).max(top) + 0.5;
    let local = bx(-x, tip - d.finger_len - 0.5, z0, x, y1, z0 + h);
    place_pin(local, cx, cy, angle)
}

/// The joint's cut at the kit's fit: its slits must stay inside the part's outline.
pub fn finger_outline(d: &Design, cx: f64, cy: f64, angle: f64, fit: f64) -> CrossSection {
    finger_cut(d, cx, cy, angle, d.tooth_r, fit)
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
