//! Rev C parts, modeled where they sit in the assembled CubeSat (CDS coordinates).
//! `export` turns each one into its print orientation.

use manifold_csg::{CrossSection, Manifold};

use crate::design::Design;
use crate::geom::{
    EPS, cuboid, cylinder, prism, root_chamfer, rounded_rect, teardrop_hole_z, union_all,
};

/// A split snap peg standing on `base`, pointing +Z, long enough to pass through
/// `through` mm of material. Returns the solid and the slot to cut afterwards.
fn peg_up(d: &Design, base: [f64; 3], through: f64) -> (Manifold, Manifold) {
    let [x, y, z] = base;
    let shank = cylinder(d.peg_shank, through + 0.2 + EPS, [x, y, z - EPS], None);
    let z_barb = z + through + 0.2;
    let barb = cylinder(d.peg_barb, d.peg_lip + EPS, [x, y, z_barb - EPS], None).union(&cylinder(
        d.peg_barb,
        d.peg_lead + EPS,
        [x, y, z_barb + d.peg_lip - EPS],
        Some(d.peg_tip),
    ));
    let top = z_barb + d.peg_lip + d.peg_lead;
    let span = d.peg_barb + 2.0;
    let slot = cuboid(
        [x - span / 2.0, y - d.peg_slot / 2.0, z - d.slot_into_body],
        [x + span / 2.0, y + d.peg_slot / 2.0, top + 1.0],
    );
    (shank.union(&barb), slot)
}

/// How far a peg reaches above its base, through `through` mm.
pub fn peg_height(d: &Design, through: f64) -> f64 {
    through + 0.2 + d.peg_lip + d.peg_lead
}

/// Side panel at +Y: the web, two corner rails, and an end wall (lug) at each
/// end with two teardrop snap holes. The -Y panel is the same part turned 180
/// degrees about Z.
pub fn side_panel(d: &Design) -> Manifold {
    let (h, hz) = (d.half(), d.half_z());
    let ri = d.rail_inner();
    // The web stops short of the ends like the end plates, so only the rail ends touch at ±Z (CDS §2.2.8).
    let web_z = hz - d.end_recess;
    // The web's outer face sits `face_recess` inside the rail plane, so only the rails
    // can touch the deployer walls.
    let web = cuboid(
        [-(h - d.rail / 2.0), d.web_inner_y(), -web_z],
        [h - d.rail / 2.0, d.face_outer(), web_z],
    );
    let rails = [
        rounded_rect([ri, ri], [h, h], d.rail_radius, -hz, d.length),
        rounded_rect([-h, ri], [-ri, h], d.rail_radius, -hz, d.length),
    ];
    let mut parts = vec![web];
    parts.extend(rails);
    for sign in [1.0, -1.0] {
        let (z0, z1) = lug_z(d, sign);
        parts.push(cuboid(
            [-ri - EPS, d.lug_inner_y, z0],
            [ri + EPS, d.web_inner_y() + EPS, z1],
        ));
    }
    let mut panel = union_all(&parts);
    let window = cuboid(
        [
            -d.window[0] / 2.0,
            d.web_inner_y() - 1.0,
            -d.window[1] / 2.0,
        ],
        [d.window[0] / 2.0, h + 1.0, d.window[1] / 2.0],
    );
    panel = panel.difference(&window);
    for sign in [1.0, -1.0] {
        let (z0, z1) = lug_z(d, sign);
        for x in [-d.peg_x, d.peg_x] {
            // The panel prints outer face down, so "up" is towards the centre (-Y).
            panel = panel.difference(&teardrop_hole_z(
                d.snap_hole,
                d.lug_t + 2.0,
                [x, d.peg_y, (z0 + z1) / 2.0],
                [0.0, -1.0],
            ));
        }
    }
    panel
}

/// Z range of the end wall at the +Z (`sign` = 1) or -Z end.
pub fn lug_z(d: &Design, sign: f64) -> (f64, f64) {
    let outer = d.lug_outer_z();
    let inner = outer - d.lug_t;
    if sign > 0.0 {
        (inner, outer)
    } else {
        (-outer, -inner)
    }
}

/// Cover at +X: closes the face between the rails, with a port for USB and the
/// inhibit jumper, vent slots, and an end wall at each end with two snap holes.
/// The -X cover is the same part turned 180 degrees about Z.
pub fn cover(d: &Design) -> Manifold {
    let hz = d.half_z();
    let edge = d.rail_inner() - d.end_gap;
    let web_z = hz - d.end_recess;
    let (inner, outer) = (d.web_inner_y(), d.face_outer());
    let mut parts = vec![cuboid([inner, -edge, -web_z], [outer, edge, web_z])];
    for sign in [1.0, -1.0] {
        let (z0, z1) = lug_z(d, sign);
        parts.push(cuboid(
            [d.lug_inner_y, -d.cover_lug_half, z0],
            [inner + EPS, d.cover_lug_half, z1],
        ));
    }
    let mut cover = union_all(&parts);
    let (py, pz) = (d.port[0] / 2.0, d.port[1] / 2.0);
    cover = cover.difference(&cuboid(
        [inner - 1.0, -py, d.port_center_z - pz],
        [outer + 1.0, py, d.port_center_z + pz],
    ));
    for y in d.vent_ys() {
        cover = cover.difference(&cuboid(
            [
                inner - 1.0,
                y - d.vent[0] / 2.0,
                d.vent_center_z - d.vent[1] / 2.0,
            ],
            [
                outer + 1.0,
                y + d.vent[0] / 2.0,
                d.vent_center_z + d.vent[1] / 2.0,
            ],
        ));
    }
    for sign in [1.0, -1.0] {
        let (z0, z1) = lug_z(d, sign);
        for [x, y] in d.cover_peg_points() {
            if x < 0.0 {
                continue;
            }
            // The cover prints outer face down, so "up" is towards the centre (-X).
            cover = cover.difference(&teardrop_hole_z(
                d.snap_hole,
                d.lug_t + 2.0,
                [x, y, (z0 + z1) / 2.0],
                [-1.0, 0.0],
            ));
        }
    }
    cover
}

/// End plate outline: a square notched at each corner for the rails, reaching out
/// to the side panels in Y and to the covers in X.
fn end_outline(d: &Design, z: f64, h: f64) -> Manifold {
    let a = d.rail_inner() - d.end_gap;
    let b = d.web_inner_y() - d.end_gap;
    prism(
        &[
            [-a, -b],
            [a, -b],
            [a, -a],
            [b, -a],
            [b, a],
            [a, a],
            [a, b],
            [-a, b],
            [-a, a],
            [-b, a],
            [-b, -a],
            [-a, -a],
        ],
        z,
        h,
    )
}

/// End plate at +Z with eight bosses and snap pegs pointing towards the centre:
/// four for the side panels, four for the covers.
fn end_plate_base(d: &Design) -> Manifold {
    let mut plate = end_outline(d, d.end_inner_z(), d.end_t);
    let footprint = end_outline(d, d.lug_outer_z() - 1.0, d.boss_h + 2.0);
    let mut slots = Vec::new();
    for [px, py] in d.all_peg_points() {
        plate = plate.union(&cylinder(
            d.boss_d,
            d.boss_h + EPS,
            [px, py, d.lug_outer_z()],
            None,
        ));
        // A cone at the boss root spreads the snap load into the plate; trimmed to
        // the outline so it can't reach the panels.
        plate = plate.union(
            &root_chamfer(d.boss_d, d.root_chamfer, [px, py], d.end_inner_z(), -1.0)
                .intersection(&footprint),
        );
        // Peg points -Z: build it pointing up from z = 0, mirror, then move into place.
        let (body, slot) = peg_up(d, [0.0, 0.0, 0.0], d.lug_t);
        plate = plate.union(
            &body
                .mirror([0.0, 0.0, 1.0])
                .translate(px, py, d.lug_outer_z()),
        );
        slots.push(
            slot.mirror([0.0, 0.0, 1.0])
                .translate(px, py, d.lug_outer_z()),
        );
    }
    for slot in slots {
        plate = plate.difference(&slot);
    }
    plate
}

/// +Z end plate, with an opening over the payload.
pub fn end_plate_top(d: &Design) -> Manifold {
    end_plate_base(d).difference(&cylinder(
        d.top_window_d,
        d.end_t + 2.0,
        [0.0, 0.0, d.end_inner_z() - 1.0],
        None,
    ))
}

/// -Z end plate (inserted first), with four columns that carry the payload plate.
pub fn end_plate_bottom(d: &Design) -> Manifold {
    let mut plate = end_plate_base(d).mirror([0.0, 0.0, 1.0]);
    let face = -d.end_inner_z();
    let mut slots = Vec::new();
    for [x, y] in d.columns {
        plate = plate.union(&cylinder(
            d.column_d,
            d.column_length + 1.0,
            [x, y, face - 1.0],
            None,
        ));
        plate = plate.union(&root_chamfer(d.column_d, d.root_chamfer, [x, y], face, 1.0));
        let (body, slot) = peg_up(d, [x, y, d.plate_bottom_z()], d.plate_t);
        plate = plate.union(&body);
        slots.push(slot);
    }
    for slot in slots {
        plate = plate.difference(&slot);
    }
    plate
}

/// Payload plate: Uno R3 on four pins with two edge clips, Nano 33 BLE Sense on
/// two rails with an end stop and two strap posts. Board side faces +Z.
pub fn payload_plate(d: &Design) -> Manifold {
    let (px, py) = (d.plate[0] / 2.0, d.plate[1] / 2.0);
    let (bottom, top) = (d.plate_bottom_z(), d.plate_top_z());
    let mut parts = vec![cuboid([-px, -py, bottom], [px, py, top])];

    // Uno posts and locating pins.
    let pcb = d.uno_pcb_bottom_z();
    for i in 0..4 {
        let [x, y] = d.uno_hole(i);
        parts.push(cylinder(
            7.0,
            d.uno_post_h - 2.5 + EPS,
            [x, y, top - EPS],
            None,
        ));
        parts.push(cylinder(
            d.uno_post_top,
            2.5 + EPS,
            [x, y, pcb - 2.5 - EPS],
            None,
        ));
        parts.push(cylinder(
            d.uno_pin,
            d.pcb_uno + 0.9,
            [x, y, pcb - EPS],
            None,
        ));
        parts.push(cylinder(
            d.uno_pin,
            0.5 + EPS,
            [x, y, pcb + d.pcb_uno + 0.9 - EPS],
            Some(d.uno_pin - 0.8),
        ));
    }

    // Uno edge clips: left between the DC jack and USB, right at mid-height.
    let o = d.uno_origin();
    let catch = pcb + d.pcb_uno + 0.1;
    let left = o[0] - d.clip_gap;
    let right = o[0] + d.uno[0] + d.clip_gap;
    for (x0, x1, yc, inward) in [
        (left - d.clip_t, left, o[1] + 22.0, 1.0),
        (right, right + d.clip_t, o[1] + d.uno[1] / 2.0, -1.0),
    ] {
        let (y0, y1) = (yc - d.clip_w / 2.0, yc + d.clip_w / 2.0);
        parts.push(cuboid(
            [x0, y0, top - EPS],
            [x1, y1, catch + d.clip_lip + 1.5],
        ));
        let (lx0, lx1) = if inward > 0.0 {
            (x1 - EPS, x1 + d.clip_lip)
        } else {
            (x0 - d.clip_lip, x0 + EPS)
        };
        parts.push(cuboid(
            [lx0, y0, catch],
            [lx1, y1, catch + d.clip_lip + EPS],
        ));
        let ramp: [[f64; 2]; 3] = if inward > 0.0 {
            [[x1, 0.0], [x1 + d.clip_lip, 0.0], [x1, 1.5]]
        } else {
            [[x0 - d.clip_lip, 0.0], [x0, 0.0], [x0, 1.5]]
        };
        parts.push(
            Manifold::extrude(&CrossSection::from_simple_polygon(&ramp), d.clip_w)
                .rotate(90.0, 0.0, 0.0)
                .translate(0.0, y1, catch + d.clip_lip - EPS),
        );
    }

    // Nano rails between the pin rows, end stop, strap posts.
    let [nx, ny] = d.nano_center;
    let rail_top = d.nano_pcb_bottom_z();
    for (y0, y1) in [(2.0, 5.0), (-5.0, -2.0)] {
        parts.push(cuboid(
            [nx - 15.0, ny + y0, top - EPS],
            [nx + 15.0, ny + y1, rail_top],
        ));
    }
    let stop = nx + d.nano[0] / 2.0 + 0.3;
    parts.push(cuboid(
        [stop, ny - 3.0, top - EPS],
        [stop + 2.5, ny + 3.0, rail_top + 1.0],
    ));
    let strap_bottom = d.strap_bottom_z();
    let mut slots = Vec::new();
    for side in [-1.0, 1.0] {
        let (x, y) = (nx, ny + side * d.strap_post);
        parts.push(cylinder(8.0, 5.0 + EPS, [x, y, top - EPS], None));
        parts.push(cylinder(
            d.peg_shank,
            strap_bottom - top - 5.0 + EPS,
            [x, y, top + 5.0 - EPS],
            None,
        ));
        let (body, slot) = peg_up(d, [x, y, strap_bottom], d.strap_t);
        parts.push(body);
        slots.push(slot);
    }

    let mut plate = union_all(&parts);
    for [x, y] in d.columns {
        plate = plate.difference(&cylinder(
            d.snap_hole,
            d.plate_t + 2.0,
            [x, y, bottom - 1.0],
            None,
        ));
    }
    for slot in slots {
        plate = plate.difference(&slot);
    }
    plate
}

/// Nano strap in print orientation (feet up). Feet press on the header pin tips.
pub fn nano_strap(d: &Design) -> Manifold {
    let length = 2.0 * d.strap_post + 10.0;
    let mut strap = cuboid(
        [-d.strap_w / 2.0, -length / 2.0, 0.0],
        [d.strap_w / 2.0, length / 2.0, d.strap_t],
    );
    let feet_h = d.strap_arch - d.strap_foot_above_pcb;
    for side in [-1.0, 1.0] {
        strap = strap.difference(&cylinder(
            d.snap_hole,
            d.strap_t + 2.0,
            [0.0, side * d.strap_post, -1.0],
            None,
        ));
        let y0 = side * (d.nano_row / 2.0 - 1.0);
        let y1 = side * (d.nano[1] / 2.0 + 0.5);
        strap = strap.union(&cuboid(
            [-d.strap_w / 2.0, y0.min(y1), d.strap_t - EPS],
            [d.strap_w / 2.0, y0.max(y1), d.strap_t + feet_h],
        ));
    }
    strap
}

/// The strap where it sits in the assembly, feet down on the Nano.
pub fn nano_strap_seated(d: &Design) -> Manifold {
    let [nx, ny] = d.nano_center;
    nano_strap(d)
        .rotate(180.0, 0.0, 0.0)
        .translate(nx, ny, d.strap_bottom_z() + d.strap_t)
}

/// Snap test, both pieces in print orientation: a boss and peg like the end plates',
/// and a standing wall with a teardrop hole like the side panels' end walls.
pub fn snap_test(d: &Design) -> (Manifold, Manifold) {
    let base = cuboid([-8.0, -8.0, 0.0], [8.0, 8.0, 2.0]);
    let boss = cylinder(d.boss_d, d.boss_h + EPS, [0.0, 0.0, 2.0 - EPS], None);
    let (body, slot) = peg_up(d, [0.0, 0.0, 2.0 + d.boss_h], d.lug_t);
    let peg = base.union(&boss).union(&body).difference(&slot);

    // A wall the same thickness as the end walls, hole at the same height above the
    // wall's root, printed standing up with the teardrop point at the top.
    let hole_h = d.web_inner_y() - d.peg_y;
    let wall_h = d.web_inner_y() - d.lug_inner_y;
    let wall = cuboid([-10.0, 2.0 - EPS, 0.0], [10.0, 2.0 + wall_h, d.lug_t])
        .difference(&teardrop_hole_z(
            d.snap_hole,
            d.lug_t + 2.0,
            [0.0, 2.0 + hole_h, d.lug_t / 2.0],
            [0.0, 1.0],
        ))
        .rotate(90.0, 0.0, 0.0);
    // After the turn the wall stands on its edge; give it a foot.
    let (lo, _) = crate::geom::bounds(&wall);
    let wall = wall.translate(0.0, -lo[1], -lo[2] + 2.0 - EPS);
    let foot = cuboid([-10.0, 0.0, 0.0], [10.0, 12.0, 2.0]);
    (peg, foot.union(&wall))
}

/// Frame parts in the assembled position, for checks and the assembly preview.
pub struct Assembly {
    pub panel_pos_y: Manifold,
    pub panel_neg_y: Manifold,
    pub cover_pos_x: Manifold,
    pub cover_neg_x: Manifold,
    pub end_top: Manifold,
    pub end_bottom: Manifold,
    pub payload: Manifold,
    pub strap: Manifold,
}

impl Assembly {
    pub fn new(d: &Design) -> Self {
        let panel = side_panel(d);
        let cover = cover(d);
        Self {
            panel_neg_y: panel.rotate(0.0, 0.0, 180.0),
            panel_pos_y: panel,
            cover_neg_x: cover.rotate(0.0, 0.0, 180.0),
            cover_pos_x: cover,
            end_top: end_plate_top(d),
            end_bottom: end_plate_bottom(d),
            payload: payload_plate(d),
            strap: nano_strap_seated(d),
        }
    }

    /// The six frame parts: panels, covers and end plates.
    pub fn frame(&self) -> [&Manifold; 6] {
        [
            &self.panel_pos_y,
            &self.panel_neg_y,
            &self.cover_pos_x,
            &self.cover_neg_x,
            &self.end_top,
            &self.end_bottom,
        ]
    }

    /// Every part by name, frame first.
    pub fn named(&self) -> [(&'static str, &Manifold); 8] {
        [
            ("+Y panel", &self.panel_pos_y),
            ("-Y panel", &self.panel_neg_y),
            ("+X cover", &self.cover_pos_x),
            ("-X cover", &self.cover_neg_x),
            ("+Z end plate", &self.end_top),
            ("-Z end plate", &self.end_bottom),
            ("payload plate", &self.payload),
            ("Nano strap", &self.strap),
        ]
    }

    pub fn all(&self) -> Manifold {
        union_all(&self.named().map(|(_, m)| m.clone()))
    }
}
