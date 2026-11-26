//! Uno snap latches with lock keys, and quarter-turn fasteners with printed spring
//! washers. A port of the AMSAT kit's `generate.py`, with the same geometry, so both
//! builds share one design. Everything here is in payload-plate coordinates (plate
//! underside at z = 0, board side up) unless it says otherwise.
//!
//! The quarter-turn fastener is the classic push-and-turn panel fastener of aircraft
//! and spacecraft, described generically. A stud with a screwdriver slot has a spiral
//! cam slot across its tip. Push it past a cross-bar and turn it 90 degrees clockwise:
//! the cam draws it in against the spring washer, then drops the bar into a detent.

use manifold_csg::{CrossSection, JoinType, Manifold};

use crate::design::{Design, Edge};
use crate::geom::{EPS, cuboid, cylinder};

/// PLA Young's modulus for the spring and latch forces, MPa.
pub const PLA_E: f64 = 3500.0;

/// Polygon with counter-clockwise winding, whichever way the points were listed.
pub fn poly(points: &[[f64; 2]]) -> CrossSection {
    let n = points.len();
    let area: f64 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if area > 0.0 {
        CrossSection::from_polygons(&[points.to_vec()])
    } else {
        CrossSection::from_polygons(&[points.iter().rev().copied().collect()])
    }
}

/// Extrude a (u, z) profile across v = -width/2 .. width/2.
fn uz_prism(points: &[[f64; 2]], width: f64) -> Manifold {
    poly(points)
        .extrude(width)
        .rotate(90.0, 0.0, 0.0)
        .translate(0.0, width / 2.0, 0.0)
}

fn bx(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Manifold {
    cuboid([x0, y0, z0], [x1, y1, z1])
}

fn cyl(d: f64, h: f64, x: f64, y: f64, z: f64) -> Manifold {
    cylinder(d, h, [x, y, z], None)
}

// ---------- Uno latches and keys ----------

/// Arm length from its root in the pocket to the board's top face.
pub fn latch_arm(d: &Design) -> f64 {
    d.uno_top() - (d.plate_t - d.latch.root_depth)
}

/// Bending strain at the arm root as the hook rides over the board edge.
pub fn latch_strain(d: &Design) -> f64 {
    1.5 * d.latch.t * d.latch.reach / latch_arm(d).powi(2)
}

/// Bending strain in a key prong as its bump passes the notch.
pub fn key_strain(d: &Design) -> f64 {
    let w = (d.key.w - d.key.slot) / 2.0;
    1.5 * w * d.key.bump / d.key.prong.powi(2)
}

/// Downward force on the board from one latch, newtons (no friction).
pub fn latch_force(d: &Design) -> f64 {
    let l = &d.latch;
    let stiffness = 3.0 * PLA_E * (l.w * l.t.powi(3) / 12.0) / latch_arm(d).powi(3);
    stiffness * l.preload / l.angle.to_radians().tan()
}

/// Latch-local frame: (rotation about Z in degrees, origin on the board edge).
/// Local u points out from the board edge, v along it.
fn latch_frame(d: &Design, i: usize) -> (f64, [f64; 2]) {
    let (edge, along, _) = d.latch.places[i];
    let [ux, uy] = d.uno_origin();
    match edge {
        Edge::Right => (0.0, [ux + d.uno[0], uy + along]),
        Edge::Top => (90.0, [ux + along, uy + d.uno[1]]),
        Edge::Left => (180.0, [ux, uy + along]),
        Edge::Bottom => (270.0, [ux + along, uy]),
    }
}

fn place(d: &Design, i: usize, solid: &Manifold) -> Manifold {
    let (angle, [x, y]) = latch_frame(d, i);
    solid.rotate(0.0, 0.0, angle).translate(x, y, 0.0)
}

/// u of the arm's back (seated), the key's inner face, and the backstop's front.
fn key_slot(d: &Design) -> (f64, f64, f64) {
    let back = d.latch.gap + d.latch.t + d.latch.preload;
    (back, back + d.key.clear, back + 2.0 * d.key.clear + d.key.t)
}

/// Width of a latch's backstop, between the outer faces of its cheeks.
fn backstop_w(d: &Design) -> f64 {
    d.key.w + 2.0 * 0.15 + 2.0 * d.key.cheek_t
}

pub fn latch_top_z(d: &Design) -> f64 {
    let l = &d.latch;
    let tip = -l.reach;
    let under_tip = d.uno_top() + (-l.preload - tip) * l.angle.to_radians().tan();
    under_tip + l.lip_t + (l.gap - tip)
}

/// Room in the plate for the lower end of an arm to flex.
pub fn latch_pocket(d: &Design, i: usize) -> Manifold {
    let l = &d.latch;
    let floor = d.plate_t - l.root_depth;
    let u1 = l.gap + l.t + l.reach + 0.2;
    place(
        d,
        i,
        &bx(
            l.gap - 1.2,
            -l.w / 2.0 - 0.4,
            floor,
            u1,
            l.w / 2.0 + 0.4,
            d.plate_t + EPS,
        ),
    )
}

/// Plate under a latch's footprint, so latches near the plate edge stand on an ear.
pub fn latch_ear(d: &Design, i: usize) -> Manifold {
    let (_, _, front) = key_slot(d);
    let reach = front + d.key.back_t + d.latch.places[i].2 + 1.0;
    let half = backstop_w(d) / 2.0 + 1.0;
    place(d, i, &bx(-1.0, -half, 0.0, reach, half, d.plate_t))
}

/// One latch. `deflect` bends the arm outward (approximated as a shift of the hook),
/// for checking the seated position and the snap-in path.
pub fn latch(d: &Design, i: usize, deflect: f64) -> Manifold {
    let (l, k) = (&d.latch, &d.key);
    let top = d.plate_t;
    let t = l.angle.to_radians().tan();
    let tip = -l.reach;
    let z_tip_under = d.uno_top() + (-l.preload - tip) * t;
    let z_root_under = d.uno_top() - (l.gap + l.preload) * t;
    let z_top = latch_top_z(d);
    let floor = top - l.root_depth;
    let arm = bx(
        l.gap,
        -l.w / 2.0,
        floor - EPS,
        l.gap + l.t,
        l.w / 2.0,
        z_top,
    );
    let hook = uz_prism(
        &[
            [l.gap + EPS, z_root_under],
            [tip, z_tip_under],
            [tip, z_tip_under + l.lip_t],
            [l.gap + EPS, z_top],
        ],
        l.w,
    );
    // A small root fillet on the board side.
    let root = uz_prism(
        &[
            [l.gap + EPS, floor - EPS],
            [l.gap - 1.0, floor - EPS],
            [l.gap + EPS, floor + 1.0],
        ],
        l.w,
    );
    let mut moving = arm.union(&hook);
    if deflect != 0.0 {
        moving = moving.translate(deflect, 0.0, 0.0);
    }
    let (_, _, front) = key_slot(d);
    let cheek_u = l.gap + l.t + l.reach + 0.2; // clear of the arm while it snaps
    let bw = backstop_w(d);
    let mut fixed = bx(
        front,
        -bw / 2.0,
        top - EPS,
        front + k.back_t,
        bw / 2.0,
        z_top,
    );
    for side in [-1.0, 1.0] {
        let (a, b) = (side * (k.w / 2.0 + 0.15), side * bw / 2.0);
        let cheek = bx(cheek_u, a.min(b), top - EPS, front + EPS, a.max(b), z_top);
        let n = 0.9 / std::f64::consts::SQRT_2;
        let notch = Manifold::cube(front - cheek_u + 2.0, n, n, true)
            .rotate(45.0, 0.0, 0.0)
            .translate(
                (cheek_u + front) / 2.0,
                side * (k.w / 2.0 + 0.15),
                top + k.notch_z,
            );
        fixed = fixed.union(&cheek.difference(&notch));
    }
    let gusset = l.places[i].2;
    if gusset > 0.0 {
        for v in [-bw / 2.0 + 0.8, bw / 2.0 - 0.8] {
            let g = uz_prism(
                &[
                    [front + k.back_t - EPS, top - EPS],
                    [front + k.back_t + gusset, top - EPS],
                    [front + k.back_t - EPS, z_top - 1.0],
                ],
                1.6,
            );
            fixed = fixed.union(&g.translate(0.0, v, 0.0));
        }
    }
    place(d, i, &moving.union(&root).union(&fixed))
}

/// Lock key as printed: lying flat, prongs down the page. x across, y along, z thickness.
pub fn key_flat(d: &Design) -> Manifold {
    let k = &d.key;
    let length = latch_top_z(d) - d.plate_t + k.tab;
    let (w, b) = (k.w / 2.0, k.bump);
    let mut body = CrossSection::square(k.w, length - k.tab + 0.6, false)
        .translate(-w, 0.0)
        .difference(&CrossSection::square(k.slot, k.prong, false).translate(-k.slot / 2.0, -EPS));
    for s in [-1.0, 1.0] {
        body = body.union(&poly(&[
            [s * w, k.notch_z - b],
            [s * (w + b), k.notch_z],
            [s * w, k.notch_z + b],
        ]));
        body = body.difference(&poly(&[
            [s * (w + 1.0), -1.0],
            [s * (w - 0.5), -1.0],
            [s * (w + 1.0), 0.5],
        ]));
    }
    let tab = CrossSection::square(k.w + 2.6, k.tab, false)
        .translate(-w - 1.3, length - k.tab)
        .offset(-0.8, JoinType::Round, 2.0, 24)
        .offset(0.8, JoinType::Round, 2.0, 24);
    body.union(&tab)
        .difference(&CrossSection::circle(1.3, 24).translate(0.0, length - k.tab / 2.0))
        .extrude(k.t)
}

/// The key seated in latch `i`, bottom on the plate.
pub fn key_in_place(d: &Design, i: usize) -> Manifold {
    let (_, inner, _) = key_slot(d);
    // Key x -> v, key y -> height above the plate, key z -> u from the key's inner face.
    let m = [
        0.0, 1.0, 0.0, // x basis
        0.0, 0.0, 1.0, // y basis
        1.0, 0.0, 0.0, // z basis
        inner, 0.0, d.plate_t,
    ];
    place(d, i, &key_flat(d).transform(&m))
}

// ---------- quarter-turn fasteners ----------

/// How far the cam has drawn the stud in at `phi` degrees of turn.
pub fn qt_drop(d: &Design, phi: f64) -> f64 {
    let q = &d.qt;
    if phi <= 75.0 {
        q.lift * phi / 75.0
    } else if phi <= 82.0 {
        q.lift + q.det * (phi - 75.0) / 7.0
    } else {
        q.lift + q.det * (90.0 - phi) / 8.0
    }
}

/// Shank length that leaves `tip` of stud behind the bar when it's locked.
pub fn qt_stud_length(d: &Design, a0: f64) -> f64 {
    a0 + d.qt.bar_h + d.qt.tip
}

/// The bar as it sits relative to the stud: at depth `a` from the head, turned `theta`.
fn qt_bar_box(d: &Design, a: f64, theta: f64) -> Manifold {
    let q = &d.qt;
    let w = (q.bar_w + q.clear) / 2.0;
    let l = q.d + 2.0;
    bx(-l, -w, a - q.clear, l, w, a + q.bar_h).rotate(0.0, 0.0, theta)
}

/// Stud as printed: head down on the bed, shank up. z = 0 is the underside of the
/// head, and `a0` is the bar's distance from it before turning (washer at rest).
pub fn qt_stud(d: &Design, a0: f64) -> Manifold {
    let q = &d.qt;
    let length = qt_stud_length(d, a0);
    let head = cyl(q.head_d, q.head_h, 0.0, 0.0, -q.head_h).difference(&bx(
        -q.head_d,
        -0.8,
        -q.head_h - 1.0,
        q.head_d,
        0.8,
        -q.head_h + 1.2,
    ));
    let chamfer = cyl(q.d + 2.0, 0.6, 0.0, 0.0, length - 0.6).difference(&cylinder(
        q.d + 0.1,
        0.6 + EPS,
        [0.0, 0.0, length - 0.6 - EPS / 2.0],
        Some(q.d - 1.0),
    ));
    let shank = cyl(q.d, length + EPS, 0.0, 0.0, -EPS).difference(&chamfer);
    let w = (q.bar_w + q.clear) / 2.0;
    let mut slot = qt_bar_box(d, a0, 0.0).union(&bx(-q.d, -w, a0 - q.clear, q.d, w, length + 1.0));
    let mut steps: Vec<f64> = (0..=18).map(|i| f64::from(i) * 5.0).collect();
    steps.extend([78.0, 82.0, 86.0]);
    steps.sort_by(f64::total_cmp);
    steps.dedup();
    let mut prev: Option<Manifold> = None;
    for phi in steps {
        let b = qt_bar_box(d, a0 - qt_drop(d, phi), -phi);
        slot = match &prev {
            None => slot.union(&b),
            Some(p) => slot.union(&Manifold::batch_hull(&[p.clone(), b.clone()])),
        };
        prev = Some(b);
    }
    head.union(&shank).difference(&slot)
}

/// Receptacle cross-bar in a hole of diameter `bore`, its near face `bar_z` inside the
/// entry face at `z_face`. `downward`: the stud comes in from above.
pub fn qt_bar(d: &Design, x: f64, y: f64, z_face: f64, downward: bool) -> Manifold {
    let q = &d.qt;
    let z0 = if downward {
        z_face - q.bar_z - q.bar_h
    } else {
        z_face + q.bar_z
    };
    bx(
        -q.bore / 2.0 - 0.5,
        -q.bar_w / 2.0,
        z0,
        q.bore / 2.0 + 0.5,
        q.bar_w / 2.0,
        z0 + q.bar_h,
    )
    .translate(x, y, 0.0)
}

/// A stud in place: head underside at `z_head` before turning, turned `phi` and drawn in.
pub fn placed_stud(d: &Design, a0: f64, x: f64, y: f64, z_head: f64, phi: f64) -> Manifold {
    qt_stud(d, a0)
        .rotate(0.0, 0.0, phi)
        .mirror([0.0, 0.0, 1.0])
        .translate(x, y, z_head - qt_drop(d, phi))
}

fn arc_band(r0: f64, r1: f64, a0: f64, a1: f64, n: u32) -> CrossSection {
    let at = |r: f64, a: f64| [r * a.to_radians().cos(), r * a.to_radians().sin()];
    let nf = f64::from(n);
    let mut pts: Vec<[f64; 2]> = (0..=n)
        .map(|i| at(r1, a0 + (a1 - a0) * f64::from(i) / nf))
        .collect();
    pts.extend((0..=n).map(|i| at(r0, a1 - (a1 - a0) * f64::from(i) / nf)));
    poly(&pts)
}

/// Hub outer radius, arm inner and outer radii, ring inner radius.
pub fn washer_radii(d: &Design) -> (f64, f64, f64, f64) {
    let w = &d.washer;
    let hub = w.id / 2.0 + 1.0;
    let arm0 = hub + w.slit;
    let arm1 = arm0 + w.arm;
    (hub, arm0, arm1, arm1 + w.slit)
}

pub fn washer_arm_length(d: &Design) -> f64 {
    let (_, arm0, arm1, _) = washer_radii(d);
    (arm0 + arm1) / 2.0 * d.washer.sweep.to_radians()
}

/// Spring washer as printed: flat, feet up. Fit it feet down under the stud head.
pub fn qt_washer(d: &Design) -> Manifold {
    let w = &d.washer;
    let (hub, arm0, arm1, ring0) = washer_radii(d);
    let mut flat = arc_band(w.id / 2.0, hub, 0.0, 360.0, 48).union(&arc_band(
        ring0,
        w.od / 2.0,
        0.0,
        360.0,
        48,
    ));
    for k in [0.0, 180.0] {
        flat = flat
            .union(&arc_band(arm0, arm1, k, k + w.sweep, 48))
            .union(&arc_band(hub - 0.1, arm0 + 0.1, k, k + 12.0, 48))
            .union(&arc_band(
                arm1 - 0.1,
                ring0 + 0.1,
                k + w.sweep - 12.0,
                k + w.sweep,
                48,
            ));
    }
    let mut washer = flat.simplify(0.02).extrude(w.t);
    for k in [90.0, 210.0, 330.0] {
        washer = washer.union(
            &arc_band(ring0 + 0.3, w.od / 2.0 - 0.3, k - 6.0, k + 6.0, 8)
                .extrude(w.foot + EPS)
                .translate(0.0, 0.0, w.t - EPS),
        );
    }
    washer
}

/// Washer compression at the detent crest: cam draw-in, detent, and the Nano's lift.
pub fn washer_crest(d: &Design) -> f64 {
    d.qt.lift + d.qt.det + d.nano.preload
}

/// Arm bending strain (guided-end beam) for a hub travel of `deflection`.
pub fn washer_strain(d: &Design, deflection: f64) -> f64 {
    3.0 * d.washer.t * deflection / washer_arm_length(d).powi(2)
}

/// Newtons from both arms for a hub travel of `deflection`.
pub fn washer_force(d: &Design, deflection: f64) -> f64 {
    let l = washer_arm_length(d);
    2.0 * 12.0 * PLA_E * (d.washer.arm * d.washer.t.powi(3) / 12.0) / l.powi(3) * deflection
}

/// Bending stress in a receptacle bar at the detent crest, MPa. The bar is fused into
/// the hole wall at both ends: fixed-fixed beam, central load, M = F L / 8.
pub fn bar_stress(d: &Design) -> f64 {
    let q = &d.qt;
    washer_force(d, washer_crest(d)) * q.bore / 8.0 / (q.bar_w * q.bar_h.powi(2) / 6.0)
}

/// Plate hold-down from the four column studs, and what the payload needs at `design_g`.
pub fn plate_hold_down(d: &Design) -> (f64, f64) {
    (
        4.0 * washer_force(d, d.qt.lift),
        d.payload_mass * 9.81 * d.design_g,
    )
}
