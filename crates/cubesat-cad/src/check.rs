//! Checks against the CubeSat Design Specification (CDS) Rev 14.1, fit checks
//! between the parts and the boards, and printability checks for a Bambu P1S.

use manifold_csg::Manifold;

use crate::design::Design;
use crate::geom::{bounds, cuboid, cylinder, overlap, union_all};
use crate::mesh::Mesh;
use crate::parts::{Assembly, lug_z, peg_height};

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

/// Bending strain of an Uno clip as the board pushes it out by the lip depth.
pub fn clip_strain(d: &Design) -> f64 {
    let arm = d.uno_post_h + d.pcb_uno + 0.1;
    1.5 * d.clip_t * d.clip_lip / (arm * arm)
}

/// Bending strain of one peg half as the barb squeezes through the hole.
pub fn peg_strain(d: &Design) -> f64 {
    let half = (d.peg_shank - d.peg_slot) / 2.0;
    let arm = peg_height(d, d.lug_t) + d.slot_into_body;
    1.5 * half * (d.peg_barb - d.snap_hole) / 2.0 / (arm * arm)
}

/// Board volumes for fit checks, in the assembled position.
pub struct Boards {
    pub uno_pcb: Manifold,
    pub uno_usb: Manifold,
    pub uno_underside: Manifold,
    pub nano_pcb: Manifold,
    pub nano_pins: Manifold,
}

impl Boards {
    pub fn new(d: &Design) -> Self {
        let o = d.uno_origin();
        let pcb = d.uno_pcb_bottom_z();
        let mut uno_pcb = cuboid(
            [o[0], o[1], pcb],
            [o[0] + d.uno[0], o[1] + d.uno[1], pcb + d.pcb_uno],
        );
        let mut posts = Vec::new();
        for i in 0..4 {
            let [x, y] = d.uno_hole(i);
            uno_pcb = uno_pcb.difference(&cylinder(3.2, d.pcb_uno + 2.0, [x, y, pcb - 1.0], None));
            posts.push(cylinder(
                7.0,
                d.uno_post_h + d.pcb_uno + 2.0,
                [x, y, d.plate_top_z() - 1.0],
                None,
            ));
        }
        // USB-B socket overhangs the left edge by about 6.2 mm and stands 11 mm tall.
        let uno_usb = cuboid(
            [o[0] - 6.2, o[1] + 31.8, pcb + d.pcb_uno],
            [o[0] + 10.0, o[1] + 43.9, pcb + d.pcb_uno + 11.0],
        );
        let uno_underside = cuboid(
            [o[0], o[1], pcb - 2.0],
            [o[0] + d.uno[0], o[1] + d.uno[1], pcb],
        )
        .difference(&union_all(&posts));
        let [nx, ny] = d.nano_center;
        let zn = d.nano_pcb_bottom_z();
        let nano_pcb = cuboid(
            [nx - d.nano[0] / 2.0, ny - d.nano[1] / 2.0, zn],
            [nx + d.nano[0] / 2.0, ny + d.nano[1] / 2.0, zn + d.pcb_nano],
        );
        let rows: Vec<Manifold> = [-1.0, 1.0]
            .iter()
            .map(|s| {
                let row = ny + s * d.nano_row / 2.0;
                cuboid([nx - 19.1, row - 1.3, zn - 8.5], [nx + 19.1, row + 1.3, zn])
            })
            .collect();
        Self {
            uno_pcb,
            uno_usb,
            uno_underside,
            nano_pcb,
            nano_pins: union_all(&rows),
        }
    }
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
        ("payload-plate", crate::parts::payload_plate(d)),
        ("nano-strap", crate::parts::nano_strap(d)),
        ("snap-test-peg", peg),
        ("snap-test-wall", wall),
    ]
}

/// Run every check. A design is ready to print when `Report::passed` is true.
#[allow(clippy::too_many_lines)]
pub fn verify(d: &Design) -> Report {
    let mut r = Report::default();
    let a = Assembly::new(d);
    let boards = Boards::new(d);
    let (h, hz) = (d.half(), d.half_z());
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
    let (alo, ahi) = bounds(&union_all(&[
        a.all(),
        boards.uno_pcb.clone(),
        boards.uno_usb.clone(),
        boards.nano_pcb.clone(),
    ]));
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
    let end_slab = cuboid([-h - 1.0, -h - 1.0, hz - 0.5], [h + 1.0, h + 1.0, hz + 1.0]);
    let end_volume = overlap(&frame, &end_slab);
    let rails_only = (end_volume - 4.0 * rail_end_area * 0.5).abs() < 0.02 * end_volume;
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
    for part in [
        &a.panel_pos_y,
        &a.panel_neg_y,
        &a.end_top,
        &a.end_bottom,
        &a.payload,
        &a.strap,
    ] {
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
        d.uno_pcb_bottom_z() + 5.0,
    ];
    let nano_c = [
        d.nano_center[0],
        d.nano_center[1],
        d.nano_pcb_bottom_z() + 1.0,
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
        format!("about {mass:.0} g with solid PLA and both boards (the slicer's figure at 20% infill is lower)"),
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

    // ---- Snap fits, clips and strain ----
    let overlap_mm = d.peg_barb - d.snap_hole;
    r.add(
        "FIT-snap",
        "Snap barb overlaps its hole enough to latch",
        "first P1S snap test (0.6 mm nozzle)",
        overlap_mm >= d.min_snap_overlap && d.snap_hole > d.peg_shank && d.peg_slot > overlap_mm,
        format!(
            "barb {:.1} mm into a {:.1} mm hole: {overlap_mm:.2} mm overlap, slot {:.1} mm",
            d.peg_barb, d.snap_hole, d.peg_slot
        ),
    );
    let reach = d.clip_lip - d.clip_gap;
    r.add(
        "FIT-clip",
        "Uno clip lips reach over the board edge",
        "Uno R3 outline",
        reach >= d.min_clip_reach,
        format!(
            "{reach:.2} mm over the edge (needs {:.1})",
            d.min_clip_reach
        ),
    );
    r.add(
        "FIT-strain",
        "Clip and peg bending strain within the PLA limit",
        "PLA yield about 3%",
        clip_strain(d) < d.pla_strain_limit && peg_strain(d) < d.pla_strain_limit,
        format!(
            "clip {:.1}%, peg {:.1}% (limit {:.0}%)",
            clip_strain(d) * 100.0,
            peg_strain(d) * 100.0,
            d.pla_strain_limit * 100.0
        ),
    );

    // ---- Parts fit together ----
    let parts = [
        ("+Y panel", &a.panel_pos_y),
        ("-Y panel", &a.panel_neg_y),
        ("+Z end plate", &a.end_top),
        ("-Z end plate", &a.end_bottom),
        ("payload plate", &a.payload),
        ("Nano strap", &a.strap),
    ];
    let mut clashes = Vec::new();
    for i in 0..parts.len() {
        for j in i + 1..parts.len() {
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
            "6 parts, 15 pairs checked".into()
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
    for [x, y] in d.columns {
        let probe = cylinder(
            d.peg_shank - 0.2,
            d.plate_t - 0.2,
            [x, y, d.plate_bottom_z() + 0.1],
            None,
        );
        snaps_ok &= overlap(&a.payload, &probe) < 1e-3
            && overlap(&a.end_bottom, &probe) > 0.5 * probe.volume();
    }
    r.add(
        "FIT-align",
        "Every snap peg passes through its hole",
        "CAD assembly",
        snaps_ok,
        "8 end-plate snaps and 4 payload-column snaps".into(),
    );

    // ---- Boards ----
    let top = d.plate_top_z();
    let features = a.payload.difference(&cuboid([-h, -h, -hz], [h, h, top]));
    let barb_h = peg_height(d, d.plate_t) - d.plate_t;
    let barb_room = union_all(
        &d.columns
            .map(|[x, y]| cylinder(d.peg_barb + 2.0, barb_h + 1.0, [x, y, top], None)),
    );
    let mut board_issues = Vec::new();
    if overlap(&features, &barb_room) > 1e-3 {
        board_issues.push("a plate feature blocks a column snap");
    }
    if overlap(&features, &boards.uno_pcb) > 1e-3 {
        board_issues.push("a plate feature passes through the Uno");
    }
    if overlap(&features, &boards.uno_underside) > 1e-3 {
        board_issues.push("something under the Uno would hit its solder joints");
    }
    if overlap(&features, &boards.nano_pcb) > 1e-3 {
        board_issues.push("a plate feature passes through the Nano");
    }
    if overlap(&features, &boards.nano_pins) > 1e-3 {
        board_issues.push("a Nano rail hits the header pins");
    }
    if overlap(&a.strap, &boards.nano_pcb) > 1e-3 {
        board_issues.push("the strap presses into the Nano");
    }
    let board_set = union_all(&[
        boards.uno_pcb.clone(),
        boards.uno_usb.clone(),
        boards.nano_pcb.clone(),
        boards.nano_pins.clone(),
    ]);
    if overlap(&frame, &board_set) > 1e-3 {
        board_issues.push("a board hits the frame");
    }
    r.add(
        "FIT-boards",
        "Uno R3 and Nano 33 BLE Sense fit with clearance",
        "board outlines",
        board_issues.is_empty(),
        if board_issues.is_empty() {
            "posts, clips, rails, strap, solder-joint zone, header pins, frame".into()
        } else {
            board_issues.join("; ")
        },
    );

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
