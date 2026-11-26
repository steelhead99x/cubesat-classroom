//! Rev C parts, modeled where they sit in the assembled CubeSat (CDS coordinates).
//! `export` turns each one into its print orientation.

use manifold_csg::Manifold;

use crate::design::Design;
use crate::fasteners::{
    key_in_place, latch, latch_ear, latch_pocket, placed_stud, poly, qt_bar, qt_stud_length,
    qt_washer,
};
use crate::geom::{EPS, cuboid, cylinder, rounded_rect, teardrop_hole_z, union_all};

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
    let web = cuboid(
        [-(h - d.rail / 2.0), d.web_inner_y(), -web_z],
        [h - d.rail / 2.0, h, web_z],
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
                -1.0,
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

/// End plate at +Z with its four bosses and snap pegs pointing towards the centre.
fn end_plate_base(d: &Design) -> Manifold {
    let x = d.rail_inner() - d.end_gap;
    let y = d.web_inner_y() - d.end_gap;
    let mut plate = cuboid([-x, -y, d.end_inner_z()], [x, y, d.end_outer_z()]);
    let mut slots = Vec::new();
    for [px, py] in d.peg_points() {
        plate = plate.union(&cylinder(
            d.boss_d,
            d.boss_h + EPS,
            [px, py, d.lug_outer_z()],
            None,
        ));
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

/// -Z end plate (inserted first), with four hollow columns that carry the payload
/// plate. Each column has a cross-bar near its top for a quarter-turn stud that comes
/// down through the plate.
pub fn end_plate_bottom(d: &Design) -> Manifold {
    let mut plate = end_plate_base(d).mirror([0.0, 0.0, 1.0]);
    let face = -d.end_inner_z();
    let top = d.plate_bottom_z();
    for [x, y] in d.columns {
        plate = plate.union(&cylinder(
            d.column_od,
            top - face + 1.0,
            [x, y, face - 1.0],
            None,
        ));
    }
    let depth = column_bore_depth(d);
    for [x, y] in d.columns {
        plate = plate.difference(&cylinder(d.qt.bore, depth + 1.0, [x, y, top - depth], None));
    }
    for [x, y] in d.columns {
        plate = plate.union(&qt_bar(d, x, y, top, true));
    }
    plate
}

/// How deep each column's bore goes below its top: room for the stud tip at the detent crest.
pub fn column_bore_depth(d: &Design) -> f64 {
    qt_stud_length(d, d.plate_stud_a0()) - (d.washer_h() + d.plate_t) + 1.0
}

/// Extrude a (y, z) profile along +X from `x0` to `x1`, centred on the Nano's board.
fn groove(d: &Design, pts: &[[f64; 2]], x0: f64, x1: f64, mouth_z: f64) -> Manifold {
    poly(pts)
        .extrude(x1 - x0)
        .rotate(0.0, 90.0, 0.0)
        .rotate(90.0, 0.0, 0.0)
        .translate(x0, d.nano.y0 + d.nano.t / 2.0, mouth_z)
}

/// Payload plate, in plate coordinates (underside at z = 0, board side up):
/// - Uno R3 on four posts with short locating pins, held by four snap latches with lock keys;
/// - Nano 33 BLE Sense on its edge in a V-groove card cage, with an end stop at each end;
/// - two barred pillars for the Nano clamp's quarter-turn studs;
/// - a hole over each column for the plate's own quarter-turn studs.
pub fn payload_plate(d: &Design) -> Manifold {
    let (px, py) = (d.plate[0] / 2.0, d.plate[1] / 2.0);
    let top = d.plate_t;
    let mut base = vec![cuboid([-px, -py, 0.0], [px, py, top])];
    base.extend((0..4).map(|i| latch_ear(d, i)));
    let mut part = union_all(&base);

    // Uno R3: four posts with locating pins that stop below the board's top face.
    let pcb = d.uno_pcb_bottom();
    let pin_h = d.pcb_uno + d.uno_pin_above;
    let mut posts = Vec::new();
    for i in 0..4 {
        let [x, y] = d.uno_hole(i);
        posts.push(cylinder(
            7.0,
            d.uno_post_h - 2.5 + EPS,
            [x, y, top - EPS],
            None,
        ));
        posts.push(cylinder(
            d.uno_post_top,
            2.5 + EPS,
            [x, y, pcb - 2.5 - EPS],
            None,
        ));
        posts.push(cylinder(
            d.uno_pin,
            pin_h - 0.4 + 2.0 * EPS,
            [x, y, pcb - EPS],
            None,
        ));
        posts.push(cylinder(
            d.uno_pin,
            0.4,
            [x, y, pcb + pin_h - 0.4],
            Some(d.uno_pin - 0.6),
        ));
    }
    part = part.union(&union_all(&posts));

    // Uno snap latches, each rooted in a pocket so its arm is longer.
    for i in 0..4 {
        part = part
            .difference(&latch_pocket(d, i))
            .union(&latch(d, i, 0.0));
    }

    // Pillars for the Nano clamp's two quarter-turn studs, each with a cross-bar inside.
    let n = &d.nano;
    let z_pillar = d.clamp_bottom_z();
    let depth = qt_stud_length(d, d.clamp_stud_a0()) - (d.washer_h() + n.clamp_t) + 1.0;
    let mut bores = Vec::new();
    let mut bars = Vec::new();
    for [x, y] in d.pillars() {
        part = part.union(&cylinder(
            n.pillar_od,
            z_pillar - top + EPS,
            [x, y, top - EPS],
            None,
        ));
        bores.push(cylinder(
            d.qt.bore,
            depth + 1.0,
            [x, y, z_pillar - depth],
            None,
        ));
        bars.push(qt_bar(d, x, y, z_pillar, true));
    }

    // Nano card cage, lower half: V-groove rail, USB-end corner stop, far-end wall.
    let (y0, y1) = (n.y0 - n.v_wall, n.y0 + n.t + n.v_wall);
    let x_end = n.x0 + n.l + n.end_play;
    part = union_all(&[
        part,
        cuboid(
            [n.x0 - 2.0, y0, top - EPS],
            [x_end + 1.0, y1, d.base_mouth_z()],
        ),
        cuboid(
            [n.x0 - 2.0, n.y0 - 1.0, d.base_mouth_z() - EPS],
            [n.x0, n.y0 + n.t + 1.0, n.z0 + n.stop_reach],
        ),
        cuboid([x_end, y0, top - EPS], [x_end + 2.0, y1, n.z0 + n.w - 2.0]),
    ]);
    let (m, a) = (n.v_mouth / 2.0, n.v_apex / 2.0);
    part = part.difference(&groove(
        d,
        &[
            [-m, 0.0],
            [-a, -n.v_depth],
            [a, -n.v_depth],
            [m, 0.0],
            [m, 1.0],
            [-m, 1.0],
        ],
        n.x0 - EPS,
        x_end,
        d.base_mouth_z(),
    ));

    let holes: Vec<Manifold> = d
        .columns
        .iter()
        .map(|&[x, y]| cylinder(d.qt.bore, top + 2.0, [x, y, -1.0], None))
        .chain(bores)
        .collect();
    part = part.difference(&union_all(&holes));
    part.union(&union_all(&bars))
}

/// Nano clamp: the cap rail over the Nano's upper edge, held down at both ends by
/// quarter-turn studs into the pillars. One flat top face, so it prints upside down
/// without supports. It doesn't touch the Uno, so either board comes out on its own.
/// Plate coordinates, resting on its pillars; the Nano's edge lifts it `preload` in service.
pub fn nano_clamp(d: &Design) -> Manifold {
    let n = &d.nano;
    let zs = d.clamp_bottom_z();
    let zt = zs + n.clamp_t;
    let [hx, hy] = n.anchor;
    let [fx, fy] = n.far_pillar;
    let pad = d.washer.od + 0.5;
    let outline = cuboid(
        [-d.plate[0] / 2.0 + 0.5, -d.plate[1] / 2.0 + 0.5, zs - 1.0],
        [d.plate[0] / 2.0 - 0.5, d.plate[1] / 2.0 - 0.5, zt + 1.0],
    );
    let rail = union_all(&[
        cuboid(
            [n.x0 - 2.0, n.y0 - n.v_wall, zs],
            [fx, n.y0 + n.t + n.v_wall, zt],
        ),
        cylinder(pad, zt - zs, [fx, fy, zs], None),
        cuboid([fx - 4.0, n.y0, zs], [fx + 4.0, fy, zt]),
    ])
    .intersection(&outline);
    let arm = cuboid([hx - 4.0, n.y0, zs], [hx + 4.0, hy, zt]).union(&cylinder(
        pad,
        zt - zs,
        [hx, hy, zs],
        None,
    ));
    let stop = cuboid(
        [n.x0 - 2.0, n.y0 - 1.0, n.z0 + n.w - n.stop_reach],
        [n.x0, n.y0 + n.t + 1.0, zs + EPS],
    );
    let (m, a) = (n.v_mouth / 2.0, n.v_apex / 2.0);
    let mut part = union_all(&[rail, arm, stop]).difference(&groove(
        d,
        &[
            [-m, -1.0],
            [m, -1.0],
            [m, 0.0],
            [a, n.v_depth],
            [-a, n.v_depth],
            [-m, 0.0],
        ],
        n.x0 - EPS,
        n.x0 + n.l + n.end_play + EPS,
        zs,
    ));
    for [x, y] in d.pillars() {
        part = part.difference(&cylinder(d.qt.bore, zt - zs + 2.0, [x, y, zs - 1.0], None));
    }
    part
}

/// A spring washer seated feet down on a surface at `z`, centred on (x, y).
pub fn washer_seated(d: &Design, x: f64, y: f64, z: f64) -> Manifold {
    qt_washer(d)
        .mirror([0.0, 0.0, 1.0])
        .translate(x, y, z + d.washer_h())
}

/// Plate studs and clamp studs, locked, with their washers, in plate coordinates.
pub fn fasteners_in_place(d: &Design) -> Manifold {
    let mut all = Vec::new();
    for [x, y] in d.columns {
        all.push(washer_seated(d, x, y, d.plate_t));
        all.push(placed_stud(
            d,
            d.plate_stud_a0(),
            x,
            y,
            d.plate_t + d.washer_h(),
            90.0,
        ));
    }
    let clamp_top = d.clamp_bottom_z() + d.nano.preload + d.nano.clamp_t;
    for [x, y] in d.pillars() {
        all.push(washer_seated(d, x, y, clamp_top));
        all.push(placed_stud(
            d,
            d.clamp_stud_a0(),
            x,
            y,
            clamp_top + d.washer_h(),
            90.0,
        ));
    }
    union_all(&all)
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
            1.0,
        ))
        .rotate(90.0, 0.0, 0.0);
    // After the turn the wall stands on its edge; give it a foot.
    let (lo, _) = crate::geom::bounds(&wall);
    let wall = wall.translate(0.0, -lo[1], -lo[2] + 2.0 - EPS);
    let foot = cuboid([-10.0, 0.0, 0.0], [10.0, 12.0, 2.0]);
    (peg, foot.union(&wall))
}

/// Move a solid from payload-plate coordinates into the CubeSat.
pub fn on_columns(d: &Design, m: &Manifold) -> Manifold {
    m.translate(0.0, 0.0, d.plate_bottom_z())
}

/// Frame parts in the assembled position, for checks and the assembly preview.
pub struct Assembly {
    pub panel_pos_y: Manifold,
    pub panel_neg_y: Manifold,
    pub end_top: Manifold,
    pub end_bottom: Manifold,
    pub payload: Manifold,
    /// Lifted by the Nano's edge, as in service.
    pub clamp: Manifold,
    pub keys: Vec<Manifold>,
}

impl Assembly {
    pub fn new(d: &Design) -> Self {
        let panel = side_panel(d);
        Self {
            panel_neg_y: panel.rotate(0.0, 0.0, 180.0),
            panel_pos_y: panel,
            end_top: end_plate_top(d),
            end_bottom: end_plate_bottom(d),
            payload: on_columns(d, &payload_plate(d)),
            clamp: on_columns(d, &nano_clamp(d).translate(0.0, 0.0, d.nano.preload)),
            keys: (0..4).map(|i| on_columns(d, &key_in_place(d, i))).collect(),
        }
    }

    /// The four frame parts (panels and end plates).
    pub fn frame(&self) -> [&Manifold; 4] {
        [
            &self.panel_pos_y,
            &self.panel_neg_y,
            &self.end_top,
            &self.end_bottom,
        ]
    }

    /// Every printed part except the studs and washers.
    pub fn parts(&self) -> Vec<(String, &Manifold)> {
        let mut v = vec![
            ("+Y panel".to_string(), &self.panel_pos_y),
            ("-Y panel".to_string(), &self.panel_neg_y),
            ("+Z end plate".to_string(), &self.end_top),
            ("-Z end plate".to_string(), &self.end_bottom),
            ("payload plate".to_string(), &self.payload),
            ("Nano clamp".to_string(), &self.clamp),
        ];
        for (i, k) in self.keys.iter().enumerate() {
            v.push((format!("lock key {}", i + 1), k));
        }
        v
    }

    pub fn all(&self, d: &Design) -> Manifold {
        let mut v: Vec<Manifold> = self.parts().into_iter().map(|(_, m)| m.clone()).collect();
        v.push(on_columns(d, &fasteners_in_place(d)));
        union_all(&v)
    }
}
