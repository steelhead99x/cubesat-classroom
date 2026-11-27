//! The kit's printed parts, built from a `Design`. Plate coordinates unless a function says not.
//!
//! - `bottom_with_columns`: AMSAT top/bottom piece with four standoff columns, three ending in
//!   snap pins and one in a locating pin (frame coordinates)
//! - `payload_plate`: holds both boards with built-in lips and snap hooks; snaps onto the columns
//! - `fit_plate`: the payload plate as an open truss, for a fit test or a display piece
//! - `coupon_*`: the snap pin with a fit ladder of three tabs, and the Nano mount on its own, to
//!   test a printer first

use std::path::{Path, PathBuf};

use manifold_csg::{CrossSection, FillRule, Manifold};

use crate::boards::NANO_USB_CENTRE;
use crate::brand;
use crate::design::Design;
use crate::geom::{
    EPS, FACETS, Mesh, bounds, bx, chamfered_square, cone, cyl, offset, poly, rect, union_all,
};
use crate::joints::{finger_cut, finger_outline, hook_cut, hook_post, lip, locating_pin, pin};

/// The kit's folder: AMSAT's upstream STLs, the lettering, and the output files.
pub fn kit_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../hardware/amsat-cubesatsim")
}

// ---------- frames of reference ----------

/// Plate coordinates to frame coordinates.
pub fn to_frame(d: &Design, solid: &Manifold) -> Manifold {
    solid
        .rotate(180.0, 0.0, 0.0)
        .translate(0.0, 0.0, -d.column_length)
}

pub fn to_plate(d: &Design, solid: &Manifold) -> Manifold {
    solid
        .translate(0.0, 0.0, d.column_length)
        .rotate(180.0, 0.0, 0.0)
}

// ---------- frame piece ----------

pub fn upstream(name: &str) -> Result<Mesh, String> {
    let path = kit_dir().join("upstream").join(name);
    Mesh::read_stl(&path).map_err(|e| e.to_string())
}

/// AMSAT end piece with columns growing from the face's inner side (frame coordinates).
pub fn bottom_with_columns(d: &Design) -> Result<Manifold, String> {
    let mut parts = vec![upstream("cubesatsim_frame_top_bottom.v2.0.stl")?.to_solid()];
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        let column = cyl(
            d.column_od,
            d.column_length + d.face_t / 2.0,
            x,
            y,
            -d.column_length - d.face_t / 2.0,
        );
        let tip = if let Some(angle) = d.snap_angle(i) {
            pin(d, x, y, angle, 0.0, d.plate_t)
        } else {
            locating_pin(d, x, y, 0.0, d.plate_t)
        };
        parts.push(to_frame(d, &column.union(&tip)));
    }
    Ok(union_all(&parts))
}

// ---------- branding ----------

/// Raised lettering, from the outlines in label.json (DejaVu Sans Bold).
pub fn lettering(lines: &[&str]) -> Result<CrossSection, String> {
    let path = kit_dir().join("label.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut out = CrossSection::empty();
    for line in data.as_array().into_iter().flatten() {
        if !lines.contains(&line["text"].as_str().unwrap_or_default()) {
            continue;
        }
        let polys: Vec<Vec<[f64; 2]>> = line["polygons"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                p.as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| [v[0].as_f64().unwrap_or(0.0), v[1].as_f64().unwrap_or(0.0)])
                    .collect()
            })
            .collect();
        out = out.union(&CrossSection::from_polygons_with_fill_rule(
            &polys,
            FillRule::EvenOdd,
        ));
    }
    Ok(out)
}

/// The CubeSat Classroom mark from docs/mark.svg: a cube with an orbit ring in front of and
/// behind it. Flat shapes in plate coordinates.
pub fn mark(size: f64, at: [f64; 2]) -> CrossSection {
    let s = size / 120.0;
    let svg = |pts: &[[f64; 2]]| -> Vec<[f64; 2]> {
        pts.iter()
            .map(|[x, y]| [at[0] + (x - 60.0) * s, at[1] - (y - 60.0) * s])
            .collect()
    };
    let mut faces = CrossSection::empty();
    for pts in [
        [
            [60.00, 28.85],
            [85.38, 43.50],
            [60.00, 58.15],
            [34.62, 43.50],
        ],
        [
            [33.02, 46.27],
            [58.40, 60.92],
            [58.40, 90.23],
            [33.02, 75.58],
        ],
        [
            [61.60, 60.92],
            [86.98, 46.27],
            [86.98, 75.58],
            [61.60, 90.23],
        ],
    ] {
        faces = faces.union(&poly(&svg(&pts)));
    }
    // The orbit ellipse (rx 53, ry 15, turned -15 degrees in the drawing) as a stroke.
    let ring = |t0: f64, t1: f64, width: f64| -> CrossSection {
        let r = (-15f64).to_radians();
        let pts: Vec<[f64; 2]> = (0..=48)
            .map(|i| {
                let t = (t0 + (t1 - t0) * f64::from(i) / 48.0).to_radians();
                let (ex, ey) = (53.0 * t.cos(), 15.0 * t.sin());
                [
                    60.0 + ex * r.cos() - ey * r.sin(),
                    60.0 + ex * r.sin() + ey * r.cos(),
                ]
            })
            .collect();
        let dots: Vec<CrossSection> = svg(&pts)
            .iter()
            .map(|p| CrossSection::circle(width / 2.0 * s, 12).translate(p[0], p[1]))
            .collect();
        let segments: Vec<CrossSection> = dots
            .windows(2)
            .map(|w| CrossSection::batch_hull(&[w[0].clone(), w[1].clone()]))
            .collect();
        CrossSection::batch_union(&segments)
    };
    let hexagon = offset(
        &poly(&svg(&[
            [60.00, 27.00],
            [88.58, 43.50],
            [88.58, 76.50],
            [60.00, 93.00],
            [31.42, 76.50],
            [31.42, 43.50],
        ])),
        2.5 * s,
        12,
    );
    faces
        .difference(&ring(0.0, 180.0, 9.5))
        .union(&ring(0.0, 180.0, 4.6))
        .union(&ring(180.0, 360.0, 4.6).difference(&hexagon))
}

/// Mark and the words CUBESAT CLASSROOM, for the plate's open middle under the Uno.
pub fn branding(d: &Design) -> Result<CrossSection, String> {
    Ok(mark(d.mark_size, d.mark_at).union(&lettering(&["CUBESAT", "CLASSROOM"])?))
}

// ---------- payload plate ----------

pub fn nano_pads(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.nano_rect();
    let mut pads = Vec::new();
    for px in [x0 + 1.5, x0 + w - 4.5] {
        for py in [y0 + 1.5, y0 + h - 4.5] {
            pads.push(bx(
                px,
                py,
                d.plate_t - EPS,
                px + 3.0,
                py + 3.0,
                d.nano_bottom(),
            ));
        }
    }
    union_all(&pads)
}

/// Stops at the Nano's four corners, clear of its USB plug.
pub fn nano_stops(d: &Design) -> Manifold {
    let [x0, y0, w, h] = d.nano_rect();
    let (gap, t) = (d.nano_end_play / 2.0, d.nano_stop_t);
    let plug_y0 = y0 + NANO_USB_CENTRE - d.plug[2] / 2.0 - 0.25;
    let plug_y1 = y0 + NANO_USB_CENTRE + d.plug[2] / 2.0 + 0.25;
    let mut stops = Vec::new();
    for sx0 in [x0 - gap - t, x0 + w + gap] {
        for (sy0, sy1) in [
            (y0, (y0 + 3.4).min(plug_y0)),
            ((y0 + h - 3.4).max(plug_y1), y0 + h),
        ] {
            stops.push(bx(
                sx0,
                sy0,
                d.plate_t - EPS,
                sx0 + t,
                sy1,
                d.nano_top() + 0.6,
            ));
        }
    }
    union_all(&stops)
}

/// Everything standing on the plate's board side, without the plate itself. `deflect` bends
/// every snap hook outward, for the checks.
pub fn board_features(d: &Design, deflect: f64) -> Manifold {
    let top = d.plate_t;
    let [x0, y0, _, _] = d.uno_rect();
    let pcb = d.uno_bottom();
    let pin_h = d.pcb_uno + d.uno_pin_above;
    let mut parts = Vec::new();
    for [hx, hy] in d.uno_holes {
        let (x, y) = (x0 + hx, y0 + hy);
        parts.push(cyl(
            7.0,
            d.uno_post_h - d.uno_post_top_h + EPS,
            x,
            y,
            top - EPS,
        ));
        parts.push(cyl(
            d.uno_post_top,
            d.uno_post_top_h + EPS,
            x,
            y,
            pcb - d.uno_post_top_h - EPS,
        ));
        parts.push(cyl(d.uno_pin, pin_h - 0.4 + 2.0 * EPS, x, y, pcb - EPS));
        parts.push(cone(
            d.uno_pin,
            d.uno_pin - 0.6,
            0.4,
            x,
            y,
            pcb + pin_h - 0.4,
        ));
    }
    for e in &d.uno_lips {
        parts.push(lip(d, d.uno_rect(), e, d.uno_top()));
    }
    for e in &d.uno_hooks {
        parts.push(hook_post(d, d.uno_rect(), e, d.uno_top(), deflect));
    }
    parts.push(nano_pads(d));
    parts.push(nano_stops(d));
    for e in &d.nano_lips {
        parts.push(lip(d, d.nano_rect(), e, d.nano_top()));
    }
    for e in &d.nano_hooks {
        parts.push(hook_post(d, d.nano_rect(), e, d.nano_top(), deflect));
    }
    union_all(&parts)
}

/// The snap pins' keyed slots and fingers and the locating hole, as cut through any plate that
/// snaps onto the columns. `tooth_r` moves the teeth, for the checks.
pub fn column_cuts(d: &Design, tooth_r: Option<f64>) -> Manifold {
    let tooth_r = tooth_r.unwrap_or(d.tooth_r);
    let mut cuts = Vec::new();
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        cuts.push(match d.snap_angle(i) {
            Some(angle) => finger_cut(d, x, y, angle, tooth_r, d.column_fit(i))
                .extrude(d.plate_t + 2.0)
                .translate(0.0, 0.0, -1.0),
            None => cyl(
                2.0 * (d.pin_r + d.locate_clear),
                d.plate_t + 2.0,
                x,
                y,
                -1.0,
            ),
        });
    }
    union_all(&cuts)
}

/// A notch in the plate's top edge for cables: the battery's, and a USB cable to the boards.
pub fn cable_notch(d: &Design) -> Manifold {
    let e = d.plate / 2.0;
    bx(-23.0, e - 6.0, -1.0, -13.0, e + 1.0, d.plate_t + 1.0)
}

/// Everything cut through the payload plate: the column cuts, the cable notch, and the slits
/// that free the board hooks' fingers.
pub fn plate_cuts(d: &Design, tooth_r: Option<f64>) -> Manifold {
    let mut cuts = vec![column_cuts(d, tooth_r), cable_notch(d)];
    for (board, edges) in [(d.uno_rect(), &d.uno_hooks), (d.nano_rect(), &d.nano_hooks)] {
        for e in edges {
            cuts.push(hook_cut(d, board, e));
        }
    }
    union_all(&cuts)
}

pub fn plate_outline(d: &Design) -> CrossSection {
    chamfered_square(d.plate, d.plate_chamfer)
}

pub fn payload_plate(d: &Design, tooth_r: Option<f64>, deflect: f64) -> Result<Manifold, String> {
    let slab = plate_outline(d)
        .extrude(d.plate_t)
        .difference(&plate_cuts(d, tooth_r));
    // The branding is a second-colour inlay in the top face (see `plate_brand`).
    Ok(union_all(&[slab, board_features(d, deflect)]))
}

/// The payload plate's logo inlay: the mark and words in the open middle under the Uno, flush
/// in the top face. The fit plate keeps the same spot.
pub fn plate_brand(d: &Design) -> Result<Manifold, String> {
    Ok(branding(d)?
        .extrude(d.letter_h + 1.0)
        .translate(0.0, 0.0, d.plate_t - d.letter_h))
}

// ---------- fit plate ----------

const FIT_RIM: f64 = 3.0;
const FIT_PAD: f64 = 1.6;
const TRUSS_PITCH: f64 = 34.0;
const TRUSS_RIB: f64 = 2.4;
const MIN_POCKET: f64 = 12.0;

/// The payload plate's mounts, joints and branding, unchanged and in the same places, on an
/// open triangular truss instead of a solid deck.
pub fn fit_plate(d: &Design) -> Result<Manifold, String> {
    let outline = plate_outline(d);
    let mut keep =
        outline.difference(&outline.offset(-FIT_RIM, manifold_csg::JoinType::Miter, 2.0, 0));
    keep = keep.union(&offset(
        &board_features(d, 0.0).slice_to_cross_section(d.plate_t + 0.1),
        FIT_PAD,
        FACETS,
    ));
    keep = keep.union(&offset(
        &plate_cuts(d, None).slice_to_cross_section(d.plate_t / 2.0),
        2.5,
        FACETS,
    ));
    keep = keep.union(&offset(&branding(d)?, brand::MARGIN + 1.0, 16));
    let pockets = isogrid_pockets(d, &outline, &keep);
    Ok(payload_plate(d, None, 0.0)?.difference(&pockets))
}

/// The kit's one way of lightening a plate: everything in `outline` that `keep` doesn't need
/// becomes triangular isogrid pockets between 2.4 mm ribs at 60 degrees, with 1 mm fillets in
/// their corners, through the plate. Tiny slivers stay solid.
pub fn isogrid_pockets(d: &Design, outline: &CrossSection, keep: &CrossSection) -> Manifold {
    let mut ribs = CrossSection::empty();
    let pitch = TRUSS_PITCH * 3f64.sqrt() / 2.0;
    for angle in [0.0, 60.0, 120.0] {
        for k in -6..=6 {
            let y = f64::from(k) * pitch;
            ribs = ribs.union(
                &rect(
                    -1.5 * d.plate,
                    y - TRUSS_RIB / 2.0,
                    1.5 * d.plate,
                    y + TRUSS_RIB / 2.0,
                )
                .rotate(angle),
            );
        }
    }
    let pockets = offset(
        &offset(&outline.difference(keep).difference(&ribs), -1.0, 16),
        1.0,
        16,
    )
    .simplify(0.02);
    let big: Vec<Vec<[f64; 2]>> = pockets
        .to_polygons()
        .into_iter()
        .filter(|p| {
            CrossSection::from_polygons(std::slice::from_ref(p))
                .area()
                .abs()
                > MIN_POCKET
        })
        .collect();
    CrossSection::from_polygons(&big)
        .extrude(d.plate_t + 2.0)
        .translate(0.0, 0.0, -1.0)
}

// ---------- test coupons ----------

/// Fits the ladder's three tabs try, and how many dots mark each: the kit's (it fit best on the
/// first print), then two looser ones for printers that make holes small.
pub const FIT_LADDER: [(f64, usize); 3] = [(0.1, 1), (0.2, 2), (0.3, 3)];
pub const COUPON_PIN_AT: f64 = 5.4;

/// A column stub with a snap pin, on a small base.
pub fn coupon_pin_base(d: &Design) -> Manifold {
    union_all(&[
        bx(-10.0, -10.0, 0.0, 10.0, 10.0, 2.4),
        cyl(d.column_od, 3.0 + EPS, 0.0, 0.0, 2.4 - EPS),
        pin(d, 0.0, 0.0, 0.0, COUPON_PIN_AT, d.plate_t),
    ])
}

/// A piece of plate with the keyed slot and fingers, at a fit clearance, marked with dots on its
/// rigid tongue. `tooth_r` moves the teeth, for the checks.
/// How far a pin tab's grip reaches past the fork.
pub const TAB_GRIP: f64 = 11.0;

pub fn coupon_pin_tab(d: &Design, fit: f64, dots: usize, tooth_r: Option<f64>) -> Manifold {
    let tooth_r = tooth_r.unwrap_or(d.tooth_r + fit - d.pin_fit);
    let [lo, hi] = {
        let b = finger_outline(d, 0.0, 0.0, 0.0, fit).bounds();
        [b.min(), b.max()]
    };
    // A grip past the fork's root end, which also carries the logo.
    let slab = bx(
        lo[0] - 2.0,
        lo[1] - 2.0,
        0.0,
        hi[0] + 2.0,
        hi[1] + 2.0 + TAB_GRIP,
        d.plate_t,
    )
    .difference(
        &finger_cut(d, 0.0, 0.0, 0.0, tooth_r, fit)
            .extrude(d.plate_t + 2.0)
            .translate(0.0, 0.0, -1.0),
    );
    let marks: Vec<Manifold> = (0..dots)
        .map(|i| {
            cyl(
                1.6,
                0.6 + EPS,
                0.0,
                -d.pin_flat - fit - 2.6 - 2.4 * i as f64,
                d.plate_t - EPS,
            )
        })
        .collect();
    union_all(&[vec![slab], marks].concat())
}

/// The plate's Nano 33 BLE Sense mount on its own: the same pads, corner stops, lips and snap
/// hooks, in the same places (plate coordinates), to try the bare board in.
pub fn coupon_nano_base(d: &Design, deflect: f64) -> Manifold {
    let rect_ = d.nano_rect();
    let mut features = vec![nano_pads(d), nano_stops(d)];
    for e in &d.nano_lips {
        features.push(lip(d, rect_, e, d.nano_top()));
    }
    for e in &d.nano_hooks {
        features.push(hook_post(d, rect_, e, d.nano_top(), deflect));
    }
    let cuts = union_all(
        &d.nano_hooks
            .iter()
            .map(|e| hook_cut(d, rect_, e))
            .collect::<Vec<_>>(),
    );
    let features = union_all(&features);
    let (lo, hi) = bounds(&union_all(&[features.clone(), cuts.clone()]));
    let outline = rect(lo[0] - 2.5, lo[1] - 2.5, hi[0] + 2.5, hi[1] + 2.5);
    // Open everything but a rim, the mount's features with a margin, the hooks' surrounds and a
    // rib across the middle that ties the lips to the hooks.
    let [x0, _, w, _] = rect_;
    let mid = x0 + w / 2.0;
    let keep = outline
        .difference(&outline.offset(-3.0, manifold_csg::JoinType::Miter, 2.0, 0))
        .union(&offset(
            &features.slice_to_cross_section(d.plate_t + 0.1),
            2.5,
            FACETS,
        ))
        .union(&offset(
            &cuts.slice_to_cross_section(d.plate_t / 2.0),
            2.5,
            FACETS,
        ))
        .union(&rect(mid - 2.5, lo[1], mid + 2.5, hi[1]));
    let pockets = offset(&offset(&outline.difference(&keep), -1.0, 16), 1.0, 16);
    let big: Vec<Vec<[f64; 2]>> = pockets
        .to_polygons()
        .into_iter()
        .filter(|p| {
            CrossSection::from_polygons(std::slice::from_ref(p))
                .area()
                .abs()
                > MIN_POCKET
        })
        .collect();
    let slab = outline.extrude(d.plate_t).difference(&cuts).difference(
        &CrossSection::from_polygons(&big)
            .extrude(d.plate_t + 2.0)
            .translate(0.0, 0.0, -1.0),
    );
    // A raised arrow on the rim points to where the Nano's USB end goes, low enough for its
    // plug to pass over.
    let (ax, ay) = (hi[0] + 1.0, y0_mid(d));
    let arrow = poly(&[[ax - 1.0, ay - 1.3], [ax + 1.0, ay], [ax - 1.0, ay + 1.3]])
        .extrude(0.6 + EPS)
        .translate(0.0, 0.0, d.plate_t - EPS);
    union_all(&[slab, features, arrow])
}

/// The middle of the Nano's long side.
fn y0_mid(d: &Design) -> f64 {
    let [_, y0, _, h] = d.nano_rect();
    y0 + h / 2.0
}
