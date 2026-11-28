//! Label tiles for the coupons the cases share (the snap pin, tripod thread, lid latch and XIAO
//! board mounts): a thin flag joined to one side of the coupon, carrying the CubeSat Classroom
//! mark and a few words saying what the coupon tests and how to try it. Lettering and mark are
//! printed in the second filament and stand `logo_raise` proud of the tile, so they read on a
//! one-colour print too.
//!
//! The tile sits beside the joint, never on it: it's at the bed, on a side that no mating part,
//! board or insertion path reaches. `brand::place` checks the lettering's strokes, gaps, margins
//! and the clear room above it, as it does every logo.

use manifold_csg::{CrossSection, Manifold};

use crate::design::Design;
use crate::geom::{bounds, bx};
use crate::{parts, text};

/// Body text em size, mm: about 3.3 mm capitals, the smallest that keeps strokes over
/// `brand::MIN_STROKE`.
pub const TEXT: f64 = 4.0;
/// The first line, a little bigger.
pub const TITLE: f64 = 4.6;
/// Tile thickness, unless the coupon sets its own to sit flush with its base.
pub const TILE_T: f64 = 2.0;
/// Tile edge to lettering.
const PAD: f64 = 2.0;
/// How far the tile reaches into the coupon, so they print as one piece.
const OVERLAP: f64 = 1.0;
/// Mark to lettering.
const MARK_GAP: f64 = 2.0;

/// Which side of the coupon the tile goes on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    MinusY,
    PlusY,
}

pub struct Label {
    pub part: &'static str,
    pub lines: &'static [&'static str],
    pub side: Side,
    /// Tile thickness; `None` is `TILE_T`.
    pub thick: Option<f64>,
}

/// Every labelled coupon. The words are short: what it tests, then how to try it.
pub const LABELS: &[Label] = &[
    Label {
        part: "coupon-xiao",
        lines: &["BOARD MOUNTS", "HALOW + XIAO MG24", "CLICK EACH BOARD IN"],
        side: Side::MinusY,
        thick: None,
    },
    Label {
        part: "coupon-tripod-thread",
        lines: &["TRIPOD THREAD", "1/4-20 SCREW IN", "DOT: LOOSER"],
        side: Side::MinusY,
        thick: None,
    },
    Label {
        part: "coupon-latch-box",
        lines: &["LID LATCH: BOX", "PRESS THE LID ON", "UNTIL IT CLICKS"],
        side: Side::MinusY,
        thick: None,
    },
    Label {
        part: "coupon-latch-lid",
        lines: &["LID LATCH: LID", "PRESS ONTO BOX", "UNTIL IT CLICKS"],
        side: Side::MinusY,
        thick: None,
    },
    Label {
        part: "coupon-pin-base",
        lines: &["SNAP PIN", "PUSH A TAB ON"],
        side: Side::MinusY,
        thick: Some(2.4),
    },
    Label {
        part: "coupon-pin-tab",
        lines: &["FIT 1", "TIGHTEST"],
        side: Side::PlusY,
        thick: Some(-1.0),
    },
    Label {
        part: "coupon-pin-tab-2-dots",
        lines: &["FIT 2", "MIDDLE"],
        side: Side::PlusY,
        thick: Some(-1.0),
    },
    Label {
        part: "coupon-pin-tab-3-dots",
        lines: &["FIT 3", "LOOSEST"],
        side: Side::PlusY,
        thick: Some(-1.0),
    },
];

pub fn for_part(part: &str) -> Option<&'static Label> {
    LABELS.iter().find(|l| l.part == part)
}

/// The label's mark and lettering, centred on the origin: the mark on the left, as tall as the
/// lettering, then the lines left-aligned.
pub fn artwork(lines: &[&str]) -> CrossSection {
    let mut rows = Vec::new();
    let mut y = 0.0;
    for (i, l) in lines.iter().enumerate() {
        let size = if i == 0 { TITLE } else { TEXT };
        let s = text::line(l, size);
        let (lw, lh) = text::size_of(&s);
        // Rows are spaced by cap height plus a gap well over brand::MIN_GAP.
        let pitch = if i == 0 { 0.0 } else { lh + 1.6 };
        y -= pitch;
        rows.push((s, lw, y));
    }
    let top = rows.first().map_or(0.0, |r| text::size_of(&r.0).1 / 2.0);
    let bottom = y - rows.last().map_or(0.0, |r| text::size_of(&r.0).1 / 2.0);
    let h = top - bottom;
    // The mark's drawing is about 0.6 of its size tall.
    let m = parts::mark((h / 0.6).min(26.0), [0.0, 0.0]);
    let mr = m.bounds_rect2();
    let mw = mr.max_x - mr.min_x;
    let mut art = m.translate(
        -mr.min_x,
        (top + bottom) / 2.0 - (mr.min_y + mr.max_y) / 2.0,
    );
    let x0 = mw + MARK_GAP;
    for (s, lw, y) in rows {
        art = art.union(&s.translate(x0 + lw / 2.0, y));
    }
    let r = art.bounds_rect2();
    art.translate(-(r.min_x + r.max_x) / 2.0, -(r.min_y + r.max_y) / 2.0)
}

/// The coupon with its tile, and the label's artwork in place with the height of its face.
pub fn add(d: &Design, solid: &Manifold, l: &Label) -> (Manifold, CrossSection, f64) {
    let (lo, hi) = bounds(solid);
    let t = match l.thick {
        None => TILE_T,
        // Negative: as thick as the plate the coupon is cut from.
        Some(t) if t < 0.0 => d.plate_t,
        Some(t) => t,
    };
    let art = artwork(l.lines);
    let r = art.bounds_rect2();
    let (aw, ah) = (r.max_x - r.min_x, r.max_y - r.min_y);
    let cx = (lo[0] + hi[0]) / 2.0;
    let (x0, x1) = (
        (cx - aw / 2.0 - PAD).min(lo[0]),
        (cx + aw / 2.0 + PAD).max(hi[0]),
    );
    let depth = ah + 2.0 * PAD;
    let (y0, y1, ay) = match l.side {
        Side::MinusY => (lo[1] - depth, lo[1] + OVERLAP, lo[1] - depth / 2.0),
        Side::PlusY => (hi[1] - OVERLAP, hi[1] + depth, hi[1] + depth / 2.0),
    };
    let z = lo[2];
    let tile = bx(x0, y0, z, x1, y1, z + t);
    (solid.union(&tile), art.translate(cx, ay), z + t)
}

/// Add every labelled coupon's tile: the label artwork and face height for each one tiled.
pub fn apply(
    d: &Design,
    solids: &mut [(&'static str, Manifold)],
) -> Vec<(&'static str, CrossSection, f64)> {
    let mut out = Vec::new();
    for (name, solid) in solids.iter_mut() {
        if let Some(l) = for_part(name) {
            let (s, art, z) = add(d, solid, l);
            *solid = s;
            out.push((*name, art, z));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brand;

    /// A label whose part was renamed would otherwise be skipped without a word, and the coupon
    /// would print bare.
    #[test]
    fn every_label_names_a_printed_part() {
        use crate::bambu::{CLASSROOM, OUTDOOR, OUTDOOR_XIAO};
        for l in LABELS {
            let printed = [&CLASSROOM, &OUTDOOR, &OUTDOOR_XIAO]
                .iter()
                .any(|k| k.parts.iter().any(|p| p.source == l.part));
            assert!(printed, "{}: no kit prints this part", l.part);
        }
    }

    #[test]
    fn every_label_prints() {
        for l in LABELS {
            let a = artwork(l.lines);
            assert!(brand::strokes_ok(&a), "{}: strokes too thin", l.part);
            assert!(brand::gaps_ok(&a), "{}: gaps too narrow", l.part);
        }
    }
}
