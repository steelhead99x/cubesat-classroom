//! Two-colour CubeSat Classroom branding on every printed part.
//!
//! The mark, with the words CUBESAT CLASSROOM where they fit, is inlaid into one flat face of
//! the part as a second volume, printed in the second filament. The inlay is `letter_h` deep
//! (three 0.3 mm layers), so the printer only swaps filament on those few layers.
//!
//! On an upward face the logo also stands `logo_raise` proud of the face, so it prints with
//! crisp side walls and reads better than a flush one, unless something mounted on the part
//! (a board, the battery) would come within `RAISE_CLEAR` of it: then it stays flush. A logo on
//! the bed face is always flush, since nothing can stand off the build plate. The STL holds the
//! part without the raised logo; the Bambu project holds both.
//!
//! Coupon label tiles carry words as well as the logo (`Spot::Label`); their lettering is
//! always raised, and a label that can't be raised is refused.
//!
//! Each part's face and spot are found by code, then checked:
//! - every point of the inlay is at least `MARGIN` from any edge, hole, slit or taller feature,
//!   so it never reaches a flex finger (fingers are only 2–2.4 mm wide between slits);
//! - there is solid plastic behind the whole inlay, at least 0.3 mm of it;
//! - no stroke is thinner than `MIN_STROKE`, and no strip of plastic between two inlay areas is
//!   narrower than `MIN_GAP`, so a 0.6 mm nozzle prints it (and a 0.4 mm one too).

use manifold_csg::{CrossSection, JoinType, Manifold};

use crate::design::Design;
use crate::geom::{EPS, Mesh, offset};
use crate::parts::{branding, mark};

/// Clear distance from the inlay to any edge, hole, slit or taller feature.
pub const MARGIN: f64 = 1.0;
/// The thinnest stroke a 0.6 mm nozzle prints cleanly (a 0.4 mm one prints it too).
pub const MIN_STROKE: f64 = 0.65;
/// The narrowest strip of plastic left between two inlay areas, so the colours stay apart.
pub const MIN_GAP: f64 = 0.8;
/// Clear room above a raised logo to anything mounted on the part.
pub const RAISE_CLEAR: f64 = 0.3;
/// Plastic left behind the inlay.
pub const BACKING: f64 = 0.3;

/// Logo widths to try, largest first: the mark with the words, then the mark alone.
const FULL_WIDTHS: [f64; 9] = [80.0, 70.0, 60.0, 52.0, 45.0, 40.0, 35.0, 31.0, 28.0];
const MARK_SIZES: [f64; 6] = [26.0, 22.0, 19.0, 16.0, 14.0, 12.5];
/// For the smallest parts: the cube alone, its three faces split by 0.5 mm lines.
const CUBE_SIZES: [f64; 5] = [10.0, 9.0, 8.0, 7.0, 6.0];
const CUBE_GAP: f64 = 0.9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Face {
    /// The face on the build plate. The logo is mirrored so it reads right side up when the
    /// part is turned over.
    Bed,
    /// The biggest upward-facing flat face.
    Top,
}

/// Where a part's logo goes: found by searching the faces in order, or fixed by its design.
pub enum Spot {
    Search(&'static [Face]),
    /// An inlay prism already placed in the part's print coordinates, and the face it's on.
    Fixed(Manifold, Face),
    /// A coupon label: lettering and logo, in place, for the upward face at this height.
    /// Always raised; refused if it can't be.
    Label(CrossSection, f64),
}

pub struct Brand {
    /// The part without the inlay.
    pub body: Manifold,
    /// The inlay, printed in the other filament.
    pub accent: Manifold,
    pub face: Face,
    /// The logo's width, and whether it carries the words.
    pub width: f64,
    pub words: bool,
    /// How far the logo stands proud of its face (0 when flush).
    pub raise: f64,
}

fn centred(s: &CrossSection) -> (CrossSection, f64, f64) {
    let r = s.bounds_rect2();
    (
        s.translate(-(r.min_x + r.max_x) / 2.0, -(r.min_y + r.max_y) / 2.0),
        r.max_x - r.min_x,
        r.max_y - r.min_y,
    )
}

/// The logo candidates, largest first: (shape centred on the origin, width, words).
fn candidates(d: &Design) -> Result<Vec<(CrossSection, f64, bool)>, String> {
    let (full, w0, _) = centred(&branding(d)?);
    let mut out = Vec::new();
    for w in FULL_WIDTHS {
        out.push((full.scale(w / w0, w / w0), w, true));
    }
    for s in MARK_SIZES {
        let (m, w, _) = centred(&mark(s, [0.0, 0.0]));
        out.push((m, w, false));
    }
    for s in CUBE_SIZES {
        out.push((cube(s), s, false));
    }
    Ok(out)
}

/// The mark's cube on its own, `size` across its corners: a hexagon split into three faces.
pub fn cube(size: f64) -> CrossSection {
    let r = size / 2.0;
    let hex: Vec<[f64; 2]> = (0..6)
        .map(|i| {
            let a = (90.0 + 60.0 * f64::from(i)).to_radians();
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    let mut lines = CrossSection::empty();
    for a in [90.0f64, 210.0, 330.0] {
        lines = lines.union(
            &crate::geom::rect(0.0, -CUBE_GAP / 2.0, r + 1.0, CUBE_GAP / 2.0).rotate(a + 180.0),
        );
    }
    crate::geom::poly(&hex).difference(&lines)
}

/// The thinnest stroke is at least `MIN_STROKE` if opening the shape by half of it loses
/// almost nothing.
pub fn strokes_ok(shape: &CrossSection) -> bool {
    let h = MIN_STROKE / 2.0;
    let opened = offset(&offset(shape, -h, 16), h, 16);
    opened.area() >= 0.93 * shape.area()
}

/// Check gaps locally. Growing both sides by half the gap must keep separate inlays apart.
/// Closing also exposes narrow slots within one inlay. Allow small fills at inside corners.
pub fn gaps_ok(shape: &CrossSection) -> bool {
    let h = MIN_GAP / 2.0;
    let grown = offset(shape, h - 1e-6, 32);
    if grown.decompose().len() != shape.decompose().len() {
        return false;
    }
    offset(&grown, -h, 32)
        .difference(shape)
        .decompose()
        .iter()
        .all(|fill| {
            let r = fill.bounds_rect2();
            // Ignore numerical traces and small patches left by rounding an inside corner.
            fill.area() < 1e-4
                || (fill.area() <= h * h && (r.max_x - r.min_x).max(r.max_y - r.min_y) <= MIN_GAP)
        })
}

/// The height of the biggest upward-facing flat face.
fn top_face(solid: &Manifold) -> f64 {
    let mesh = Mesh::from_solid(solid);
    let mut areas: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    for (t, tri) in mesh.tris.iter().enumerate() {
        if mesh.normal(t)[2] > 0.9999 {
            let [a, b, c] = tri.map(|i| mesh.verts[i as usize]);
            let area = ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0;
            *areas.entry((a[2] * 100.0).round() as i64).or_insert(0.0) += area;
        }
    }
    let best = areas
        .iter()
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
        .map_or(0, |(z, _)| *z);
    best as f64 / 100.0
}

/// The face's height, and the flat area where an inlay could go: solid at the face and behind
/// the inlay, and nothing standing on it.
fn footprint(solid: &Manifold, face: Face, depth: f64) -> (f64, CrossSection) {
    let (lo, _) = crate::geom::bounds(solid);
    match face {
        Face::Bed => {
            let z = lo[2];
            let f = solid
                .slice_to_cross_section(z + 0.05)
                .intersection(&solid.slice_to_cross_section(z + depth + BACKING));
            (z, f)
        }
        Face::Top => {
            let z = top_face(solid);
            let f = solid
                .slice_to_cross_section(z - 0.05)
                .intersection(&solid.slice_to_cross_section(z - depth - BACKING))
                .difference(&solid.slice_to_cross_section(z + 0.05));
            (z, f)
        }
    }
}

/// The flat area at an upward face of height `z` (as `footprint` does for the top face).
fn footprint_at(solid: &Manifold, z: f64, depth: f64) -> (f64, CrossSection) {
    let f = solid
        .slice_to_cross_section(z - 0.05)
        .intersection(&solid.slice_to_cross_section(z - depth - BACKING))
        .difference(&solid.slice_to_cross_section(z + 0.05));
    (z, f)
}

/// The inlay's prism for a shape on a face, reaching a little past the face.
pub fn prism(shape: &CrossSection, face: Face, z: f64, depth: f64) -> Manifold {
    match face {
        Face::Bed => shape.extrude(depth + 1.0).translate(0.0, 0.0, z - 1.0),
        Face::Top => shape.extrude(depth + 1.0).translate(0.0, 0.0, z - depth),
    }
}

/// True if `shape`, grown by the margin, lies inside the footprint.
fn fits(shape: &CrossSection, area: &CrossSection) -> bool {
    offset(shape, MARGIN, 16).difference(area).area() < 1e-3
}

/// Spots where a w × h box would fit inside `area` with the margin, nearest the middle first.
fn spots(area: &CrossSection, w: f64, h: f64) -> Vec<[f64; 2]> {
    let k = h / w;
    let inner = offset(area, -MARGIN, 16)
        .scale(k, 1.0)
        .offset(-h / 2.0, JoinType::Miter, 2.0, 0)
        .scale(1.0 / k, 1.0);
    if inner.is_empty() {
        return Vec::new();
    }
    let r = area.bounds_rect2();
    let mid = [(r.min_x + r.max_x) / 2.0, (r.min_y + r.max_y) / 2.0];
    let dist = |p: &[f64; 2]| (p[0] - mid[0]).powi(2) + (p[1] - mid[1]).powi(2);
    let order = |v: &mut Vec<[f64; 2]>| {
        v.sort_by(|a, b| dist(a).total_cmp(&dist(b)).then(a[0].total_cmp(&b[0])));
        v.dedup();
    };
    // Each open patch's middle first, then points round their edges.
    let parts = inner.decompose();
    let mut centres: Vec<[f64; 2]> = parts
        .iter()
        .map(|part| {
            let b = part.bounds_rect2();
            [(b.min_x + b.max_x) / 2.0, (b.min_y + b.max_y) / 2.0]
        })
        .collect();
    order(&mut centres);
    let mut edges = Vec::new();
    for part in &parts {
        for p in offset(part, -0.3, 8).to_polygons() {
            edges.extend(p);
        }
        for p in part.to_polygons() {
            edges.extend(p);
        }
    }
    order(&mut edges);
    edges.truncate(400);
    centres.extend(edges);
    centres
}

/// Put the biggest logo that fits on the first face where one fits. `above` is whatever is
/// mounted on the part, in its print coordinates: a raised logo must clear it.
pub fn place(
    d: &Design,
    name: &str,
    solid: &Manifold,
    spot: Spot,
    above: Option<&Manifold>,
) -> Result<Brand, String> {
    let mut depth = d.letter_h;
    let mut must_raise = false;
    let (inlay, face, width, words) = match spot {
        Spot::Fixed(inlay, face) => {
            // A fixed inlay's depth is however far it reaches into the part.
            let (lo, hi) = crate::geom::bounds(&solid.intersection(&inlay));
            depth = hi[2] - lo[2];
            let shape = inlay.slice_to_cross_section(match face {
                Face::Bed => crate::geom::bounds(solid).0[2] + depth / 2.0,
                Face::Top => top_face(solid) - depth / 2.0,
            });
            let r = shape.bounds_rect2();
            let (_, area) = footprint(solid, face, depth);
            if !fits(&shape, &area) {
                return Err(format!(
                    "{name}: the logo comes within {MARGIN} mm of an edge, hole, slit or taller feature, \
                     or has no plastic behind it"
                ));
            }
            if !strokes_ok(&shape) {
                return Err(format!(
                    "{name}: the logo has strokes under {MIN_STROKE} mm"
                ));
            }
            if !gaps_ok(&shape) {
                return Err(format!(
                    "{name}: the logo leaves plastic strips under {MIN_GAP} mm"
                ));
            }
            (inlay, face, r.max_x - r.min_x, true)
        }
        Spot::Label(shape, z) => {
            must_raise = true;
            let (_, area) = footprint_at(solid, z, depth);
            if !fits(&shape, &area) {
                return Err(format!(
                    "{name}: the label comes within {MARGIN} mm of an edge, hole or taller \
                     feature, or has no plastic behind it"
                ));
            }
            if !strokes_ok(&shape) {
                return Err(format!(
                    "{name}: the label has strokes under {MIN_STROKE} mm"
                ));
            }
            if !gaps_ok(&shape) {
                return Err(format!(
                    "{name}: the label leaves plastic strips under {MIN_GAP} mm"
                ));
            }
            let r = shape.bounds_rect2();
            (
                prism(&shape, Face::Top, z, depth),
                Face::Top,
                r.max_x - r.min_x,
                true,
            )
        }
        Spot::Search(faces) => {
            let all = candidates(d)?;
            let mut found = None;
            'faces: for &face in faces {
                let (z, area) = footprint(solid, face, depth);
                for (shape, width, words) in &all {
                    for turn in [0.0, 90.0] {
                        let mut s = shape.rotate(turn);
                        if face == Face::Bed {
                            s = s.scale(-1.0, 1.0);
                        }
                        if !strokes_ok(&s) || !gaps_ok(&s) {
                            continue;
                        }
                        let b = s.bounds_rect2();
                        for p in spots(&area, b.max_x - b.min_x, b.max_y - b.min_y) {
                            let at = s.translate(p[0], p[1]);
                            if fits(&at, &area) {
                                found = Some((prism(&at, face, z, depth), face, *width, *words));
                                break 'faces;
                            }
                        }
                    }
                }
            }
            found.ok_or_else(|| format!("{name}: no flat face has room for the logo"))?
        }
    };
    let inside = solid.intersection(&inlay);
    let body = solid.difference(&inlay);
    let want = inlay_volume(&inlay, solid, face, depth);
    if (inside.volume() - want).abs() > 0.02 * want + EPS {
        return Err(format!("{name}: the logo isn't fully inside the part"));
    }
    // Raise it where it's on an upward face and nothing sits within RAISE_CLEAR of it.
    let mut raise = 0.0;
    let mut accent = inside.clone();
    if face == Face::Top {
        let z = crate::geom::bounds(&inside).1[2];
        let shape = inlay.slice_to_cross_section(z - depth / 2.0);
        let up = shape
            .extrude(d.logo_raise + EPS)
            .translate(0.0, 0.0, z - EPS);
        let room = offset(&shape, RAISE_CLEAR, 16)
            .extrude(d.logo_raise + RAISE_CLEAR)
            .translate(0.0, 0.0, z);
        let clear = above.is_none_or(|a| a.intersection(&room).volume() < 1e-6)
            && body.intersection(&room).volume() < 1e-6;
        if clear {
            raise = d.logo_raise;
            accent = inside.union(&up);
        } else if must_raise {
            return Err(format!(
                "{name}: the raised label would come within {RAISE_CLEAR} mm of something above it"
            ));
        }
    }
    Ok(Brand {
        body,
        accent,
        face,
        width,
        words,
        raise,
    })
}

/// The inlay's volume inside the part should be its area times its depth.
fn inlay_volume(inlay: &Manifold, solid: &Manifold, face: Face, depth: f64) -> f64 {
    let z = match face {
        Face::Bed => crate::geom::bounds(solid).0[2] + depth / 2.0,
        // The face is where the part ends inside the inlay's footprint.
        Face::Top => crate::geom::bounds(&solid.intersection(inlay)).1[2] - depth / 2.0,
    };
    inlay.slice_to_cross_section(z).area() * depth
}

/// Solid room to keep in a lightened plate for its logo: the biggest logo that prints cleanly
/// and fits in `free` (the plate's open top face), grown by the margin and 1 mm more.
pub fn pad(d: &Design, free: &CrossSection) -> Result<CrossSection, String> {
    for (shape, _, _) in candidates(d)? {
        if !strokes_ok(&shape) {
            continue;
        }
        let grown = offset(&shape, MARGIN + 1.0, 16);
        let b = grown.bounds_rect2();
        for p in spots(free, b.max_x - b.min_x, b.max_y - b.min_y) {
            let at = grown.translate(p[0], p[1]);
            if at.difference(free).area() < 1e-3 {
                return Ok(at);
            }
        }
    }
    Ok(CrossSection::empty())
}

/// Logos that must be refused, each by a different check.
/// A broken design's label and what placing its logo gave.
pub type Attempt = (&'static str, Result<Brand, String>);

pub fn broken_designs(d: &Design) -> Result<Vec<Attempt>, String> {
    let plate = crate::parts::payload_plate(d, None, 0.0)?;
    let plate_logo = crate::parts::plate_brand(d)?;
    let fixed = |shape: CrossSection| {
        Spot::Fixed(prism(&shape, Face::Top, d.plate_t, d.letter_h), Face::Top)
    };
    let tiny = crate::geom::bx(0.0, 0.0, 0.0, 6.0, 6.0, 3.0);
    Ok(vec![
        (
            "logo hanging over the plate's edge",
            place(
                d,
                "broken",
                &plate,
                Spot::Fixed(plate_logo.translate(0.0, -40.0, 0.0), Face::Top),
                None,
            ),
        ),
        (
            "logo over a snap pin's slot",
            place(
                d,
                "broken",
                &plate,
                {
                    let [x, y] = d.plate_holes()[0];
                    fixed(mark(12.5, [x, y]))
                },
                None,
            ),
        ),
        (
            "logo strokes too thin to print",
            place(
                d,
                "broken",
                &plate,
                fixed(CrossSection::square(0.5, 8.0, false).translate(0.0, -11.0)),
                None,
            ),
        ),
        (
            "a part with no room for a logo",
            place(
                d,
                "broken",
                &tiny,
                Spot::Search(&[Face::Bed, Face::Top]),
                None,
            ),
        ),
        (
            "logo deeper than the plate has plastic behind it",
            place(
                d,
                "broken",
                &plate,
                Spot::Fixed(
                    prism(
                        &mark(17.0, [0.0, -7.0]),
                        Face::Top,
                        d.plate_t,
                        d.plate_t - 0.1,
                    ),
                    Face::Top,
                ),
                None,
            ),
        ),
        (
            "two logo areas 0.5 mm apart, too close for the colours to stay apart",
            place(d, "broken", &plate, fixed(two_bars(0.5, [0.0, -7.0])), None),
        ),
        (
            "a raised label right under a board",
            place(
                d,
                "broken",
                &plate,
                Spot::Label(two_bars(2.0, [0.0, -7.0]), d.plate_t),
                Some(&crate::geom::bx(
                    -10.0,
                    -15.0,
                    d.plate_t + 0.4,
                    10.0,
                    1.0,
                    d.plate_t + 2.0,
                )),
            ),
        ),
    ])
}

/// Two 3 mm bars `gap` apart, for the broken designs.
fn two_bars(gap: f64, at: [f64; 2]) -> CrossSection {
    let bar = |x0: f64| CrossSection::square(3.0, 8.0, false).translate(x0, -4.0);
    bar(-3.0 - gap / 2.0)
        .union(&bar(gap / 2.0))
        .translate(at[0], at[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::{bx, rect};

    #[test]
    fn narrow_local_gaps_fail_in_large_labels() {
        let large = rect(20.0, 20.0, 80.0, 80.0);
        let two_letters = |gap| rect(0.0, 0.0, 2.0, 1.0).union(&rect(2.0 + gap, 0.0, 4.0, 1.0));
        assert!(!gaps_ok(&large.union(&two_letters(0.33))));
        assert!(!gaps_ok(&large.union(&two_letters(0.79))));
        assert!(gaps_ok(&large.union(&two_letters(MIN_GAP))));

        let slot = |gap| large.difference(&rect(40.0, 75.0, 40.0 + gap, 81.0));
        assert!(!gaps_ok(&slot(0.02)));
        assert!(gaps_ok(&slot(0.81)));
    }

    #[test]
    fn raised_label_clearance_runs_from_the_face_to_the_relief_plus_clearance() {
        let d = Design::default();
        let z = 2.0;
        let part = bx(0.0, 0.0, 0.0, 20.0, 20.0, z);
        let limit = d.logo_raise + RAISE_CLEAR;
        for (bottom, top, clears) in [
            (-0.04, 0.0, true),
            (0.01, 0.04, false),
            (limit - 0.02, limit - 0.01, false),
            (limit, limit + 0.03, true),
        ] {
            let obstruction = bx(6.5, 9.0, z + bottom, 8.5, 11.0, z + top);
            let result = place(
                &d,
                "clearance test",
                &part,
                Spot::Label(two_bars(2.0, [10.0, 10.0]), z),
                Some(&obstruction),
            );
            if clears {
                assert_eq!(result.unwrap().raise, d.logo_raise);
            } else {
                assert!(matches!(result, Err(e) if e.contains("raised label would come within")));
            }
        }
    }
}
