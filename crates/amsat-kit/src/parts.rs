//! The kit's printed parts, built from a `Design`. Plate coordinates unless a function says not.
//!
//! - `bottom_with_columns`: AMSAT top/bottom piece with four standoff columns, three ending in
//!   snap pins and one in a locating pin (frame coordinates)
//! - `payload_plate`: holds both boards with built-in lips and snap hooks; snaps onto the columns
//! - `fit_plate`: the payload plate as an open truss, for a fit test or a display piece
//! - `coupon_*`: each joint on its own, a few grams, to test a printer first

use std::path::{Path, PathBuf};

use manifold_csg::{CrossSection, FillRule, Manifold};

use crate::boards::NANO_USB_CENTRE;
use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{
    EPS, FACETS, Mesh, bx, chamfered_square, cone, cyl, offset, poly, rect, union_all,
};
use crate::joints::{finger_cut, hook_cut, hook_post, lip, locating_pin, pin};

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
        let tip = if d.snap_a0(i).is_some() {
            pin(d, x, y, 0.0, d.plate_t)
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

/// Everything cut through the plate: the snap pins' fingers, the locating hole, and the slits
/// that free the board hooks' fingers.
pub fn plate_cuts(d: &Design, tooth_r: Option<f64>) -> Manifold {
    let mut cuts = Vec::new();
    for (i, [x, y]) in d.plate_holes().into_iter().enumerate() {
        cuts.push(match d.snap_a0(i) {
            Some(a0) => finger_cut(d, x, y, a0, tooth_r)
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
    let relief = branding(d)?
        .extrude(d.letter_h + EPS)
        .translate(0.0, 0.0, d.plate_t - EPS);
    Ok(union_all(&[slab, board_features(d, deflect), relief]))
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
    keep = keep.union(&offset(&branding(d)?, 1.0, 16));
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
        &offset(&outline.difference(&keep).difference(&ribs), -1.0, 16),
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
    let pockets = CrossSection::from_polygons(&big)
        .extrude(d.plate_t + 2.0)
        .translate(0.0, 0.0, -1.0);
    Ok(payload_plate(d, None, 0.0)?.difference(&pockets))
}

// ---------- test coupons ----------

/// x0, y0, width, height of the coupon's strip of test board.
pub const COUPON_BOARD: Rect = [-10.0, -7.0, 20.0, 14.0];
const COUPON_PAD_H: f64 = 1.5;
pub const COUPON_PIN_AT: f64 = 5.4;

/// A column stub with a snap pin, on a small base.
pub fn coupon_pin_base(d: &Design) -> Manifold {
    union_all(&[
        bx(-12.0, -12.0, 0.0, 12.0, 12.0, 2.4),
        cyl(d.column_od, 3.0 + EPS, 0.0, 0.0, 2.4 - EPS),
        pin(d, 0.0, 0.0, COUPON_PIN_AT, d.plate_t),
    ])
}

/// A square of plate with the flex fingers, to snap onto the pin.
pub fn coupon_pin_tab(d: &Design, tooth_r: Option<f64>) -> Manifold {
    bx(-12.0, -12.0, 0.0, 12.0, 12.0, d.plate_t).difference(
        &finger_cut(d, 0.0, 0.0, 0.0, tooth_r)
            .extrude(d.plate_t + 2.0)
            .translate(0.0, 0.0, -1.0),
    )
}

pub fn coupon_board_top(d: &Design) -> f64 {
    d.plate_t + COUPON_PAD_H + d.pcb_uno
}

pub fn coupon_edges() -> (Edge, Edge) {
    (
        Edge::new(Side::Left, 7.0, 1),
        Edge::new(Side::Right, 4.0, 1),
    )
}

/// A square of plate with one lip and one snap hook either side of a strip of test board.
pub fn coupon_hook_base(d: &Design, deflect: f64) -> Manifold {
    let board = COUPON_BOARD;
    let [x0, y0, w, h] = board;
    let (lip_edge, hook_edge) = coupon_edges();
    let top = coupon_board_top(d);
    let base = bx(
        x0 - 5.0,
        y0 - 2.0,
        0.0,
        x0 + w + 5.0,
        y0 + d.hook_len + 3.0,
        d.plate_t,
    )
    .difference(&hook_cut(d, board, &hook_edge));
    let pad_top = d.plate_t + COUPON_PAD_H;
    union_all(&[
        base,
        bx(
            x0 + 2.0,
            y0 + 2.0,
            d.plate_t - EPS,
            x0 + 6.0,
            y0 + h - 2.0,
            pad_top,
        ),
        bx(
            x0 + w - 7.0,
            y0 + 2.0,
            d.plate_t - EPS,
            x0 + w - 3.0,
            y0 + h - 2.0,
            pad_top,
        ),
        lip(d, board, &lip_edge, top),
        hook_post(d, board, &hook_edge, top, deflect),
    ])
}

/// The strip of 1.6 mm board, printed flat.
pub fn coupon_board(d: &Design) -> Manifold {
    let [x0, y0, w, h] = COUPON_BOARD;
    bx(x0, y0, 0.0, x0 + w, y0 + h, d.pcb_uno)
}
