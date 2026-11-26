//! The archived Rev A bench case and Rev B rail-fit set, their P1S plates and
//! the Rev B drawing. A port of `hardware/generate.py`, `hardware/p1s/prepare.py`
//! and `hardware/drawing.py` with the same geometry. Dimensions in mm.
//!
//! `cubesat-cad archive hardware` rewrites `hardware/stl/*.stl`,
//! `hardware/p1s/*.3mf` and `hardware/rev-b-drawing.svg`, after checking that
//! every part is one closed solid of the expected size and that the lid and
//! plates seat without colliding with their frames.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::Path;

use manifold_csg::{CrossSection, Manifold};

use crate::mesh::{Mesh, write_3mf};

const WIDTH: f64 = 100.0;
const LENGTH: f64 = 113.5;
const CORNER_CLIP: f64 = 3.0;
const WALL: f64 = 3.0;
const FLOOR: f64 = 4.0;
const LID: f64 = 4.0;
const LIP_HEIGHT: f64 = 2.0;
const LIP_CLEARANCE_PER_SIDE: f64 = 0.35;
const BOSS_INSET: f64 = 6.0;
const BOSS_SIDE: f64 = 8.0;
const M3_CLEARANCE: f64 = 3.5;
const M3_PILOT: f64 = 2.8;
const PILOT_DEPTH: f64 = 10.0;
const CAMERA_APERTURE: f64 = 18.0;
const SERVICE_SLOT_WIDTH: f64 = 30.0;
const SERVICE_SLOT_DEPTH: f64 = 18.0;
const COUPON_DIAMETERS: [f64; 6] = [2.6, 2.8, 3.0, 3.2, 3.5, 3.8];
const COUPON_PITCH: f64 = 8.0;
const FACETS: i32 = 64;
const EPS: f64 = 0.1;

// Rev B bench geometry. The outside reference follows nominal 1U dimensions.
const RAIL_WIDTH: f64 = 9.0;
const SIDE_RECESS: f64 = 3.0;
const END_RECESS: f64 = 3.0;
const FACE_THICKNESS: f64 = 2.4;
const FACE_CLEARANCE: f64 = 0.35;
const LOCATOR_LENGTH: f64 = 18.0;
const LOCATOR_DEPTH: f64 = 2.5;
const RAIL_COUPON_WIDTH: f64 = 26.0;
const RAIL_COUPON_LENGTH: f64 = 20.0;

fn cube(x: f64, y: f64, z: f64) -> Manifold {
    Manifold::cube(x, y, z, false)
}

fn extrude(points: &[[f64; 2]], height: f64) -> Manifold {
    Manifold::extrude(&CrossSection::from_simple_polygon(points), height)
}

fn profile(inset: f64, height: f64) -> Manifold {
    let (a, b, c) = (inset, WIDTH - inset, CORNER_CLIP);
    extrude(
        &[
            [a + c, a],
            [b - c, a],
            [b, a + c],
            [b, b - c],
            [b - c, b],
            [a + c, b],
            [a, b - c],
            [a, a + c],
        ],
        height,
    )
}

fn bore(diameter: f64, depth: f64, x: f64, y: f64, z: f64) -> Manifold {
    Manifold::cylinder(depth, diameter / 2.0, diameter / 2.0, FACETS, false).translate(x, y, z)
}

fn positions() -> [(f64, f64); 4] {
    let d = BOSS_INSET;
    [
        (d, d),
        (d, WIDTH - d),
        (WIDTH - d, d),
        (WIDTH - d, WIDTH - d),
    ]
}

/// Rev A shell. Corner columns engage the wall and take a short thread at the open rim.
pub fn shell() -> Manifold {
    let height = LENGTH - LID;
    let mut body =
        profile(0.0, height).difference(&profile(WALL, height).translate(0.0, 0.0, FLOOR));
    for (x, y) in positions() {
        let boss = cube(BOSS_SIDE, BOSS_SIDE, height - FLOOR + EPS);
        body = body.union(&boss.translate(x - BOSS_SIDE / 2.0, y - BOSS_SIDE / 2.0, FLOOR - EPS));
        body = body.difference(&bore(
            M3_PILOT,
            PILOT_DEPTH + EPS,
            x,
            y,
            height - PILOT_DEPTH,
        ));
    }
    // The cable slot opens at the top. The other faces stay free for fit studies.
    let slot = cube(
        SERVICE_SLOT_WIDTH,
        WALL + 2.0 * EPS,
        SERVICE_SLOT_DEPTH + EPS,
    );
    body.difference(&slot.translate(
        (WIDTH - SERVICE_SLOT_WIDTH) / 2.0,
        -EPS,
        height - SERVICE_SLOT_DEPTH,
    ))
}

/// Rev A lid. Print exterior face down. Mid-side tabs avoid the corner bosses.
pub fn lid() -> Manifold {
    let mut top = profile(0.0, LID);
    let (tab_w, tab_d) = (18.0, 3.0);
    let edge = WALL + LIP_CLEARANCE_PER_SIDE;
    let mid = (WIDTH - tab_w) / 2.0;
    for (x, y, dx, dy) in [
        (mid, edge, tab_w, tab_d),
        (mid, WIDTH - edge - tab_d, tab_w, tab_d),
        (edge, mid, tab_d, tab_w),
        (WIDTH - edge - tab_d, mid, tab_d, tab_w),
    ] {
        top = top.union(&cube(dx, dy, LIP_HEIGHT + EPS).translate(x, y, LID - EPS));
    }
    for (x, y) in positions() {
        top = top.difference(&bore(M3_CLEARANCE, LID + 2.0 * EPS, x, y, -EPS));
    }
    top.difference(&bore(
        CAMERA_APERTURE,
        LID + 2.0 * EPS,
        WIDTH / 2.0,
        WIDTH / 2.0,
        -EPS,
    ))
}

/// Rev A M3 fit coupon: one hole per candidate size.
pub fn coupon() -> Manifold {
    let width = COUPON_PITCH * COUPON_DIAMETERS.len() as f64;
    let (depth, height) = (14.0, PILOT_DEPTH);
    let mut part = cube(width, depth, height);
    for (i, diameter) in COUPON_DIAMETERS.iter().enumerate() {
        let x = COUPON_PITCH * (i as f64 + 0.5);
        part = part.difference(&bore(*diameter, height + 2.0 * EPS, x, depth / 2.0, -EPS));
    }
    // A vertical key at the 2.6 mm end makes the hole order unambiguous.
    part.difference(&bore(4.0, height + 2.0 * EPS, 0.0, 0.0, -EPS))
}

/// Rev B frame: four continuous corner rails and four recessed side walls.
pub fn rail_frame() -> Manifold {
    let wall_start = END_RECESS + FACE_THICKNESS;
    let wall_height = LENGTH - 2.0 * wall_start;
    let outer_width = WIDTH - 2.0 * SIDE_RECESS;
    let inner_width = outer_width - 2.0 * WALL;
    let outer = cube(outer_width, outer_width, wall_height);
    let cavity = cube(inner_width, inner_width, wall_height + 2.0 * EPS);
    let mut walls = outer
        .difference(&cavity.translate(WALL, WALL, -EPS))
        .translate(SIDE_RECESS, SIDE_RECESS, wall_start);
    for x in [0.0, WIDTH - RAIL_WIDTH] {
        for y in [0.0, WIDTH - RAIL_WIDTH] {
            walls = walls.union(&cube(RAIL_WIDTH, RAIL_WIDTH, LENGTH).translate(x, y, 0.0));
        }
    }
    walls
}

/// Rev B end plate. It rests on the end rim and locates inside the walls.
pub fn rail_face(camera: bool) -> Manifold {
    let inset = SIDE_RECESS + FACE_CLEARANCE;
    let width = WIDTH - 2.0 * inset;
    let clip = 2.0 * (RAIL_WIDTH - SIDE_RECESS);
    let mut face = extrude(
        &[
            [clip, 0.0],
            [width - clip, 0.0],
            [width, clip],
            [width, width - clip],
            [width - clip, width],
            [clip, width],
            [0.0, width - clip],
            [0.0, clip],
        ],
        FACE_THICKNESS,
    );
    let edge = WALL;
    let center = (width - LOCATOR_LENGTH) / 2.0;
    for (x, y, dx, dy) in [
        (center, edge, LOCATOR_LENGTH, LOCATOR_DEPTH),
        (
            center,
            width - edge - LOCATOR_DEPTH,
            LOCATOR_LENGTH,
            LOCATOR_DEPTH,
        ),
        (edge, center, LOCATOR_DEPTH, LOCATOR_LENGTH),
        (
            width - edge - LOCATOR_DEPTH,
            center,
            LOCATOR_DEPTH,
            LOCATOR_LENGTH,
        ),
    ] {
        face = face.union(&cube(dx, dy, LIP_HEIGHT + EPS).translate(x, y, FACE_THICKNESS - EPS));
    }
    if camera {
        face = face.difference(&bore(
            CAMERA_APERTURE,
            FACE_THICKNESS + 2.0 * EPS,
            width / 2.0,
            width / 2.0,
            -EPS,
        ));
    }
    face
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-5 + 1e-8 * b.abs()
}

fn bounds(m: &Manifold) -> ([f64; 3], [f64; 3]) {
    let b = m.bounding_box().expect("non-empty solid");
    (b.min(), b.max())
}

fn verify_rail_set(frame: &Manifold, base: &Manifold, camera: &Manifold) -> Result<(), String> {
    let inset = SIDE_RECESS + FACE_CLEARANCE;
    let upper = camera
        .rotate(180.0, 0.0, 0.0)
        .translate(inset, WIDTH - inset, LENGTH - END_RECESS);
    let lower = base.translate(inset, inset, END_RECESS);
    for face in [&upper, &lower] {
        if !close(frame.intersection(face).volume(), 0.0) {
            return Err("rail-fit plate collides with the frame".into());
        }
    }
    let (lo, hi) = bounds(&frame.union(&upper).union(&lower));
    let want = [0.0, 0.0, 0.0, WIDTH, WIDTH, LENGTH];
    if !lo.iter().chain(&hi).zip(want).all(|(a, b)| close(*a, b)) {
        return Err("rail-fit assembly exceeds the nominal 1U envelope".into());
    }
    Ok(())
}

/// A checked part ready to write.
pub struct Part {
    /// File stem under `hardware/stl`.
    pub name: &'static str,
    /// The mesh, as modelled (already on the bed).
    pub mesh: Mesh,
}

fn checked(name: &'static str, solid: &Manifold, expected: [f64; 3]) -> Result<Part, String> {
    let mesh = Mesh::from_solid(name, solid)?;
    let (lo, hi) = mesh.bounds();
    let extents = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let ok = mesh.is_closed()
        && mesh.volume_and_centroid().0 > 0.0
        && solid.decompose().len() == 1
        && extents.iter().zip(expected).all(|(a, b)| close(*a, b))
        && close(lo[2], 0.0);
    if !ok {
        return Err(format!("{name} failed mesh or size checks: {lo:?} {hi:?}"));
    }
    Ok(Part { name, mesh })
}

/// The seven archived parts, checked.
///
/// # Errors
/// If a part is not one closed solid of the expected size, or a lid or plate
/// collides with its frame.
pub fn parts() -> Result<Vec<Part>, String> {
    let body = shell();
    let cover = lid();
    let seated = cover.rotate(180.0, 0.0, 0.0).translate(0.0, WIDTH, LENGTH);
    if !close(body.intersection(&seated).volume(), 0.0) {
        return Err("seated lid interferes with the shell".into());
    }
    let frame = rail_frame();
    let base = rail_face(false);
    let camera = rail_face(true);
    verify_rail_set(&frame, &base, &camera)?;
    let corner = frame.intersection(&cube(
        RAIL_COUPON_WIDTH,
        RAIL_COUPON_WIDTH,
        RAIL_COUPON_LENGTH,
    ));
    let face_width = WIDTH - 2.0 * (SIDE_RECESS + FACE_CLEARANCE);
    Ok(vec![
        checked("bench-shell", &body, [WIDTH, WIDTH, LENGTH - LID])?,
        checked("camera-lid", &cover, [WIDTH, WIDTH, LID + LIP_HEIGHT])?,
        checked(
            "m3-fit-coupon",
            &coupon(),
            [
                COUPON_PITCH * COUPON_DIAMETERS.len() as f64,
                14.0,
                PILOT_DEPTH,
            ],
        )?,
        checked(
            "rev-b-rail-corner-coupon",
            &corner,
            [RAIL_COUPON_WIDTH, RAIL_COUPON_WIDTH, RAIL_COUPON_LENGTH],
        )?,
        checked("rev-b-rail-frame", &frame, [WIDTH, WIDTH, LENGTH])?,
        checked(
            "rev-b-base-plate",
            &base,
            [face_width, face_width, FACE_THICKNESS + LIP_HEIGHT],
        )?,
        checked(
            "rev-b-camera-plate",
            &camera,
            [face_width, face_width, FACE_THICKNESS + LIP_HEIGHT],
        )?,
    ])
}

const PRINTER_VOLUME: [f64; 3] = [256.0, 256.0, 256.0];
const EDGE_MARGIN: f64 = 10.0;

/// Rev A's three P1S test plates: (plate file stem, part).
pub const PLATES: [(&str, &str); 3] = [
    ("01-m3-fit-coupon", "m3-fit-coupon"),
    ("02-case-shell", "bench-shell"),
    ("03-camera-lid", "camera-lid"),
];

/// Centre a part on the 256 mm bed in its print orientation.
///
/// # Errors
/// If the part does not fit inside the plate margin or the print height.
pub fn on_plate(name: &str, mesh: &Mesh) -> Result<Mesh, String> {
    let (lo, hi) = mesh.bounds();
    let shift = [
        PRINTER_VOLUME[0] / 2.0 - (lo[0] + hi[0]) / 2.0,
        PRINTER_VOLUME[1] / 2.0 - (lo[1] + hi[1]) / 2.0,
        -lo[2],
    ];
    let placed = mesh.clone().translated(shift);
    let (lo, hi) = placed.bounds();
    if lo[0] < EDGE_MARGIN
        || lo[1] < EDGE_MARGIN
        || hi[0] > PRINTER_VOLUME[0] - EDGE_MARGIN
        || hi[1] > PRINTER_VOLUME[1] - EDGE_MARGIN
    {
        return Err(format!("{name}: footprint exceeds the P1S plate margin"));
    }
    if hi[2] >= PRINTER_VOLUME[2] {
        return Err(format!("{name}: part is taller than the P1S print volume"));
    }
    Ok(placed)
}

const SCALE: f64 = 2.5;

/// Python's `format(x, "g")`: six significant digits, no trailing zeros.
fn g(x: f64) -> String {
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0".into()
        } else {
            "0".into()
        };
    }
    let exp = x.abs().log10().floor() as i32;
    if (-4..6).contains(&exp) {
        let decimals = usize::try_from(5 - exp).unwrap_or(0);
        let text = format!("{x:.decimals$}");
        if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            text
        }
    } else {
        format!("{x:e}")
    }
}

fn round3(x: f64) -> f64 {
    let r = (x * 1000.0).round_ties_even() / 1000.0;
    if r == 0.0 { 0.0 } else { r }
}

/// Edges between faces that meet at more than 1 degree.
fn creases(mesh: &Mesh) -> Vec<[usize; 2]> {
    let normal = |t: &[usize; 3]| {
        let (a, b, c) = (
            mesh.vertices[t[0]],
            mesh.vertices[t[1]],
            mesh.vertices[t[2]],
        );
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-30);
        n.map(|x| x / len)
    };
    let normals: Vec<[f64; 3]> = mesh.triangles.iter().map(normal).collect();
    let mut faces: HashMap<[usize; 2], Vec<usize>> = HashMap::new();
    for (i, t) in mesh.triangles.iter().enumerate() {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            faces.entry([a.min(b), a.max(b)]).or_default().push(i);
        }
    }
    let limit = 1f64.to_radians().cos();
    let mut out: Vec<[usize; 2]> = faces
        .into_iter()
        .filter(|(_, f)| f.len() == 2)
        .filter(|(_, f)| {
            let (n, m) = (normals[f[0]], normals[f[1]]);
            n[0] * m[0] + n[1] * m[1] + n[2] * m[2] < limit
        })
        .map(|(e, _)| e)
        .collect();
    out.sort_unstable();
    out
}

/// Project crease edges onto two axes as an SVG path. With `face_only`, only
/// edges lying in the part's lowest face (the camera plate's exterior) count.
fn outline(mesh: &Mesh, axes: [usize; 2], origin: [f64; 2], face_only: bool) -> String {
    let (lo, _) = mesh.bounds();
    let mut segments: BTreeSet<[[u64; 2]; 2]> = BTreeSet::new();
    let key = |p: [f64; 2]| [p[0].to_bits(), p[1].to_bits()];
    let mut points: Vec<([f64; 2], [f64; 2])> = Vec::new();
    for [a, b] in creases(mesh) {
        let (pa, pb) = (mesh.vertices[a], mesh.vertices[b]);
        if face_only && !(close(pa[2], lo[2]) && close(pb[2], lo[2])) {
            continue;
        }
        let project = |p: [f64; 3]| {
            [
                round3((p[axes[0]] - lo[axes[0]]) * SCALE + origin[0]),
                round3(-(p[axes[1]] - lo[axes[1]]) * SCALE + origin[1]),
            ]
        };
        let (mut first, mut second) = (project(pa), project(pb));
        if (second[0], second[1]) < (first[0], first[1]) {
            std::mem::swap(&mut first, &mut second);
        }
        if first == second {
            continue;
        }
        if segments.insert([key(first), key(second)]) {
            points.push((first, second));
        }
    }
    points.sort_by(|x, y| {
        (x.0[0], x.0[1], x.1[0], x.1[1])
            .partial_cmp(&(y.0[0], y.0[1], y.1[0], y.1[1]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    points
        .iter()
        .map(|(p, q)| format!("M{},{}L{},{}", g(p[0]), g(p[1]), g(q[0]), g(q[1])))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The Rev B drawing: front elevation of the rail frame and the exterior face
/// of the camera plate, projected from their meshes, with nominal dimensions.
#[must_use]
pub fn drawing(frame: &Mesh, camera: &Mesh) -> String {
    let (flo, fhi) = frame.bounds();
    let (clo, chi) = camera.bounds();
    let (width, depth, height) = (fhi[0] - flo[0], fhi[1] - flo[1], fhi[2] - flo[2]);
    let (plate_width, plate_height) = (chi[0] - clo[0], chi[2] - clo[2]);
    let face: Vec<[f64; 3]> = camera
        .vertices
        .iter()
        .copied()
        .filter(|v| close(v[2], clo[2]))
        .collect();
    let bore: Vec<f64> = face
        .iter()
        .filter(|v| (v[0] - plate_width / 2.0).hypot(v[1] - plate_width / 2.0) < plate_width / 4.0)
        .map(|v| v[0])
        .collect();
    let diameter = bore.iter().copied().fold(f64::MIN, f64::max)
        - bore.iter().copied().fold(f64::MAX, f64::min);
    let rail_width = frame
        .vertices
        .iter()
        .map(|v| (v[0] * 1e4).round() / 1e4)
        .filter(|x| *x > 0.0 && *x < width / 2.0)
        .fold(f64::MIN, f64::max);
    let ground = 445.0;
    let (frame_x, plate_x) = (125.0, 530.0);
    let (frame_right, frame_top) = (frame_x + width * SCALE, ground - height * SCALE);
    let (plate_right, plate_top) = (plate_x + plate_width * SCALE, ground - plate_width * SCALE);
    let frame_path = outline(frame, [0, 2], [frame_x, ground], false);
    let plate_path = outline(camera, [0, 1], [plate_x, ground], true);
    let mut svg = String::new();
    let _ = write!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 590" role="img" aria-labelledby="title desc">
<title id="title">Rev B CubeSat rail-frame and camera-plate drawing</title>
<desc id="desc">Front elevation of the rail frame and exterior plan of the camera plate, projected from their STL meshes with nominal millimetre dimensions.</desc>
<rect width="900" height="590" fill="#fff"/>
<g font-family="system-ui, sans-serif" fill="#182b41">
<text x="38" y="53" font-size="31" font-weight="700">Rev B · Rail-fit bench model</text>
<text x="38" y="83" font-size="18">Orthographic views from the printable meshes</text>
<path d="M38 103H862" fill="none" stroke="#a4b5c7"/>
<text x="{fx}" y="133" font-size="17" font-weight="700">Rail frame · front</text>
<text x="{px}" y="183" font-size="17" font-weight="700">Camera plate · exterior face</text>
<g fill="none" stroke="#182b41" stroke-width="1.7" stroke-linejoin="round">
<path d="{frame_path}"/>
<path d="{plate_path}"/>
</g>
<g fill="none" stroke="#386497" stroke-width="1.15">
<path d="M{fx} {g5}v36M{fr} {g5}v36M{fx} {g30}H{fr}M{fx6} {ft}H63M{fx6} {gr}H63M72 {ft}V{gr}"/>
<path d="M{px} {g5}v36M{pr} {g5}v36M{px} {g30}H{pr}"/>
<path d="M120 480l10 -10m240 10 10 -10M525 480l10 -10m223.25 10 10 -10M67 {ft5}l10 -10m-10 {hs} 10 -10"/>
<path d="M{lead_x} {lead_y}h28l23 -35h46"/>
</g>
<g font-size="15" font-weight="700" fill="#294e79">
<text x="{fmid}" y="501" text-anchor="middle">{w} mm</text>
<text transform="translate(56 {fvmid}) rotate(-90)" text-anchor="middle">{h} mm</text>
<text x="{pmid}" y="501" text-anchor="middle">{pw:.1} mm</text>
<text x="722" y="{pt72}">Ø{dia:.0} mm</text>
</g>
<path d="M38 523H862" fill="none" stroke="#a4b5c7"/>
<text x="38" y="551" font-size="15">Frame depth {d} mm · Rails {rw} mm square · Camera plate with locators {ph:.1} mm thick</text>
<text x="38" y="574" font-size="14" fill="#435974">Nominal geometry for bench fit checks · Do not measure from this drawing</text>
</g>
</svg>
"##,
        fx = g(frame_x),
        px = g(plate_x),
        g5 = g(ground + 5.0),
        g30 = g(ground + 30.0),
        fr = g(frame_right),
        fx6 = g(frame_x - 6.0),
        ft = g(frame_top),
        gr = g(ground),
        pr = g(plate_right),
        ft5 = g(frame_top + 5.0),
        hs = g(height * SCALE),
        lead_x = g(plate_x + plate_width * SCALE / 2.0 + diameter * SCALE / 2.0),
        lead_y = g(ground - plate_width * SCALE / 2.0),
        fmid = g(f64::midpoint(frame_x, frame_right)),
        w = g(width),
        fvmid = g(f64::midpoint(frame_top, ground)),
        h = g(height),
        pmid = g(f64::midpoint(plate_x, plate_right)),
        pw = plate_width,
        pt72 = g(plate_top + 72.0),
        dia = diameter,
        d = g(depth),
        rw = g(rail_width),
        ph = plate_height,
    );
    svg
}

/// Check every archived part, then write the STL files, the P1S plates and
/// the drawing under `hardware`.
///
/// # Errors
/// If a check fails or a file cannot be written.
pub fn write_all(hardware: &Path) -> Result<(), String> {
    let parts = parts()?;
    let stl_dir = hardware.join("stl");
    std::fs::create_dir_all(&stl_dir).map_err(|e| e.to_string())?;
    for part in &parts {
        let path = stl_dir.join(format!("{}.stl", part.name));
        let mut bytes = Vec::new();
        part.mesh
            .write_stl_titled(
                &mut bytes,
                &format!("CubeSat Classroom archive {}", part.name),
            )
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let (lo, hi) = part.mesh.bounds();
        println!(
            "{}: [{}, {}, {}] mm, closed, {} triangles",
            part.name,
            g(hi[0] - lo[0]),
            g(hi[1] - lo[1]),
            g(hi[2] - lo[2]),
            part.mesh.triangles.len()
        );
    }
    let by_name: HashMap<&str, &Mesh> = parts.iter().map(|p| (p.name, &p.mesh)).collect();
    // A fresh output folder (a fork, or `archive /tmp/out`) has no p1s/ yet.
    let p1s_dir = hardware.join("p1s");
    std::fs::create_dir_all(&p1s_dir).map_err(|e| format!("{}: {e}", p1s_dir.display()))?;
    for (plate, part) in PLATES {
        let placed = on_plate(part, by_name[part])?;
        let path = hardware.join("p1s").join(format!("{plate}.3mf"));
        write_3mf(&path, &[(part.to_string(), placed)])
            .map_err(|e| format!("{}: {e}", path.display()))?;
        println!("{plate}.3mf: {part} centred on the P1S plate");
    }
    let svg = drawing(by_name["rev-b-rail-frame"], by_name["rev-b-camera-plate"]);
    let path = hardware.join("rev-b-drawing.svg");
    std::fs::write(&path, svg).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("rev-b-drawing.svg: frame front and camera plate face");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_g_format() {
        assert_eq!(g(125.0), "125");
        assert_eq!(g(412.875), "412.875");
        assert_eq!(g(113.5), "113.5");
        assert_eq!(g(0.0), "0");
        assert_eq!(g(1234.5678), "1234.57");
    }

    #[test]
    fn archived_parts_pass_their_checks() {
        let parts = parts().expect("archived parts");
        assert_eq!(parts.len(), 7);
        let frame = &parts
            .iter()
            .find(|p| p.name == "rev-b-rail-frame")
            .unwrap()
            .mesh;
        let camera = &parts
            .iter()
            .find(|p| p.name == "rev-b-camera-plate")
            .unwrap()
            .mesh;
        let svg = drawing(frame, camera);
        assert!(
            svg.contains(">100 mm<") && svg.contains(">113.5 mm<"),
            "{svg}"
        );
        assert!(svg.contains("Ø18 mm"));
        assert!(svg.contains("Rails 9 mm square"));
        for (_, part) in PLATES {
            let mesh = &parts.iter().find(|p| p.name == part).unwrap().mesh;
            on_plate(part, mesh).expect("fits the P1S plate");
        }
    }
}
