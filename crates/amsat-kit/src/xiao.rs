//! The XIAO plate for the outdoor XIAO case: a Seeed Wi-Fi HaLow board for XIAO (42 x 25 mm,
//! FGH100M-H) with a XIAO ESP32-S3 plugged on top, and a XIAO MG24 Sense with its pin headers
//! soldered. Both go in with the boards' joint: tuck one edge under its lips, swing it down past
//! its snap hooks.
//!
//! Assumed, not measured (check them with the XIAO coupon before printing a plate): board
//! thicknesses (HaLow 1.6 mm, XIAO 1.2 mm), 8.5 mm female headers with the ESP32-S3 centred on
//! the HaLow board, no part within 2 mm of either board's edges on top (the lips reach 0.8 mm), and XIAO pins on a
//! 2.54 mm pitch in two rows 15.24 mm apart, 6 mm long under 2.5 mm header plastic.

use manifold_csg::{CrossSection, Manifold};

use crate::design::{Design, Edge, Rect, Side};
use crate::geom::{EPS, FACETS, bx, offset, rect, union_all};
use crate::joints::{hook_cut, hook_post, lip};
use crate::parts::{cable_notch, column_cuts, plate_outline};

pub const HALOW: Rect = [-23.0, -20.0, 42.0, 25.0];
pub const HALOW_PCB: f64 = 1.6;
pub const HALOW_PADS: f64 = 2.0;
pub const HALOW_LIPS: [Edge; 2] = [Edge::new(Side::Top, 4.5, 1), Edge::new(Side::Top, 34.5, 1)];
pub const HALOW_HOOKS: [Edge; 2] = [
    Edge::new(Side::Bottom, 4.2, 1),
    Edge::new(Side::Bottom, 37.8, -1),
];
const XIAO: [f64; 2] = [21.0, 17.8];
const XIAO_PCB: f64 = 1.2;
const HEADER_F: f64 = 8.5;
/// The MG24 Sense, USB end towards the wall that faces down (low y).
pub const MG24: Rect = [22.0, -14.0, 17.8, 21.0];
const HEADER_M: f64 = 2.5;
const PIN_BELOW: f64 = 6.0;
const ROW: f64 = 1.27;
pub const MG24_LIPS: [Edge; 2] = [
    Edge::new(Side::Bottom, 2.0, 1),
    Edge::new(Side::Bottom, 15.8, 1),
];
pub const MG24_HOOKS: [Edge; 1] = [Edge::new(Side::Top, 8.9, -1)];

pub fn halow_top(d: &Design) -> f64 {
    d.plate_t + HALOW_PADS + HALOW_PCB
}

pub fn mg24_top(d: &Design) -> f64 {
    d.plate_t + HEADER_M + XIAO_PCB
}

/// The MG24's lips are narrow, to fit beside its USB-C socket.
pub fn mg24_joint(d: &Design) -> Design {
    let mut j = d.clone();
    j.lip_w = 3.5;
    j
}

/// The HaLow board with the XIAO ESP32-S3 on its headers (plate coordinates).
pub fn halow_board(d: &Design) -> Manifold {
    let [x0, y0, w, h] = HALOW;
    let z0 = d.plate_t + HALOW_PADS;
    let (cx, cy) = (x0 + w / 2.0, y0 + h / 2.0);
    let (xw, xh) = (XIAO[0] / 2.0, XIAO[1] / 2.0);
    let top = halow_top(d);
    union_all(&[
        bx(x0, y0, z0, x0 + w, y0 + h, top),
        // The ESP32-S3 on its female headers, and their solder joints underneath.
        bx(
            cx - xw,
            cy - xh,
            top - EPS,
            cx + xw,
            cy + xh,
            top + HEADER_F + XIAO_PCB + 3.0,
        ),
        bx(cx - xw, cy - xh, z0 - 1.5, cx + xw, cy + xh, z0 + EPS),
        // Parts on top, 2 mm in from the edges.
        bx(
            x0 + 2.0,
            y0 + 2.0,
            top - EPS,
            x0 + w - 2.0,
            y0 + h - 2.0,
            top + 2.5,
        ),
    ])
}

/// The MG24 Sense with its headers and pins (plate coordinates).
pub fn mg24_board(d: &Design) -> Manifold {
    let [x0, y0, w, h] = MG24;
    let z0 = d.plate_t + HEADER_M;
    let top = mg24_top(d);
    let mut parts = vec![
        bx(x0, y0, z0, x0 + w, y0 + h, top),
        bx(
            x0 + 2.0,
            y0 + 2.0,
            top - EPS,
            x0 + w - 2.0,
            y0 + h - 2.0,
            top + 2.0,
        ),
        // USB-C socket, 1.3 mm past its end.
        bx(
            x0 + w / 2.0 - 4.5,
            y0 - 1.3,
            top - EPS,
            x0 + w / 2.0 + 4.5,
            y0 + 6.1,
            top + 3.2,
        ),
    ];
    for rx in [x0 + ROW, x0 + w - ROW] {
        parts.push(bx(
            rx - 1.25,
            y0 + 1.6,
            d.plate_t,
            rx + 1.25,
            y0 + h - 1.6,
            z0 + EPS,
        ));
        for i in 0..7 {
            let py = y0 + h / 2.0 + (i as f64 - 3.0) * 2.54;
            parts.push(bx(
                rx - 0.32,
                py - 0.32,
                d.plate_t - PIN_BELOW + HEADER_M,
                rx + 0.32,
                py + 0.32,
                d.plate_t + EPS,
            ));
        }
    }
    union_all(&parts)
}

pub fn boards(d: &Design) -> Manifold {
    halow_board(d).union(&mg24_board(d))
}

/// Slots through the plate for the MG24's pins.
pub fn pin_slots(d: &Design) -> Manifold {
    let [x0, y0, w, h] = MG24;
    union_all(&[x0 + ROW, x0 + w - ROW].map(|rx| {
        bx(
            rx - 0.8,
            y0 + 1.6,
            -1.0,
            rx + 0.8,
            y0 + h - 1.6,
            d.plate_t + 1.0,
        )
    }))
}

fn halow_pads(d: &Design) -> Manifold {
    let [x0, y0, w, h] = HALOW;
    let mut pads = Vec::new();
    for px in [x0 + 1.5, x0 + w - 4.5] {
        for py in [y0 + 1.5, y0 + h - 4.5] {
            pads.push(bx(
                px,
                py,
                d.plate_t - EPS,
                px + 3.0,
                py + 3.0,
                d.plate_t + HALOW_PADS,
            ));
        }
    }
    union_all(&pads)
}

/// Corner stops at the HaLow board's short ends.
fn halow_stops(d: &Design) -> Manifold {
    let [x0, y0, w, h] = HALOW;
    let (gap, t) = (0.25, 2.0);
    let mut stops = Vec::new();
    for sx0 in [x0 - gap - t, x0 + w + gap] {
        for (sy0, sy1) in [(y0, y0 + 3.4), (y0 + h - 3.4, y0 + h)] {
            stops.push(bx(
                sx0,
                sy0,
                d.plate_t - EPS,
                sx0 + t,
                sy1,
                halow_top(d) + 0.6,
            ));
        }
    }
    union_all(&stops)
}

/// Pads, stops, lips and hooks for both boards. `deflect` bends the hooks outward.
pub fn features(d: &Design, deflect: f64) -> Manifold {
    let j = mg24_joint(d);
    let mut parts = vec![halow_pads(d), halow_stops(d)];
    for e in &HALOW_LIPS {
        parts.push(lip(d, HALOW, e, halow_top(d)));
    }
    for e in &HALOW_HOOKS {
        parts.push(hook_post(d, HALOW, e, halow_top(d), deflect));
    }
    for e in &MG24_LIPS {
        parts.push(lip(&j, MG24, e, mg24_top(d)));
    }
    for e in &MG24_HOOKS {
        parts.push(hook_post(&j, MG24, e, mg24_top(d), deflect));
    }
    union_all(&parts)
}

pub fn hook_cuts(d: &Design) -> Manifold {
    let mut cuts: Vec<Manifold> = HALOW_HOOKS.iter().map(|e| hook_cut(d, HALOW, e)).collect();
    cuts.extend(MG24_HOOKS.iter().map(|e| hook_cut(&mg24_joint(d), MG24, e)));
    union_all(&cuts)
}

/// The XIAO plate: the payload plate's outline and column cuts, opened up wherever the boards
/// and their joints don't need plate.
pub fn xiao_plate(d: &Design, tooth_r: Option<f64>, deflect: f64) -> Manifold {
    let outline = plate_outline(d);
    let cuts = union_all(&[
        column_cuts(d, tooth_r),
        hook_cuts(d),
        pin_slots(d),
        cable_notch(d),
    ]);
    let feats = features(d, 0.0);
    let [mx, my, mw, mh] = MG24;
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
        .union(&rect(mx - 1.0, my, mx + mw + 1.0, my + mh))
        .union(&rect(
            HALOW[0],
            HALOW[1] + HALOW[3] / 2.0 - 3.0,
            HALOW[0] + HALOW[2],
            HALOW[1] + HALOW[3] / 2.0 + 3.0,
        ));
    let windows = offset(&offset(&outline.difference(&keep), -1.0, 16), 1.0, 16);
    let big: Vec<Vec<[f64; 2]>> = windows
        .to_polygons()
        .into_iter()
        .filter(|p| {
            CrossSection::from_polygons(std::slice::from_ref(p))
                .area()
                .abs()
                > 12.0
        })
        .collect();
    outline
        .extrude(d.plate_t)
        .difference(&cuts)
        .difference(
            &CrossSection::from_polygons(&big)
                .extrude(d.plate_t + 2.0)
                .translate(0.0, 0.0, -1.0),
        )
        .union(&features(d, deflect))
}

/// The XIAO coupon: the plate round both boards' mounts, to try the real boards in first.
pub fn coupon(d: &Design, deflect: f64) -> Manifold {
    let [hx, hy, _, hh] = HALOW;
    let [mx, my, mw, mh] = MG24;
    let (x0, x1) = (hx - 7.0, mx + mw + 5.0);
    let (y0, y1) = (hy - 7.0, (hy + hh).max(my + mh) + 7.0);
    let plate = xiao_plate(d, None, deflect).intersection(&bx(x0, y0, -1.0, x1, y1, 40.0));
    // A rim, so the coupon's cut edge is a solid frame.
    let rim = rect(x0, y0, x1, y1).difference(&rect(x0 + 2.5, y0 + 2.5, x1 - 2.5, y1 - 2.5));
    plate.union(&rim.extrude(d.plate_t))
}
