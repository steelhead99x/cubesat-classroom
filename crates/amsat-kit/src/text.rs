//! Lettering for the coupon labels, in the vendored DejaVu Sans Bold (the font label.json was
//! drawn from), for any words.
//!
//! Glyph outlines are flattened to polygons and filled non-zero, the way the font rasterizes.
//! `size` is the em size in mm. Two things keep small bold lettering printable on a 0.6 mm
//! nozzle as well as a 0.4 mm one:
//! - letters are spaced out by `size / 8`, so neighbours keep plastic between them;
//! - letters are grown by `BOLD`, so their strokes stay wide enough at small sizes;
//! - the outline is closed by `CLOSE` (grown, then shrunk back), which fills the sharp
//!   notches inside letters such as N and M, where the plastic between strokes tapers away.

use manifold_csg::{CrossSection, FillRule, JoinType};
use std::sync::OnceLock;
use ttf_parser::{Face, OutlineBuilder};

/// DejaVu Sans Bold 2.37, Bitstream Vera licence (see `fonts/LICENSE-DejaVu.txt`).
const FONT: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");

/// Notches narrower than twice this are filled.
pub const CLOSE: f64 = 0.4;
/// Letters are grown by this all round, so small lettering's strokes reach `brand::MIN_STROKE`.
pub const BOLD: f64 = 0.12;

fn face() -> &'static Face<'static> {
    static FACE: OnceLock<Face<'static>> = OnceLock::new();
    FACE.get_or_init(|| Face::parse(FONT, 0).expect("the vendored font parses"))
}

struct Flatten {
    contours: Vec<Vec<[f64; 2]>>,
    current: Vec<[f64; 2]>,
    scale: f64,
    pen_x: f64,
    last: [f64; 2],
}

const STEPS: usize = 8;

impl Flatten {
    fn pt(&self, x: f32, y: f32) -> [f64; 2] {
        [
            self.pen_x + f64::from(x) * self.scale,
            f64::from(y) * self.scale,
        ]
    }
    fn push(&mut self, p: [f64; 2]) {
        self.current.push(p);
        self.last = p;
    }
    fn finish(&mut self) {
        if self.current.len() > 2 {
            self.contours.push(std::mem::take(&mut self.current));
        }
        self.current.clear();
    }
}

impl OutlineBuilder for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish();
        let p = self.pt(x, y);
        self.push(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.push(p);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (p0, c, p1) = (self.last, self.pt(x1, y1), self.pt(x, y));
        for i in 1..=STEPS {
            let t = i as f64 / STEPS as f64;
            let u = 1.0 - t;
            self.push([
                u * u * p0[0] + 2.0 * u * t * c[0] + t * t * p1[0],
                u * u * p0[1] + 2.0 * u * t * c[1] + t * t * p1[1],
            ]);
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (p0, c1, c2, p1) = (self.last, self.pt(x1, y1), self.pt(x2, y2), self.pt(x, y));
        for i in 1..=STEPS {
            let t = i as f64 / STEPS as f64;
            let u = 1.0 - t;
            let b = |k: usize| {
                u * u * u * p0[k]
                    + 3.0 * u * u * t * c1[k]
                    + 3.0 * u * t * t * c2[k]
                    + t * t * t * p1[k]
            };
            self.push([b(0), b(1)]);
        }
    }
    fn close(&mut self) {
        self.finish();
    }
}

/// One line of lettering, `size` mm em, centred on the origin, spaced and closed as above.
pub fn line(text: &str, size: f64) -> CrossSection {
    let face = face();
    let scale = size / f64::from(face.units_per_em());
    let mut pen = Flatten {
        contours: Vec::new(),
        current: Vec::new(),
        scale,
        pen_x: 0.0,
        last: [0.0, 0.0],
    };
    for ch in text.chars() {
        let Some(id) = face.glyph_index(ch) else {
            continue;
        };
        face.outline_glyph(id, &mut pen);
        pen.finish();
        pen.pen_x += f64::from(face.glyph_hor_advance(id).unwrap_or(0)) * scale + size / 8.0;
    }
    let shape = CrossSection::from_polygons_with_fill_rule(&pen.contours, FillRule::NonZero)
        .offset(CLOSE, JoinType::Round, 2.0, 16)
        .offset(BOLD - CLOSE, JoinType::Round, 2.0, 16);
    if shape.is_empty() {
        return shape;
    }
    let b = shape.bounds_rect2();
    shape.translate(-(b.min_x + b.max_x) / 2.0, -(b.min_y + b.max_y) / 2.0)
}

/// Width and height of a shape, mm.
pub fn size_of(shape: &CrossSection) -> (f64, f64) {
    if shape.is_empty() {
        return (0.0, 0.0);
    }
    let b = shape.bounds_rect2();
    (b.max_x - b.min_x, b.max_y - b.min_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_have_ink_and_grow_with_size() {
        let small = line("TESTS PIN HOLD", 4.5);
        let big = line("TESTS PIN HOLD", 9.0);
        assert!(small.area() > 10.0);
        let (ws, hs) = size_of(&small);
        let (wb, hb) = size_of(&big);
        assert!(wb > 1.9 * ws && hb > 1.9 * hs, "{ws}x{hs} vs {wb}x{hb}");
        let b = small.bounds_rect2();
        assert!(
            (b.min_x + b.max_x).abs() < 1e-6 && (b.min_y + b.max_y).abs() < 1e-6,
            "centred"
        );
    }

    #[test]
    fn unknown_characters_are_skipped() {
        assert!(line("\u{fffe}", 5.0).is_empty());
    }
}
