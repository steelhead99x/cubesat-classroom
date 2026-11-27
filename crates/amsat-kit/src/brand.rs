//! Two-colour CubeSat Classroom branding on every printed part.
//!
//! The mark, with the words CUBESAT CLASSROOM where they fit, is inlaid flush into one flat face
//! of the part as a second volume, printed in the second filament. The part's shape doesn't
//! change: the inlay and the rest of the part together are exactly the part, so a one-colour
//! print of the STL is the same part. The inlay is `letter_h` deep (three 0.2 mm layers), so the
//! printer only swaps filament on those few layers.
//!
//! Each part's face and spot are found by code, then checked:
//! - every point of the inlay is at least `MARGIN` from any edge, hole, slit or taller feature,
//!   so it never reaches a flex finger (fingers are only 2–2.4 mm wide between slits);
//! - there is solid plastic behind the whole inlay, at least 0.3 mm of it;
//! - no stroke is thinner than `MIN_STROKE`, so a 0.4 mm nozzle can print it.

use manifold_csg::{CrossSection, JoinType, Manifold};

use crate::design::Design;
use crate::geom::{EPS, Mesh, offset};
use crate::parts::{branding, mark};

/// Clear distance from the inlay to any edge, hole, slit or taller feature.
pub const MARGIN: f64 = 1.0;
/// The thinnest stroke a 0.4 mm nozzle prints cleanly.
pub const MIN_STROKE: f64 = 0.45;
/// Plastic left behind the inlay.
pub const BACKING: f64 = 0.3;

/// Logo widths to try, largest first: the mark with the words, then the mark alone.
const FULL_WIDTHS: [f64; 9] = [80.0, 70.0, 60.0, 52.0, 45.0, 40.0, 35.0, 31.0, 28.0];
const MARK_SIZES: [f64; 6] = [26.0, 22.0, 19.0, 16.0, 14.0, 12.5];
/// For the smallest parts: the cube alone, its three faces split by 0.5 mm lines.
const CUBE_SIZES: [f64; 5] = [10.0, 9.0, 8.0, 7.0, 6.0];
const CUBE_GAP: f64 = 0.5;

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

/// Put the biggest logo that fits on the first face where one fits.
pub fn place(d: &Design, name: &str, solid: &Manifold, spot: Spot) -> Result<Brand, String> {
    let mut depth = d.letter_h;
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
            (inlay, face, r.max_x - r.min_x, true)
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
                        if !strokes_ok(&s) {
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
    let accent = solid.intersection(&inlay);
    let body = solid.difference(&inlay);
    let want = inlay_volume(&inlay, solid, face, depth);
    if (accent.volume() - want).abs() > 0.02 * want + EPS {
        return Err(format!("{name}: the logo isn't fully inside the part"));
    }
    Ok(Brand {
        body,
        accent,
        face,
        width,
        words,
    })
}

/// The inlay's volume inside the part should be its area times its depth.
fn inlay_volume(inlay: &Manifold, solid: &Manifold, face: Face, depth: f64) -> f64 {
    let z = match face {
        Face::Bed => crate::geom::bounds(solid).0[2] + depth / 2.0,
        Face::Top => top_face(solid) - depth / 2.0,
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
            ),
        ),
        (
            "logo over a snap pin's slot",
            place(d, "broken", &plate, {
                let [x, y] = d.plate_holes()[0];
                fixed(mark(12.5, [x, y]))
            }),
        ),
        (
            "logo shrunk until its strokes are too thin to print",
            place(d, "broken", &plate, fixed(mark(6.0, [0.0, -7.0]))),
        ),
        (
            "a part with no room for a logo",
            place(d, "broken", &tiny, Spot::Search(&[Face::Bed, Face::Top])),
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
            ),
        ),
    ])
}
