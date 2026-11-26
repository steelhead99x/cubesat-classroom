//! Checks against the CubeSat Design Specification (CDS) Rev 14.1, fit checks
//! between the parts and the boards, and printability checks for a Bambu P1S.

use manifold_csg::Manifold;

use crate::boards::{
    nano_card, nano_pcb, nano_plug, nano_top_parts, uno_parts, uno_pcb, uno_underside,
};
use crate::design::Design;
use crate::fasteners::{
    bar_stress, key_flat, key_in_place, key_strain, latch, latch_force, latch_strain, placed_stud,
    plate_hold_down, qt_stud, qt_washer, washer_crest, washer_strain,
};
use crate::geom::{bounds, cuboid, cylinder, overlap, union_all};
use crate::mesh::Mesh;
use crate::parts::{
    Assembly, fasteners_in_place, lug_z, nano_clamp, on_columns, payload_plate, peg_height,
};

/// PLA density for the mass estimate, g/mm3. Solid parts give an upper bound;
/// the slicer reports the real mass at its infill.
pub const PLA_G_PER_MM3: f64 = 1.24e-3;
/// Board masses for the centre-of-mass estimate, grams.
pub const UNO_G: f64 = 25.0;
pub const NANO_G: f64 = 5.0;
/// Bambu P1S build plate and height, mm.
pub const BED: f64 = 256.0;
pub const BED_HEIGHT: f64 = 256.0;
/// Overhang allowed per part without supports: 45 degrees, ignoring the first mm.
pub const MAX_OVERHANG_MM2: f64 = 200.0;

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Pass,
    Fail,
    /// Not met by a printed classroom frame; a flight version needs a provider waiver
    /// or a change of material or hardware.
    Deviation,
    NotApplicable,
}

#[derive(Clone, Debug)]
pub struct Check {
    pub id: &'static str,
    pub requirement: &'static str,
    pub source: &'static str,
    pub outcome: Outcome,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct Report {
    pub checks: Vec<Check>,
}

impl Report {
    fn add(
        &mut self,
        id: &'static str,
        requirement: &'static str,
        source: &'static str,
        ok: bool,
        detail: String,
    ) {
        let outcome = if ok { Outcome::Pass } else { Outcome::Fail };
        self.checks.push(Check {
            id,
            requirement,
            source,
            outcome,
            detail,
        });
    }

    fn note(
        &mut self,
        id: &'static str,
        requirement: &'static str,
        source: &'static str,
        outcome: Outcome,
        detail: &str,
    ) {
        self.checks.push(Check {
            id,
            requirement,
            source,
            outcome,
            detail: detail.to_string(),
        });
    }

    pub fn failures(&self) -> Vec<&Check> {
        self.checks
            .iter()
            .filter(|c| c.outcome == Outcome::Fail)
            .collect()
    }

    pub fn passed(&self) -> bool {
        self.failures().is_empty()
    }

    /// Markdown compliance table.
    pub fn markdown(&self) -> String {
        use std::fmt::Write as _;
        let mut s = String::from(
            "| ID | Requirement | Source | Result | Detail |\n| --- | --- | --- | --- | --- |\n",
        );
        for c in &self.checks {
            let result = match c.outcome {
                Outcome::Pass => "pass",
                Outcome::Fail => "**FAIL**",
                Outcome::Deviation => "deviation",
                Outcome::NotApplicable => "n/a",
            };
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | {} |",
                c.id, c.requirement, c.source, result, c.detail
            );
        }
        s
    }
}

/// Bending strain of one peg half as the barb squeezes through the hole.
pub fn peg_strain(d: &Design) -> f64 {
    let half = (d.peg_shank - d.peg_slot) / 2.0;
    let arm = peg_height(d, d.lug_t) + d.slot_into_body;
    1.5 * half * (d.peg_barb - d.snap_hole) / 2.0 / (arm * arm)
}

/// `value >= minimum`, ignoring floating-point noise far below print resolution.
fn at_least(value: f64, minimum: f64) -> bool {
    value >= minimum - 1e-6
}

/// Parts in print orientation, by name.
pub fn print_parts(d: &Design) -> Vec<(&'static str, Manifold)> {
    let (peg, wall) = crate::parts::snap_test(d);
    vec![
        (
            "side-panel",
            crate::parts::side_panel(d).rotate(-90.0, 0.0, 0.0),
        ),
        (
            "end-plate-top",
            crate::parts::end_plate_top(d).rotate(180.0, 0.0, 0.0),
        ),
        ("end-plate-bottom", crate::parts::end_plate_bottom(d)),
        ("payload-plate", payload_plate(d)),
        // Flat top on the bed.
        ("nano-clamp", nano_clamp(d).rotate(180.0, 0.0, 0.0)),
        ("lock-key", key_flat(d)),
        ("qt-stud-plate", qt_stud(d, d.plate_stud_a0())),
        ("qt-stud-clamp", qt_stud(d, d.clamp_stud_a0())),
        ("qt-washer", qt_washer(d)),
        ("snap-test-peg", peg),
        ("snap-test-wall", wall),
    ]
}

/// Uno latches and keys, in plate coordinates. Returns the problems found.
fn latch_issues(
    d: &Design,
    features: &Manifold,
    others: &Manifold,
    keys: &[Manifold],
) -> Vec<String> {
    let mut issues = Vec::new();
    let l = &d.latch;
    let [ux, uy] = d.uno_origin();
    let top = d.uno_top();
    let mut fixed = features.clone();
    let mut seated = Vec::new();
    for i in 0..4 {
        fixed = fixed.difference(&latch(d, i, 0.0));
        seated.push(latch(d, i, l.preload + 0.01));
    }
    let mut in_service = vec![fixed.clone(), others.clone()];
    in_service.extend(seated.iter().cloned());
    in_service.extend(keys.iter().cloned());
    let in_service = union_all(&in_service);
    if overlap(&in_service, &uno_pcb(d)) > 1e-3 {
        issues.push("a latch or part passes through the Uno board".into());
    }
    if overlap(&in_service, &uno_underside(d)) > 1e-3 {
        issues.push("something under the Uno would hit its solder joints".into());
    }
    for (name, body) in uno_parts(d) {
        if overlap(&in_service, &body) > 1e-3 {
            issues.push(format!(
                "a latch, key, stud or pillar hits the Uno's {name}"
            ));
        }
    }
    let above = cuboid([ux, uy, top], [ux + d.uno[0], uy + d.uno[1], top + 3.0]);
    for (i, s) in seated.iter().enumerate() {
        let edge = format!("{:?}", l.places[i].0).to_lowercase();
        if overlap(s, &above) / l.w <= 0.3 {
            issues.push(format!(
                "the {edge} latch's hook doesn't reach over the Uno"
            ));
        }
        let relaxed = latch(d, i, 0.0);
        if keys.iter().any(|k| overlap(k, &relaxed) > 1e-3) {
            issues.push(format!("a lock key doesn't fit the {edge} latch"));
        }
        let opened = latch(d, i, l.preload + 2.0 * d.key.clear + 0.05);
        if keys.iter().map(|k| overlap(&opened, k)).sum::<f64>() <= 1e-3 {
            issues.push(format!(
                "the {edge} latch can open past its key: the key doesn't lock it"
            ));
        }
    }
    let mut snapping = vec![fixed];
    snapping.extend((0..4).map(|i| latch(d, i, l.reach + 0.05)));
    // The board's outline swept down from above, with its USB-B socket.
    let path = cuboid([ux, uy, top + 0.01], [ux + d.uno[0], uy + d.uno[1], 80.0]).union(&cuboid(
        [ux - 7.0, uy + 30.0, top + 0.01],
        [ux, uy + 46.0, 80.0],
    ));
    if overlap(&union_all(&snapping), &path) > 1e-3 {
        issues.push("the Uno can't be pushed down past its latches".into());
    }
    issues
}

/// Nano card cage and clamp, in plate coordinates. Returns the problems found.
fn nano_issues(
    d: &Design,
    features: &Manifold,
    clamp: &Manifold,
    everything: &Manifold,
) -> Vec<String> {
    let mut issues = Vec::new();
    let n = &d.nano;
    let pcb = nano_pcb(d);
    if overlap(everything, &nano_top_parts(d)) > 1e-3 {
        issues.push("a groove wall, stop or clamp hits a part on the Nano".into());
    }
    if overlap(everything, &pcb) > 1e-3 {
        issues.push("the Nano's board does not fit between its V-grooves and stops".into());
    }
    let moves = [
        ("up", [0.0, 0.0, 0.3]),
        ("down", [0.0, 0.0, -0.3]),
        ("towards the Uno", [0.0, 0.3, 0.0]),
        ("towards the plate edge", [0.0, -0.3, 0.0]),
        ("towards the USB end", [-n.end_play - 0.2, 0.0, 0.0]),
        ("towards the far end", [n.end_play + 0.2, 0.0, 0.0]),
    ];
    for (label, [dx, dy, dz]) in moves {
        let moved = pcb.translate(dx, dy, dz);
        if overlap(everything, &moved) <= 1e-3 {
            issues.push(format!("the Nano could move {label}: nothing holds it"));
        }
        if dx == 0.0 && dz == 0.0 {
            for (half, solid) in [("base", features), ("cap", clamp)] {
                if overlap(solid, &moved) <= 1e-3 {
                    issues.push(format!("the {half} groove does not hold the Nano sideways"));
                }
            }
        }
    }
    // With the clamp off, the Nano goes straight down into its groove, USB end first
    // past the stop: its outline and parts swept up from the groove must be clear.
    let (h, x1) = (d.plate[1], n.x0 + n.l + 0.1);
    let lift = union_all(&[
        cuboid(
            [n.x0 + 0.1, n.y0 + 0.02, n.z0 + 0.01],
            [x1, n.y0 + n.t - 0.02, 90.0],
        ),
        nano_top_parts(d).translate(0.1, 0.0, 0.0),
        cuboid(
            [n.x0 + 0.1, n.y0, n.z0 + n.edge_clear],
            [n.x0 + n.l, n.y0 + n.t + 2.8, 90.0],
        ),
    ])
    .difference(&cuboid([-h, -h, -1.0], [h, h, n.z0 + 0.01]))
    .union(&cuboid(
        [n.x0 + 0.1, n.y0 + 0.02, n.z0 + n.w],
        [x1, n.y0 + n.t + 2.8, 90.0],
    ));
    if overlap(features, &lift) > 1e-3 {
        issues.push("with the clamp off, the Nano can't be lowered into its groove".into());
    }
    let room = n.x0 + crate::boards::NANO_PARTS[0].1[0] + d.plate[0] / 2.0;
    if !at_least(room, n.min_plug_room) {
        issues.push(format!(
            "only {room:.1} mm from the Nano's USB to the plate edge, less than {:.0} mm for the plug",
            n.min_plug_room
        ));
    }
    issues
}

/// Quarter-turn studs, in plate coordinates: each passes its bar going in, clears
/// everything when locked, and is held by the bar if pulled.
fn stud_issues(
    d: &Design,
    column_tops: &Manifold,
    plate: &Manifold,
    clamp: &Manifold,
) -> Vec<String> {
    let mut issues = Vec::new();
    let mut check = |what: &str, a0: f64, xy: &[[f64; 2]], z_head: f64, stack: &Manifold| {
        for &[x, y] in xy {
            if overlap(&placed_stud(d, a0, x, y, z_head, 0.0), stack) > 1e-3 {
                issues.push(format!("a {what} stud can't be pushed in past its bar"));
            }
            if overlap(&placed_stud(d, a0, x, y, z_head - 0.02, 90.0), stack) > 1e-3 {
                issues.push(format!("a locked {what} stud collides with its receptacle"));
            }
            if overlap(&placed_stud(d, a0, x, y, z_head + 0.3, 90.0), stack) <= 1e-3 {
                issues.push(format!("a locked {what} stud isn't held by its bar"));
            }
        }
    };
    let plate_stack = plate.union(column_tops);
    check(
        "plate",
        d.plate_stud_a0(),
        &d.columns,
        d.plate_t + d.washer_h(),
        &plate_stack,
    );
    let clamp_stack = plate.union(clamp);
    check(
        "Nano clamp",
        d.clamp_stud_a0(),
        &d.pillars(),
        d.clamp_bottom_z() + d.nano.clamp_t + d.washer_h(),
        &clamp_stack,
    );
    issues
}

/// Run every check. A design is ready to print when `Report::passed` is true.
#[allow(clippy::too_many_lines)]
pub fn verify(d: &Design) -> Report {
    let mut r = Report::default();
    let a = Assembly::new(d);
    let (h, hz) = (d.half(), d.half_z());
    // Both boards with their parts, and the Uno's clearance margin, in the CubeSat.
    let m = d.min_board_clearance;
    let [ux, uy] = d.uno_origin();
    let mut boards = vec![
        cuboid(
            [ux - m, uy - m, d.uno_pcb_bottom()],
            [ux + d.uno[0] + m, uy + d.uno[1] + m, d.uno_top()],
        ),
        nano_card(d),
    ];
    boards.extend(uno_parts(d).into_iter().map(|(_, b)| b));
    let board_set = on_columns(d, &union_all(&boards));
    let frame = union_all(&a.frame().map(Clone::clone));

    // ---- CDS mechanical (Rev 14.1 §2.2) ----
    let (lo, hi) = bounds(&frame);
    let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let centred = lo.iter().zip(hi.iter()).all(|(l, u)| (l + u).abs() < 1e-6);
    r.add(
        "CDS-2.2.1",
        "Outer size 100.0 x 100.0 x 113.5 mm (±0.1), origin at the geometric centre",
        "CDS Rev 14.1 §2.2.1, Appendix B",
        (size[0] - 100.0).abs() <= 0.1
            && (size[1] - 100.0).abs() <= 0.1
            && (size[2] - 113.5).abs() <= 0.1
            && centred,
        format!(
            "CAD {:.2} x {:.2} x {:.2} mm; measure the print",
            size[0], size[1], size[2]
        ),
    );
    let (alo, ahi) = bounds(&union_all(&[a.all(d), board_set.clone()]));
    let inside = alo[0] >= -h - 1e-6
        && alo[1] >= -h - 1e-6
        && ahi[0] <= h + 1e-6
        && ahi[1] <= h + 1e-6
        && alo[2] >= -hz - 1e-6
        && ahi[2] <= hz + 1e-6;
    r.add(
        "CDS-2.2.3",
        "Nothing protrudes more than 6.5 mm from the rail planes",
        "CDS Rev 14.1 §2.2.3",
        inside,
        "every part and both boards stay inside the rail envelope (0 mm)".into(),
    );
    r.add(
        "CDS-2.2.5",
        "Rails at least 8.5 mm wide",
        "CDS Rev 14.1 §2.2.5",
        d.rail >= 8.5,
        format!("{:.1} mm square rails, no protrusions on any face", d.rail),
    );
    r.add(
        "CDS-2.2.7",
        "Rail edges rounded to at least 1 mm (should)",
        "CDS Rev 14.1 §2.2.7",
        d.rail_radius >= 1.0,
        format!(
            "{:.1} mm radius on all four long edges of each rail",
            d.rail_radius
        ),
    );
    let rail_end_area =
        d.rail * d.rail - (4.0 - std::f64::consts::PI) * d.rail_radius * d.rail_radius;
    let flat = d.rail - 2.0 * d.rail_radius;
    // Both ends: the -Z end plate carries the payload columns, so it is a different part.
    let rails_only = [1.0, -1.0].iter().all(|&s: &f64| {
        let (z0, z1) = if s > 0.0 {
            (hz - 0.5, hz + 1.0)
        } else {
            (-hz - 1.0, -hz + 0.5)
        };
        let end_slab = cuboid([-h - 1.0, -h - 1.0, z0], [h + 1.0, h + 1.0, z1]);
        let end_volume = overlap(&frame, &end_slab);
        let rails = 4.0 * rail_end_area * 0.5;
        (end_volume - rails).abs() < 0.02 * rails
    });
    r.add(
        "CDS-2.2.8",
        "Rail ends give at least 6.5 x 6.5 mm contact on the ±Z faces; nothing else reaches the ends",
        "CDS Rev 14.1 §2.2.8",
        flat >= 6.5 && rails_only && d.end_recess > 0.0,
        format!("{flat:.1} x {flat:.1} mm flat on each rail end; end plates {:.1} mm below the rail ends", d.end_recess),
    );
    // Contact along each rail's two outer faces: a 0.2 mm skin just inside each face,
    // between the rounded edges. The worst face must be at least 75% solid.
    let skin = 0.2;
    let (ri, rr) = (d.rail_inner(), d.rail_radius);
    let mut worst_face: f64 = 1.0;
    for sx in [-1.0, 1.0] {
        for sy in [-1.0, 1.0] {
            let span = |s: f64| {
                if s > 0.0 {
                    (ri + rr, h - rr)
                } else {
                    (-h + rr, -ri - rr)
                }
            };
            let face = |s: f64| {
                if s > 0.0 {
                    (h - skin, h)
                } else {
                    (-h, -h + skin)
                }
            };
            let ((fx0, fx1), (ay0, ay1)) = (face(sx), span(sy));
            let ((bx0, bx1), (fy0, fy1)) = (span(sx), face(sy));
            for strip in [
                cuboid([fx0, ay0, -hz], [fx1, ay1, hz]),
                cuboid([bx0, fy0, -hz], [bx1, fy1, hz]),
            ] {
                worst_face = worst_face.min(overlap(&frame, &strip) / strip.volume());
            }
        }
    }
    r.add(
        "CDS-2.2.9",
        "At least 75% of each rail contacts the deployer rails (should)",
        "CDS Rev 14.1 §2.2.9",
        worst_face >= 0.75,
        format!(
            "worst rail face is {:.0}% solid along its length",
            worst_face * 100.0
        ),
    );

    // Mass and centre of mass, solid PLA (upper bound) plus the boards.
    let mut mass = 0.0;
    let mut moment = [0.0; 3];
    let fasteners = on_columns(d, &fasteners_in_place(d));
    let mut massed: Vec<&Manifold> = a.parts().into_iter().map(|(_, m)| m).collect();
    massed.push(&fasteners);
    for part in massed {
        let (v, c) =
            Mesh::from_solid("mass", part).map_or((0.0, [0.0; 3]), |m| m.volume_and_centroid());
        let g = v * PLA_G_PER_MM3;
        mass += g;
        for i in 0..3 {
            moment[i] += g * c[i];
        }
    }
    let o = d.uno_origin();
    let uno_c = [
        o[0] + d.uno[0] / 2.0,
        o[1] + d.uno[1] / 2.0,
        d.plate_bottom_z() + d.uno_pcb_bottom() + 5.0,
    ];
    let n = &d.nano;
    let nano_c = [
        n.x0 + n.l / 2.0,
        n.y0 + n.t,
        d.plate_bottom_z() + n.z0 + n.w / 2.0,
    ];
    for (g, c) in [(UNO_G, uno_c), (NANO_G, nano_c)] {
        mass += g;
        for i in 0..3 {
            moment[i] += g * c[i];
        }
    }
    let cg = moment.map(|m| m / mass);
    r.add(
        "CDS-2.2.10",
        "Mass within the 1U limit of 2.00 kg",
        "CDS Rev 14.1 §2.2.10, Table 1",
        mass <= 2000.0,
        format!("about {mass:.0} g with solid PLA, fasteners and both boards (the slicer's figure at 20% infill is lower)"),
    );
    r.add(
        "CDS-2.2.11",
        "Centre of mass within ±2 cm of the geometric centre in X, Y and Z",
        "CDS Rev 14.1 §2.2.11",
        cg.iter().all(|c| c.abs() <= 20.0),
        format!(
            "X {:+.1}, Y {:+.1}, Z {:+.1} mm (estimate; weigh and balance the build)",
            cg[0], cg[1], cg[2]
        ),
    );
    r.note(
        "CDS-2.2.6",
        "Rail surface roughness below 1.6 µm Ra (should)",
        "CDS Rev 14.1 §2.2.6",
        Outcome::Deviation,
        "printed layers are much rougher; a flight frame needs machined or finished rails",
    );
    r.note(
        "CDS-2.2.12/13",
        "Aluminium structure; hard-anodised rails and standoffs",
        "CDS Rev 14.1 §2.2.12, §2.2.13",
        Outcome::Deviation,
        "PLA classroom frame; any printed flight structure needs the launch provider's approval",
    );
    r.note(
        "CDS-2.1.7",
        "Low-outgassing materials (TML ≤ 1.0%, CVCM ≤ 0.1%)",
        "CDS Rev 14.1 §2.1.7",
        Outcome::Deviation,
        "PLA is not outgassing-qualified; see quest m2/material-outgassing",
    );
    r.note(
        "CDS-2.2.14",
        "Separation spring when sharing a deployer",
        "CDS Rev 14.1 §2.2.14",
        Outcome::NotApplicable,
        "bench frame, not in a shared deployer",
    );

    // ---- CDS electrical and operational (§2.3, §2.4), handled outside the frame ----
    r.note("CDS-2.3.2", "At least one deployment switch that cuts power while in the deployer", "CDS Rev 14.1 §2.3.2", Outcome::Deviation,
        "bench kit: the D2 jumper stands in for it and the firmware resets to pre-launch when it is refitted (§2.3.2.4); there is no rail-end switch");
    r.note("CDS-2.3.5", "Remove-before-flight pin cuts all power and protrudes at most 6.5 mm", "CDS Rev 14.1 §2.3.5", Outcome::Deviation,
        "the D2 jumper only inhibits the firmware; it reaches through the open -X face without protruding");
    r.note(
        "CDS-2.3.7/8",
        "Three independent RF inhibits; three deployable inhibits",
        "CDS Rev 14.1 §2.3.7, §2.3.8",
        Outcome::NotApplicable,
        "no transmitter and no deployables: the radio stays off and USB stands in for the link",
    );
    r.note("CDS-2.4.4/5", "Deployables wait 30 min; no transmission for 45 min after deployment", "CDS Rev 14.1 §2.4.4, §2.4.5", Outcome::NotApplicable,
        "firmware: the `cds` timing profile in firmware/nano33-cubesat uses these values; the bench default is 30 s");

    // ---- End-plate snaps ----
    let overlap_mm = d.peg_barb - d.snap_hole;
    r.add(
        "FIT-snap",
        "End-plate snap barbs overlap their holes enough to latch",
        "first P1S snap test (0.6 mm nozzle)",
        overlap_mm >= d.min_snap_overlap && d.snap_hole > d.peg_shank && d.peg_slot > overlap_mm,
        format!(
            "barb {:.1} mm into a {:.1} mm hole: {overlap_mm:.2} mm overlap, slot {:.1} mm",
            d.peg_barb, d.snap_hole, d.peg_slot
        ),
    );

    // ---- Uno latch and lock-key rules ----
    let l = &d.latch;
    let mut rules = Vec::new();
    if latch_strain(d) >= l.strain_limit {
        rules.push(format!(
            "latch arm bends {:.1}% snapping in, over the {:.1}% limit for an arm printed upright",
            latch_strain(d) * 100.0,
            l.strain_limit * 100.0
        ));
    }
    if d.plate_t - l.root_depth < 1.2 {
        rules.push("too little plate left under the latch pockets".to_string());
    }
    if !at_least(l.reach, l.min_reach) {
        rules.push(format!(
            "hooks reach {:.2} mm over the board, less than {:.1} mm",
            l.reach, l.min_reach
        ));
    }
    let free = 2.0 * d.key.clear;
    if free >= (l.reach - l.preload) / 2.0 {
        rules.push(format!(
            "with its key in, a latch can still open {free:.2} mm, more than half its {:.2} mm grip",
            l.reach - l.preload
        ));
    }
    if !(l.preload > 0.0 && l.preload < l.reach / 2.0) {
        rules.push("latch preload must be positive and well inside the hook's reach".to_string());
    }
    if key_strain(d) >= d.pla_strain_limit {
        rules.push(format!("key prongs bend {:.1}%", key_strain(d) * 100.0));
    }
    if d.key.slot < 2.0 * d.key.bump + 0.2 {
        rules.push(
            "the key's slot is too narrow for its prongs to flex past the notches".to_string(),
        );
    }
    r.add(
        "FIT-latch",
        "Uno latches snap in without cracking, hold the board down, and lock with their keys",
        "AMSAT kit latch design (generate.py)",
        rules.is_empty(),
        if rules.is_empty() {
            format!(
                "arm strain {:.1}% (limit {:.1}%), hook {:.1} mm over the edge, about {:.0} N down each, \
                 {free:.2} mm free with the key in, key prongs {:.1}%",
                latch_strain(d) * 100.0,
                l.strain_limit * 100.0,
                l.reach,
                latch_force(d),
                key_strain(d) * 100.0
            )
        } else {
            rules.join("; ")
        },
    );

    // ---- Quarter-turn fastener rules ----
    let q = &d.qt;
    let crest = washer_crest(d);
    let (have, need) = plate_hold_down(d);
    let mut rules = Vec::new();
    if !(q.det > 0.0 && q.det < q.lift) {
        rules.push("each stud needs a detent smaller than its cam draw-in".to_string());
    }
    if !at_least(d.pla_strain_limit, washer_strain(d, crest)) {
        rules.push(format!(
            "washer arms bend {:.2}% at the detent crest, over {:.0}%",
            washer_strain(d, crest) * 100.0,
            d.pla_strain_limit * 100.0
        ));
    }
    if d.washer.foot < q.lift + q.det + d.nano.preload + 0.2 - 1e-6 {
        rules.push("the washer's feet are too short for its hub to travel".to_string());
    }
    if have < need {
        rules.push(format!(
            "the four plate studs hold {have:.1} N down, less than the {need:.1} N the payload weighs at {:.0} g",
            d.design_g
        ));
    }
    if q.tip < 1.2 {
        rules.push("too little stud left behind the bar to carry the load".to_string());
    }
    if bar_stress(d) >= 20.0 {
        rules.push(format!(
            "bar bending stress {:.0} MPa is too high for PLA",
            bar_stress(d)
        ));
    }
    r.add(
        "FIT-qt",
        "Quarter-turn studs lock with a detent and hold the payload down at the design load",
        "AMSAT kit fastener design (generate.py)",
        rules.is_empty(),
        if rules.is_empty() {
            format!(
                "washer strain {:.2}% at the detent crest, plate held with {have:.0} N ({need:.0} N at {:.0} g), \
                 bar stress {:.0} MPa",
                washer_strain(d, crest) * 100.0,
                d.design_g,
                bar_stress(d)
            )
        } else {
            rules.join("; ")
        },
    );
    r.add(
        "FIT-strain",
        "End-plate snap pegs bend within the PLA limit",
        "PLA yield about 3%",
        peg_strain(d) < d.pla_strain_limit,
        format!(
            "peg {:.1}% (limit {:.0}%)",
            peg_strain(d) * 100.0,
            d.pla_strain_limit * 100.0
        ),
    );

    // ---- Parts fit together ----
    let parts = a.parts();
    let mut clashes = Vec::new();
    let mut pairs = 0;
    for i in 0..parts.len() {
        for j in i + 1..parts.len() {
            pairs += 1;
            if overlap(parts[i].1, parts[j].1) > 1e-3 {
                clashes.push(format!("{} / {}", parts[i].0, parts[j].0));
            }
        }
    }
    r.add(
        "FIT-parts",
        "Assembled parts do not overlap",
        "CAD assembly",
        clashes.is_empty(),
        if clashes.is_empty() {
            format!("{} parts, {pairs} pairs checked", parts.len())
        } else {
            clashes.join(", ")
        },
    );

    let mut snaps_ok = true;
    for sign in [1.0, -1.0] {
        let (z0, z1) = lug_z(d, sign);
        for [x, y] in d.peg_points() {
            let probe = cylinder(d.peg_shank - 0.2, z1 - z0 - 0.2, [x, y, z0 + 0.1], None);
            let panel = if y > 0.0 {
                &a.panel_pos_y
            } else {
                &a.panel_neg_y
            };
            let end = if sign > 0.0 {
                &a.end_top
            } else {
                &a.end_bottom
            };
            snaps_ok &=
                overlap(panel, &probe) < 1e-3 && overlap(end, &probe) > 0.5 * probe.volume();
        }
    }
    // Each plate hole lines up with its column's bore: a stud-sized probe through both is clear.
    for [x, y] in d.columns {
        let probe = cylinder(
            d.qt.d,
            d.plate_t + d.qt.bar_z - 0.1,
            [x, y, d.plate_bottom_z() - d.qt.bar_z + 0.05],
            None,
        );
        snaps_ok &= overlap(&a.payload, &probe) < 1e-3 && overlap(&a.end_bottom, &probe) < 1e-3;
    }
    r.add(
        "FIT-align",
        "Every snap peg passes through its hole, and every plate hole lines up with its column",
        "CAD assembly",
        snaps_ok,
        "8 end-plate snaps and 4 plate studs".into(),
    );

    // ---- Boards and fasteners, in plate coordinates ----
    let plate = payload_plate(d);
    let clamp = nano_clamp(d).translate(0.0, 0.0, d.nano.preload);
    let keys: Vec<Manifold> = (0..4).map(|i| key_in_place(d, i)).collect();
    let fasteners = fasteners_in_place(d);
    let features = plate.difference(&cuboid([-h, -h, -1.0], [h, h, d.plate_t]));
    let others = clamp.union(&fasteners);
    let mut everything = vec![features.clone(), others.clone()];
    everything.extend(keys.iter().cloned());
    let everything = union_all(&everything);

    let issues = latch_issues(d, &features, &others, &keys);
    r.add(
        "FIT-uno",
        "Uno R3 pushes down past its latches, and clears every latch, key, stud and pillar",
        "Arduino UNO-TH_Rev3e.brd",
        issues.is_empty(),
        if issues.is_empty() {
            format!(
                "board, underside, {} parts, hooks, keys and the push-down path",
                crate::boards::UNO_PARTS.len()
            )
        } else {
            issues.join("; ")
        },
    );

    let n = &d.nano;
    let mut issues = nano_issues(d, &features, &clamp, &everything);
    let seat = d.nano_seat();
    if !at_least(seat, n.min_seat) {
        issues.push(format!(
            "the Nano's edges sit {seat:.2} mm into the V-grooves, less than {:.1} mm",
            n.min_seat
        ));
    }
    let lowest = crate::boards::NANO_PARTS
        .iter()
        .map(|(_, p)| p[1])
        .filter(|&y| y > 0.0)
        .fold(f64::INFINITY, f64::min);
    let highest = crate::boards::NANO_PARTS
        .iter()
        .map(|(_, p)| p[3])
        .fold(0.0, f64::max);
    if seat + 0.3 >= lowest.min(n.w - highest) {
        issues.push("the V-groove walls reach the parts along the Nano's edges".into());
    }
    // The plug must clear everything in the CubeSat, frame included.
    let plug = on_columns(d, &nano_plug(d));
    if overlap(&a.all(d), &plug) > 1e-3 {
        issues.push("something blocks the Nano's USB plug".into());
    }
    r.add(
        "FIT-nano",
        "Nano 33 BLE Sense is held on its edge in V-grooves, with room for its USB plug",
        "Arduino NANO33BLE_SENSE.PcbDoc",
        issues.is_empty(),
        if issues.is_empty() {
            format!(
                "{seat:.2} mm in each groove, {:.1} mm end play, captured in all six directions, plug clear",
                n.end_play
            )
        } else {
            issues.join("; ")
        },
    );

    let column_tops = a.end_bottom.translate(0.0, 0.0, -d.plate_bottom_z());
    let mut issues = stud_issues(d, &column_tops, &plate, &nano_clamp(d));
    // Washers and heads sit flat: nothing else on the plate under them.
    for [x, y] in d.columns {
        let seat = cylinder(
            d.washer.od + 0.5,
            d.washer_h() + d.qt.head_h,
            [x, y, d.plate_t + 0.01],
            None,
        );
        if overlap(&features, &seat) > 1e-3 || keys.iter().any(|k| overlap(k, &seat) > 1e-3) {
            issues.push("a plate stud's washer or head sits on a raised part of the plate".into());
        }
    }
    r.add(
        "FIT-studs",
        "Every quarter-turn stud passes its bar, locks clear, and is held when pulled",
        "CAD assembly",
        issues.is_empty(),
        if issues.is_empty() {
            "4 plate studs into the columns, 2 Nano clamp studs into the pillars".into()
        } else {
            issues.join("; ")
        },
    );

    if overlap(&frame, &board_set) > 1e-3 {
        r.add(
            "FIT-boards",
            "Both boards clear the frame",
            "board outlines",
            false,
            "a board hits the frame".into(),
        );
    } else {
        r.add(
            "FIT-boards",
            "Both boards clear the frame",
            "board outlines",
            true,
            format!(
                "Uno with {:.2} mm margin, Nano with its parts",
                d.min_board_clearance
            ),
        );
    }

    // ---- Printability on a Bambu P1S ----
    let mut print_issues = Vec::new();
    let mut worst = (0.0, "");
    for (name, part) in print_parts(d) {
        let mesh = match Mesh::from_solid(name, &part) {
            Ok(m) => m,
            Err(e) => {
                print_issues.push(e);
                continue;
            }
        };
        if part.decompose().len() != 1 || !mesh.is_closed() {
            print_issues.push(format!("{name} is not one closed solid"));
        }
        let (plo, phi) = mesh.bounds();
        if phi[0] - plo[0] > BED - 10.0
            || phi[1] - plo[1] > BED - 10.0
            || phi[2] - plo[2] > BED_HEIGHT
        {
            print_issues.push(format!("{name} does not fit the P1S"));
        }
        let area = mesh.overhang_area(46.0, 1.0);
        if area > worst.0 {
            worst = (area, name);
        }
        if area > MAX_OVERHANG_MM2 {
            print_issues.push(format!("{name} needs supports ({area:.0} mm2 of overhang)"));
        }
    }
    r.add(
        "PRINT",
        "Every part is one closed solid that prints on a P1S without supports",
        "Bambu Lab P1S, 256 mm build volume",
        print_issues.is_empty(),
        if print_issues.is_empty() {
            format!("largest overhang {:.0} mm2 ({})", worst.0, worst.1)
        } else {
            print_issues.join("; ")
        },
    );

    r
}
