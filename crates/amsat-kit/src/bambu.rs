//! Bambu Studio project files for a Bambu Lab printer with a 256 mm bed (P1S, P1P, X1C, X1E, A1).
//!
//! Every part is placed in its print orientation as its own object (Bambu Studio keeps one
//! instance per object per plate, so copies of one object can't be spread across plates). The
//! coupons are set to 100% infill, and each plate has a preview picture. The file follows Bambu
//! Studio's own project layout (src/libslic3r/Format/bbs_3mf.cpp): per-object meshes in
//! 3D/Objects/, plates and object settings in Metadata/model_settings.config. It carries no
//! printer or filament profile, so Bambu Studio keeps the ones you have selected. The output is
//! the same byte for byte on every run.
//!
//! Two filaments: white in AMS slot 1 and black in slot 2. White parts (frame, sides, cases,
//! coupons) carry a black CubeSat Classroom logo; black plates carry a white one. The logo is a
//! second part (volume) of the same object, inlaid flush, so the object's shape doesn't change.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;

use flate2::Compression;
use flate2::write::{DeflateEncoder, ZlibEncoder};

use crate::geom::Mesh;
use crate::parts::upstream;

const BED: f64 = 256.0;
/// Bambu Studio lays plates out with a 1/5 plate gap.
const PLATE_STRIDE: f64 = BED * 1.2;
const GAP: f64 = 6.0;
const ROW_GAP: f64 = 8.0;
const MARGIN: f64 = 8.0;
const APP: &str = "BambuStudio-02.00.00.00";
const DATE: &str = "2026-11-25";

#[derive(Clone, Copy)]
pub struct Part {
    pub name: &'static str,
    /// A generated mesh's name, or an AMSAT upstream file.
    pub source: &'static str,
    pub flip: bool,
    pub solid_infill: bool,
    pub colour: [f64; 3],
}

const fn p(name: &'static str, source: &'static str, solid_infill: bool, colour: [f64; 3]) -> Part {
    Part {
        name,
        source,
        flip: false,
        solid_infill,
        colour,
    }
}

// Monochrome, like a launch vehicle: white structure and coupons, black decks inside, each
// with the logo in the other colour. White is held a little under full so its lit faces still
// show against the white background.
const WHITE: [f64; 3] = [0.88, 0.88, 0.87];
const BLACK: [f64; 3] = [0.16, 0.16, 0.17];
// The bed in the previews, a warm textured-PEI tone so black and white parts both stand out.
const PEI: [f64; 3] = [0.55, 0.50, 0.40];

/// File, title, and plates: each plate a name and rows of parts, top to bottom.
pub type Plates = &'static [(&'static str, &'static [&'static [&'static str]])];
pub type Project = (&'static str, &'static str, Plates);

/// One kit: its parts, its Bambu Studio projects, and the description they carry.
pub struct Kit {
    pub parts: &'static [Part],
    pub projects: &'static [Project],
    pub description: &'static str,
}

const PIN_COUPONS: [&str; 4] = [
    "Pin coupon base",
    "Pin tab 1 dot (kit fit)",
    "Pin tab 2 dots (looser)",
    "Pin tab 3 dots (loosest)",
];
const PIN_PARTS: [Part; 4] = [
    p("Pin coupon base", "coupon-pin-base", true, WHITE),
    p("Pin tab 1 dot (kit fit)", "coupon-pin-tab", true, WHITE),
    p(
        "Pin tab 2 dots (looser)",
        "coupon-pin-tab-2-dots",
        true,
        WHITE,
    ),
    p(
        "Pin tab 3 dots (loosest)",
        "coupon-pin-tab-3-dots",
        true,
        WHITE,
    ),
];

pub const CLASSROOM: Kit = Kit {
    parts: &[
        p("Payload plate", "payload-plate", false, BLACK),
        p("Fit plate", "fit-plate", false, BLACK),
        p("Battery plate", "battery-plate", false, BLACK),
        PIN_PARTS[0],
        PIN_PARTS[1],
        PIN_PARTS[2],
        PIN_PARTS[3],
        p("Nano coupon", "coupon-nano-base", true, WHITE),
        p(
            "Side",
            "upstream/cubesatsim_frame_side.v2.0.stl",
            false,
            WHITE,
        ),
        p("Side with camera", "side-with-camera", false, WHITE),
        p("Camera coupon", "coupon-camera", true, WHITE),
        p("Bottom with columns", "bottom-with-columns", false, WHITE),
        p("Top with columns", "bottom-with-columns", false, WHITE),
    ],
    projects: &[
        (
            "cubesat-kit.3mf",
            "CubeSat Classroom kit",
            &[
                (
                    "Payload plate and bottom",
                    &[&["Payload plate", "Bottom with columns"]],
                ),
                ("Sides", &[&["Side", "Side with camera"]]),
                (
                    "Top and battery plate",
                    &[&["Top with columns", "Battery plate"]],
                ),
            ],
        ),
        (
            "cubesat-fit-plate.3mf",
            "CubeSat Classroom fit plate",
            &[("Fit plate", &[&["Fit plate"]])],
        ),
        (
            "cubesat-test-coupons.3mf",
            "CubeSat Classroom test coupons",
            &[(
                "Test coupons",
                &[&PIN_COUPONS, &["Nano coupon", "Camera coupon"]],
            )],
        ),
    ],
    description: "AMSAT CubeSatSim v2.0 frame with an Arduino Uno R3 and Nano 33 BLE Sense payload and a 3000 mAh \
                  battery. Frame by Alan Johnston, KU2Y (AMSAT). GPL-3.0.",
};

const OUTDOOR_COUPON_PARTS: [Part; 7] = [
    p("Tripod thread coupon", "coupon-tripod-thread", true, WHITE),
    p("Latch coupon box", "coupon-latch-box", false, WHITE),
    p("Latch coupon lid", "coupon-latch-lid", true, WHITE),
    PIN_PARTS[0],
    PIN_PARTS[1],
    PIN_PARTS[2],
    PIN_PARTS[3],
];

pub const OUTDOOR: Kit = Kit {
    parts: &[
        p("Box", "outdoor-box", false, WHITE),
        p("Lid", "outdoor-lid", false, WHITE),
        p("Payload plate", "payload-plate", false, BLACK),
        OUTDOOR_COUPON_PARTS[0],
        OUTDOOR_COUPON_PARTS[1],
        OUTDOOR_COUPON_PARTS[2],
        OUTDOOR_COUPON_PARTS[3],
        OUTDOOR_COUPON_PARTS[4],
        OUTDOOR_COUPON_PARTS[5],
        OUTDOOR_COUPON_PARTS[6],
    ],
    projects: &[
        (
            "outdoor-station.3mf",
            "CubeSat Classroom outdoor ground station",
            &[
                ("Box (ASA or PETG)", &[&["Box"]]),
                (
                    "Lid and payload plate (ASA or PETG)",
                    &[&["Lid", "Payload plate"]],
                ),
            ],
        ),
        (
            "outdoor-test-coupons.3mf",
            "CubeSat Classroom outdoor test coupons",
            &[(
                "Test coupons (ASA or PETG)",
                &[
                    &[
                        "Tripod thread coupon",
                        "Latch coupon box",
                        "Latch coupon lid",
                    ],
                    &PIN_COUPONS,
                ],
            )],
        ),
    ],
    description: "Outdoor, sealed CubeSat Classroom ground station for an Arduino Uno R3, a Nano 33 BLE Sense \
                  and a 3000 mAh battery, with a 1/4-20 tripod thread. GPL-3.0.",
};

pub const OUTDOOR_XIAO: Kit = Kit {
    parts: &[
        p("Box", "outdoor-box", false, WHITE),
        p("Lid", "outdoor-lid", false, WHITE),
        p("XIAO plate", "xiao-plate", false, BLACK),
        p("XIAO coupon", "coupon-xiao", true, WHITE),
        OUTDOOR_COUPON_PARTS[0],
        OUTDOOR_COUPON_PARTS[1],
        OUTDOOR_COUPON_PARTS[2],
        OUTDOOR_COUPON_PARTS[3],
        OUTDOOR_COUPON_PARTS[4],
        OUTDOOR_COUPON_PARTS[5],
        OUTDOOR_COUPON_PARTS[6],
    ],
    projects: &[
        (
            "outdoor-xiao.3mf",
            "CubeSat Classroom outdoor XIAO station",
            &[
                ("Box (ASA or PETG)", &[&["Box"]]),
                (
                    "Lid and XIAO plate (ASA or PETG)",
                    &[&["Lid", "XIAO plate"]],
                ),
            ],
        ),
        (
            "outdoor-xiao-test-coupons.3mf",
            "CubeSat Classroom outdoor XIAO test coupons",
            &[(
                "Test coupons (ASA or PETG)",
                &[
                    &["XIAO coupon"],
                    &[
                        "Tripod thread coupon",
                        "Latch coupon box",
                        "Latch coupon lid",
                    ],
                    &PIN_COUPONS,
                ],
            )],
        ),
    ],
    description: "Outdoor, sealed CubeSat Classroom station for a XIAO ESP32-S3 on the Wi-Fi HaLow board, a XIAO \
                  MG24 Sense and a 3000 mAh battery, with a 1/4-20 tripod thread. GPL-3.0.",
};

impl Part {
    /// The AMS slot the part's body prints from: 1 is white, 2 is black.
    fn filament(&self) -> u8 {
        if self.colour == BLACK { 2 } else { 1 }
    }
}

/// The logo's colour on a part: the other filament.
fn accent_colour(body: [f64; 3]) -> [f64; 3] {
    if body == BLACK { WHITE } else { BLACK }
}

/// A part's body and, if it has one, its two-colour logo, in the same coordinates.
#[derive(Clone)]
pub struct Volumes {
    pub body: Mesh,
    pub accent: Option<Mesh>,
}

impl Volumes {
    fn all(&self) -> Mesh {
        let mut m = self.body.clone();
        if let Some(a) = &self.accent {
            let n = m.verts.len() as u32;
            m.verts.extend(&a.verts);
            m.tris.extend(a.tris.iter().map(|t| t.map(|i| i + n)));
        }
        m
    }
}

fn part(kit: &Kit, name: &str) -> &'static Part {
    kit.parts
        .iter()
        .find(|p| p.name == name)
        .expect("every placed part is listed in the kit")
}

/// Each part's volumes, centred on the bed and standing on it. `brands` holds the body and logo
/// of every source that has a logo.
fn load_meshes(
    kit: &Kit,
    generated: &[(&str, Mesh)],
    brands: &[(&str, Mesh, Mesh)],
) -> Result<BTreeMap<&'static str, Volumes>, String> {
    let mut out = BTreeMap::new();
    for p in kit.parts {
        let whole = match p.source.strip_prefix("upstream/") {
            Some(file) => upstream(file)?,
            None => generated
                .iter()
                .find(|(n, _)| *n == p.source)
                .map(|(_, m)| m.clone())
                .ok_or_else(|| format!("{} wasn't built", p.source))?,
        };
        if !whole.is_closed() {
            return Err(format!("{}: not a closed solid", p.source));
        }
        let mut v = match brands.iter().find(|(n, ..)| *n == p.source) {
            Some((_, body, accent)) => {
                if !body.is_closed() || !accent.is_closed() {
                    return Err(format!("{}: its logo or body isn't closed", p.source));
                }
                let parts = body.volume() + accent.volume();
                if (parts - whole.volume()).abs() > 1e-3 * whole.volume() {
                    return Err(format!(
                        "{}: body and logo don't add up to the part",
                        p.source
                    ));
                }
                Volumes {
                    body: body.clone(),
                    accent: Some(accent.clone()),
                }
            }
            None => Volumes {
                body: whole,
                accent: None,
            },
        };
        let turn = |m: &mut Mesh| {
            if p.flip {
                // Half a turn about x.
                m.verts.iter_mut().for_each(|v| *v = [v[0], -v[1], -v[2]]);
            }
        };
        turn(&mut v.body);
        if let Some(a) = v.accent.as_mut() {
            turn(a);
        }
        let (lo, hi) = v.all().bounds();
        let shift = [-(lo[0] + hi[0]) / 2.0, -(lo[1] + hi[1]) / 2.0, -lo[2]];
        v.body = v.body.translated(shift);
        v.accent = v.accent.map(|a| a.translated(shift));
        out.insert(p.name, v);
    }
    Ok(out)
}

/// Centre each row on the bed and stack the rows top to bottom. Returns (name, x, y).
fn layout(
    rows: &[&[&'static str]],
    meshes: &BTreeMap<&str, Volumes>,
) -> Result<Vec<(&'static str, f64, f64)>, String> {
    let size = |n: &str| meshes[n].all().extents();
    let heights: Vec<f64> = rows
        .iter()
        .map(|r| r.iter().map(|n| size(n)[1]).fold(0.0, f64::max))
        .collect();
    let total = heights.iter().sum::<f64>() + ROW_GAP * (rows.len() as f64 - 1.0);
    let mut y = BED / 2.0 + total / 2.0;
    let mut placed = Vec::new();
    for (row, h) in rows.iter().zip(heights) {
        let width = row.iter().map(|n| size(n)[0]).sum::<f64>() + GAP * (row.len() as f64 - 1.0);
        if width > BED - 2.0 * MARGIN || total > BED - 2.0 * MARGIN {
            return Err(format!("{row:?} does not fit the 256 mm plate"));
        }
        let mut x = BED / 2.0 - width / 2.0;
        for n in *row {
            let w = size(n)[0];
            placed.push((*n, x + w / 2.0, y - h / 2.0));
            x += w + GAP;
        }
        y -= h + ROW_GAP;
    }
    Ok(placed)
}

/// Bambu Studio's compute_colum_count() and compute_shape_position().
fn plate_origin(index: usize, count: usize) -> (f64, f64) {
    let value = (count as f64).sqrt();
    let cols = if value > value.round() {
        value.round() as usize + 1
    } else {
        value.round() as usize
    };
    (
        (index % cols) as f64 * PLATE_STRIDE,
        -((index / cols) as f64) * PLATE_STRIDE,
    )
}

fn fmt(v: f64) -> String {
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() {
        "0".into()
    } else {
        s.into()
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

const MODEL_HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xml:lang=\"en-US\" \
    xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" \
    xmlns:BambuStudio=\"http://schemas.bambulab.com/package/2021\" \
    xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n";

fn mesh_xml(mesh: &Mesh, id: usize, uuid: &str) -> String {
    let mut out = format!(
        "  <object id=\"{id}\" p:UUID=\"{uuid}\" type=\"model\">\n   <mesh>\n    <vertices>\n"
    );
    for v in &mesh.verts {
        let _ = writeln!(
            out,
            "     <vertex x=\"{}\" y=\"{}\" z=\"{}\"/>",
            fmt(v[0]),
            fmt(v[1]),
            fmt(v[2])
        );
    }
    out.push_str("    </vertices>\n    <triangles>\n");
    for t in &mesh.tris {
        let _ = writeln!(
            out,
            "     <triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>",
            t[0], t[1], t[2]
        );
    }
    out.push_str("    </triangles>\n   </mesh>\n  </object>\n");
    out
}

// ---------- preview pictures: a small orthographic z-buffer renderer ----------

/// RGB image, row by row.
pub struct Image {
    pub size: usize,
    pub rgb: Vec<u8>,
}

/// Isometric view of meshes in plate coordinates on a 256 mm bed.
pub fn render(items: &[(Mesh, [f64; 3])], size: usize) -> Image {
    let (az, el) = ((-35f64).to_radians(), 52f64.to_radians());
    let rot = [
        [az.cos(), -az.sin(), 0.0],
        [az.sin(), az.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ];
    let tilt = [
        [1.0, 0.0, 0.0],
        [0.0, el.cos(), -el.sin()],
        [0.0, el.sin(), el.cos()],
    ];
    let view: [[f64; 3]; 3] = std::array::from_fn(|i| {
        std::array::from_fn(|j| (0..3).map(|k| tilt[i][k] * rot[k][j]).sum())
    });
    let apply = |v: [f64; 3]| -> [f64; 3] {
        std::array::from_fn(|i| v[0] * view[i][0] + v[1] * view[i][1] + v[2] * view[i][2])
    };
    let l: [f64; 3] = [-0.35, 0.45, 0.82];
    let ln = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
    let light = [l[0] / ln, l[1] / ln, l[2] / ln];

    let bed = bed_mesh();
    let mut scene: Vec<(&Mesh, [f64; 3])> = vec![(&bed, PEI)];
    scene.extend(items.iter().map(|(m, c)| (m, *c)));
    let centred = |v: &[f64; 3]| apply([v[0] - BED / 2.0, v[1] - BED / 2.0, v[2]]);
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (m, _) in &scene {
        for v in &m.verts {
            let p = centred(v);
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
    }
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]) * 1.06;
    let centre = [(hi[0] + lo[0]) / 2.0, (hi[1] + lo[1]) / 2.0];
    let scale = size as f64 / span;
    let half = size as f64 / 2.0;
    let mut rgb = vec![255u8; size * size * 3];
    let mut depth = vec![f64::NEG_INFINITY; size * size];
    for (mesh, colour) in scene {
        let v: Vec<[f64; 3]> = mesh.verts.iter().map(centred).collect();
        let sx: Vec<f64> = v
            .iter()
            .map(|p| (p[0] - centre[0]) * scale + half)
            .collect();
        let sy: Vec<f64> = v
            .iter()
            .map(|p| half - (p[1] - centre[1]) * scale)
            .collect();
        for (f, tri) in mesh.tris.iter().enumerate() {
            let n = mesh.normal(f);
            if apply(n)[2] <= 0.0 {
                continue;
            }
            let shade =
                0.45 + 0.55 * (n[0] * light[0] + n[1] * light[1] + n[2] * light[2]).clamp(0.0, 1.0);
            let idx = tri.map(|i| i as usize);
            let (xs, ys, zs) = (idx.map(|i| sx[i]), idx.map(|i| sy[i]), idx.map(|i| v[i][2]));
            let x0 = (xs.iter().copied().fold(f64::INFINITY, f64::min) as i64).max(0);
            let x1 = (xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) as i64 + 1)
                .min(size as i64 - 1);
            let y0 = (ys.iter().copied().fold(f64::INFINITY, f64::min) as i64).max(0);
            let y1 = (ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) as i64 + 1)
                .min(size as i64 - 1);
            let det = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2]);
            if x1 < x0 || y1 < y0 || det.abs() < 1e-12 {
                continue;
            }
            let pixel = colour.map(|c| (c * 255.0 * shade).clamp(0.0, 255.0) as u8);
            for py in y0..=y1 {
                for px in x0..=x1 {
                    let (gx, gy) = (px as f64 + 0.5, py as f64 + 0.5);
                    let a = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / det;
                    let b = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / det;
                    let c = 1.0 - a - b;
                    if a < 0.0 || b < 0.0 || c < 0.0 {
                        continue;
                    }
                    let z = a * zs[0] + b * zs[1] + c * zs[2];
                    let k = py as usize * size + px as usize;
                    if z > depth[k] {
                        depth[k] = z;
                        rgb[3 * k..3 * k + 3].copy_from_slice(&pixel);
                    }
                }
            }
        }
    }
    Image { size, rgb }
}

fn bed_mesh() -> Mesh {
    let (a, b) = (0.0, BED);
    let verts = vec![
        [a, a, -1.0],
        [b, a, -1.0],
        [b, b, -1.0],
        [a, b, -1.0],
        [a, a, 0.0],
        [b, a, 0.0],
        [b, b, 0.0],
        [a, b, 0.0],
    ];
    let tris = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    Mesh { verts, tris }
}

pub fn png(img: &Image) -> Vec<u8> {
    let mut raw = Vec::with_capacity(img.size * (img.size * 3 + 1));
    for row in img.rgb.chunks(img.size * 3) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = ZlibEncoder::new(Vec::new(), Compression::best());
    z.write_all(&raw).expect("in-memory write");
    let data = z.finish().expect("in-memory write");
    let chunk = |kind: &[u8], body: &[u8]| -> Vec<u8> {
        let mut c = (body.len() as u32).to_be_bytes().to_vec();
        c.extend_from_slice(kind);
        c.extend_from_slice(body);
        let mut h = crc32fast::Hasher::new();
        h.update(kind);
        h.update(body);
        c.extend_from_slice(&h.finalize().to_be_bytes());
        c
    };
    let size = (img.size as u32).to_be_bytes();
    let mut ihdr = [size, size].concat();
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    [
        b"\x89PNG\r\n\x1a\n".to_vec(),
        chunk(b"IHDR", &ihdr),
        chunk(b"IDAT", &data),
        chunk(b"IEND", &[]),
    ]
    .concat()
}

// ---------- zip ----------

/// A zip archive with fixed timestamps, so it is the same on every run.
fn zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::new(6));
        enc.write_all(data).expect("in-memory write");
        let packed = enc.finish().expect("in-memory write");
        let crc = crc32fast::hash(data);
        let offset = out.len() as u32;
        let common = |v: &mut Vec<u8>| {
            v.extend_from_slice(&20u16.to_le_bytes()); // version needed
            v.extend_from_slice(&0u16.to_le_bytes()); // flags
            v.extend_from_slice(&8u16.to_le_bytes()); // deflate
            v.extend_from_slice(&0u16.to_le_bytes()); // time 00:00
            v.extend_from_slice(&0x21u16.to_le_bytes()); // date 1980-01-01
            v.extend_from_slice(&crc.to_le_bytes());
            v.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            v.extend_from_slice(&(data.len() as u32).to_le_bytes());
            v.extend_from_slice(&(name.len() as u16).to_le_bytes());
            v.extend_from_slice(&0u16.to_le_bytes()); // extra
        };
        out.extend_from_slice(b"PK\x03\x04");
        common(&mut out);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&packed);
        central.extend_from_slice(b"PK\x01\x02");
        central.extend_from_slice(&20u16.to_le_bytes()); // made by
        common(&mut central);
        central.extend_from_slice(&[0; 6]); // comment length, disk, internal attributes
        central.extend_from_slice(&0u32.to_le_bytes()); // external attributes
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let start = out.len() as u32;
    let count = files.len() as u16;
    let size = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(b"PK\x05\x06");
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&[0; 2]);
    out
}

// ---------- projects ----------

/// Ids of an object's body mesh, logo mesh and object, and its object file number.
struct Ids {
    body: usize,
    accent: Option<usize>,
    obj: usize,
    num: usize,
}

struct Placed {
    label: String,
    part: &'static str,
    plate: usize,
    x: f64,
    y: f64,
}

/// One object per placed part; repeated parts get numbered names.
fn place_objects(meshes: &BTreeMap<&str, Volumes>, plates: Plates) -> Result<Vec<Placed>, String> {
    let mut placed = Vec::new();
    for (p, (_, rows)) in plates.iter().enumerate() {
        for (n, x, y) in layout(rows, meshes)? {
            placed.push((n, p, x, y));
        }
    }
    let total = |n: &str| placed.iter().filter(|(m, ..)| *m == n).count();
    let mut counters: BTreeMap<&str, usize> = BTreeMap::new();
    Ok(placed
        .iter()
        .map(|&(n, p, x, y)| {
            let c = counters.entry(n).or_insert(0);
            *c += 1;
            let label = if total(n) > 1 {
                format!("{n} {c}")
            } else {
                n.to_string()
            };
            Placed {
                label,
                part: n,
                plate: p,
                x,
                y,
            }
        })
        .collect())
}

fn project(
    kit: &Kit,
    title: &str,
    plates: Plates,
    meshes: &BTreeMap<&str, Volumes>,
) -> Result<(Vec<u8>, Vec<Placed>), String> {
    let objects = place_objects(meshes, plates)?;
    // Bambu numbering: each object's meshes (its body, then its logo) take ids, then the object.
    let mut next = 0;
    let ids: Vec<Ids> = objects
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let body = next + 1;
            let accent = meshes[o.part].accent.as_ref().map(|_| body + 1);
            let obj = accent.unwrap_or(body) + 1;
            next = obj;
            Ids {
                body,
                accent,
                obj,
                num: i + 1,
            }
        })
        .collect();
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut rels = Vec::new();
    for (o, id) in objects.iter().zip(&ids) {
        let num = id.num;
        let path = format!("3D/Objects/object_{num}.model");
        let v = &meshes[o.part];
        // Check each volume as the 3MF stores it: six decimals, equal corners merged.
        for (what, m) in [("body", Some(&v.body)), ("logo", v.accent.as_ref())] {
            let Some(m) = m else { continue };
            let stored = m.quantized(|x| fmt(x).parse().unwrap_or(f64::NAN));
            if let Some(e) = stored.defects() {
                return Err(format!("{}: its {what} in {title} has {e}", o.label));
            }
        }
        let mut res = mesh_xml(
            &v.body,
            id.body,
            &format!("{:08x}-81cb-4c03-9d28-80fed5dfa1dc", num << 16),
        );
        if let (Some(a), Some(aid)) = (&v.accent, id.accent) {
            res.push_str(&mesh_xml(
                a,
                aid,
                &format!("{:08x}-81cb-4c03-9d28-80fed5dfa1dc", (num << 16) + 1),
            ));
        }
        let body = format!("{MODEL_HEAD} <resources>\n{res} </resources>\n <build/>\n</model>\n");
        files.insert(path.clone(), body.into_bytes());
        rels.push(path);
    }

    let mut model = String::from(MODEL_HEAD);
    let meta = BTreeMap::from([
        ("Application", APP.to_string()),
        ("BambuStudio:3mfVersion", "1".to_string()),
        ("CreationDate", DATE.to_string()),
        ("Description", kit.description.to_string()),
        ("Designer", "CubeSat Classroom".to_string()),
        ("License", "GPL-3.0".to_string()),
        ("ModificationDate", DATE.to_string()),
        ("Title", title.to_string()),
    ]);
    for (k, v) in &meta {
        let _ = writeln!(model, " <metadata name=\"{k}\">{}</metadata>", escape(v));
    }
    model.push_str(" <resources>\n");
    for id in &ids {
        let num = id.num;
        let _ = write!(
            model,
            "  <object id=\"{}\" p:UUID=\"{num:08x}-61cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   <components>\n",
            id.obj
        );
        for (k, mesh_id) in std::iter::once(id.body).chain(id.accent).enumerate() {
            let _ = writeln!(
                model,
                "    <component p:path=\"/3D/Objects/object_{num}.model\" objectid=\"{mesh_id}\" \
                 p:UUID=\"{:08x}-b206-40ff-9872-83e8017abed1\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"/>",
                (num << 16) + k
            );
        }
        model.push_str("   </components>\n  </object>\n");
    }
    model.push_str(" </resources>\n <build p:UUID=\"2c7c17d8-22b5-4d84-8835-1976022ea369\">\n");
    for (o, id) in objects.iter().zip(&ids) {
        let obj_id = id.obj;
        let (ox, oy) = plate_origin(o.plate, plates.len());
        let _ = writeln!(
            model,
            "  <item objectid=\"{obj_id}\" p:UUID=\"{obj_id:08x}-b1ec-4553-aec9-835e5b724bb4\" \
             transform=\"1 0 0 0 1 0 0 0 1 {} {} 0\" printable=\"1\"/>",
            fmt(ox + o.x),
            fmt(oy + o.y)
        );
    }
    model.push_str(" </build>\n</model>\n");
    files.insert("3D/3dmodel.model".into(), model.into_bytes());

    let mut cfg = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<config>\n");
    for (o, id) in objects.iter().zip(&ids) {
        let v = &meshes[o.part];
        let pt = part(kit, o.part);
        let faces = v.body.tris.len() + v.accent.as_ref().map_or(0, |a| a.tris.len());
        let label = escape(&o.label);
        let _ = writeln!(cfg, "  <object id=\"{}\">", id.obj);
        let _ = writeln!(cfg, "    <metadata key=\"name\" value=\"{label}\"/>");
        let _ = writeln!(
            cfg,
            "    <metadata key=\"extruder\" value=\"{}\"/>",
            pt.filament()
        );
        if pt.solid_infill {
            let _ = writeln!(
                cfg,
                "    <metadata key=\"sparse_infill_density\" value=\"100%\"/>"
            );
        }
        let _ = writeln!(cfg, "    <metadata face_count=\"{faces}\"/>");
        let mut volumes = vec![(id.body, label.clone(), pt.filament(), v.body.tris.len())];
        if let (Some(a), Some(aid)) = (&v.accent, id.accent) {
            volumes.push((
                aid,
                format!("{label} logo"),
                3 - pt.filament(),
                a.tris.len(),
            ));
        }
        for (pid, name, filament, f) in volumes {
            let _ = write!(
                cfg,
                "    <part id=\"{pid}\" subtype=\"normal_part\">\n      \
                 <metadata key=\"name\" value=\"{name}\"/>\n      \
                 <metadata key=\"matrix\" value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"/>\n      \
                 <metadata key=\"extruder\" value=\"{filament}\"/>\n      \
                 <mesh_stat face_count=\"{f}\" edges_fixed=\"0\" degenerate_facets=\"0\" facets_removed=\"0\" \
                 facets_reversed=\"0\" backwards_edges=\"0\"/>\n    </part>\n"
            );
        }
        cfg.push_str("  </object>\n");
    }
    for (p, (plate_name, _)) in plates.iter().enumerate() {
        let _ = write!(
            cfg,
            "  <plate>\n    <metadata key=\"plater_id\" value=\"{}\"/>\n    \
             <metadata key=\"plater_name\" value=\"{}\"/>\n    <metadata key=\"locked\" value=\"false\"/>\n    \
             <metadata key=\"thumbnail_file\" value=\"Metadata/plate_{}.png\"/>\n",
            p + 1,
            escape(plate_name),
            p + 1
        );
        for (o, id) in objects.iter().zip(&ids) {
            if o.plate == p {
                let _ = write!(
                    cfg,
                    "    <model_instance>\n      <metadata key=\"object_id\" value=\"{}\"/>\n      \
                     <metadata key=\"instance_id\" value=\"0\"/>\n      \
                     <metadata key=\"identify_id\" value=\"{}\"/>\n    </model_instance>\n",
                    id.obj,
                    1000 + id.num
                );
            }
        }
        cfg.push_str("  </plate>\n");
    }
    cfg.push_str("</config>\n");
    files.insert("Metadata/model_settings.config".into(), cfg.into_bytes());

    for p in 0..plates.len() {
        let mut scene: Vec<(Mesh, [f64; 3])> = Vec::new();
        for o in objects.iter().filter(|o| o.plate == p) {
            let v = &meshes[o.part];
            let colour = part(kit, o.part).colour;
            scene.push((v.body.translated([o.x, o.y, 0.0]), colour));
            if let Some(a) = &v.accent {
                scene.push((a.translated([o.x, o.y, 0.0]), accent_colour(colour)));
            }
        }
        files.insert(
            format!("Metadata/plate_{}.png", p + 1),
            png(&render(&scene, 512)),
        );
        files.insert(
            format!("Metadata/plate_{}_small.png", p + 1),
            png(&render(&scene, 128)),
        );
    }

    let rel = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";
    let head = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    let content_types = format!(
        "{head}<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n \
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n \
         <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\n \
         <Default Extension=\"png\" ContentType=\"image/png\"/>\n \
         <Default Extension=\"gcode\" ContentType=\"text/x.gcode\"/>\n</Types>"
    );
    let root_rels = format!(
        "{head}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n \
         <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel-1\" Type=\"{rel}\"/>\n \
         <Relationship Target=\"/Metadata/plate_1.png\" Id=\"rel-2\" \
         Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\n \
         <Relationship Target=\"/Metadata/plate_1.png\" Id=\"rel-4\" \
         Type=\"http://schemas.bambulab.com/package/2021/cover-thumbnail-middle\"/>\n \
         <Relationship Target=\"/Metadata/plate_1_small.png\" Id=\"rel-5\" \
         Type=\"http://schemas.bambulab.com/package/2021/cover-thumbnail-small\"/>\n</Relationships>"
    );
    let mut model_rels = format!(
        "{head}<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n"
    );
    for (i, r) in rels.iter().enumerate() {
        let _ = writeln!(
            model_rels,
            " <Relationship Target=\"/{r}\" Id=\"rel-{}\" Type=\"{rel}\"/>",
            i + 1
        );
    }
    model_rels.push_str("</Relationships>");

    let mut ordered = vec![
        (
            "[Content_Types].xml".to_string(),
            content_types.into_bytes(),
        ),
        ("_rels/.rels".to_string(), root_rels.into_bytes()),
        (
            "3D/3dmodel.model".to_string(),
            files.remove("3D/3dmodel.model").unwrap_or_default(),
        ),
        (
            "3D/_rels/3dmodel.model.rels".to_string(),
            model_rels.into_bytes(),
        ),
    ];
    ordered.extend(files);
    Ok((zip(&ordered), objects))
}

/// Read the project back and check the plate split: every object on exactly one plate, once,
/// with a single instance, lying inside that plate's area.
fn check(
    model: &str,
    cfg: &str,
    plates: usize,
    objects: &[Placed],
    meshes: &BTreeMap<&str, Volumes>,
) -> Result<(), String> {
    let attr = |line: &str, key: &str| -> Option<String> {
        let at = line.find(&format!("{key}=\""))? + key.len() + 2;
        Some(line[at..].split('"').next()?.to_string())
    };
    let mut position = BTreeMap::new();
    let mut order = Vec::new();
    for line in model
        .lines()
        .filter(|l| l.trim_start().starts_with("<item "))
    {
        let obj = attr(line, "objectid").ok_or("item without an object")?;
        let t: Vec<f64> = attr(line, "transform")
            .unwrap_or_default()
            .split(' ')
            .filter_map(|v| v.parse().ok())
            .collect();
        order.push(obj.clone());
        if t.len() != 12 || position.insert(obj.clone(), (t[9], t[10])).is_some() {
            return Err(format!("object {obj} must have exactly one instance"));
        }
    }
    if position.len() != objects.len() {
        return Err("every object must have exactly one instance".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for (p, plate) in cfg.split("<plate>").skip(1).enumerate() {
        let (ox, oy) = plate_origin(p, plates);
        for line in plate.lines().filter(|l| l.contains("key=\"object_id\"")) {
            let obj = attr(line, "value").ok_or("plate entry without an object")?;
            if !seen.insert(obj.clone()) {
                return Err(format!("object {obj} is on more than one plate"));
            }
            let index = order
                .iter()
                .position(|o| *o == obj)
                .ok_or("plate entry for an unknown object")?;
            let o = &objects[index];
            let [w, dpt, _] = meshes[o.part].all().extents();
            let (x, y) = position[&obj];
            if !(ox + w / 2.0 <= x
                && x <= ox + BED - w / 2.0
                && oy + dpt / 2.0 <= y
                && y <= oy + BED - dpt / 2.0)
            {
                return Err(format!(
                    "{} on plate {} lies outside that plate",
                    o.label,
                    p + 1
                ));
            }
        }
    }
    if seen.len() != position.len() {
        return Err("some objects are on no plate".into());
    }
    // Every object with a logo prints it in the other filament.
    for obj in cfg.split("<object id=").skip(1) {
        let obj = obj.split("</object>").next().unwrap_or_default();
        let filaments: Vec<String> = obj
            .split("<part ")
            .skip(1)
            .filter_map(|part| {
                part.lines()
                    .find(|l| l.contains("key=\"extruder\""))
                    .and_then(|l| attr(l, "value"))
            })
            .collect();
        if filaments.len() == 2 && filaments[0] == filaments[1] {
            return Err("a logo prints in the same filament as its part".into());
        }
    }
    Ok(())
}

/// Write every Bambu Studio project into `dir/p1s/`.
pub fn write_projects(
    dir: &Path,
    kit: &Kit,
    generated: &[(&str, Mesh)],
    brands: &[(&str, Mesh, Mesh)],
) -> Result<(), String> {
    let meshes = load_meshes(kit, generated, brands)?;
    let out = dir.join("p1s");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for &(file, title, plates) in kit.projects {
        let (data, objects) = project(kit, title, plates, &meshes)?;
        // The model and config go into the archive as written: check what Bambu Studio will read.
        let (model, cfg) = read_back(&data)?;
        check(&model, &cfg, plates.len(), &objects, &meshes)?;
        std::fs::write(out.join(file), data).map_err(|e| e.to_string())?;
        println!("wrote p1s/{file}");
        for (p, (name, _)) in plates.iter().enumerate() {
            println!(
                "  plate {}, {name}: {} parts",
                p + 1,
                objects.iter().filter(|o| o.plate == p).count()
            );
        }
    }
    Ok(())
}

/// The model and config files from an archive written by `zip`.
fn read_back(data: &[u8]) -> Result<(String, String), String> {
    let mut found = BTreeMap::new();
    let mut at = 0;
    while data.get(at..at + 4) == Some(b"PK\x03\x04") {
        let u16_at = |o: usize| u16::from_le_bytes([data[at + o], data[at + o + 1]]) as usize;
        let u32_at = |o: usize| {
            u32::from_le_bytes(data[at + o..at + o + 4].try_into().expect("4 bytes")) as usize
        };
        let (packed, name_len, extra) = (u32_at(18), u16_at(26), u16_at(28));
        let name = String::from_utf8_lossy(&data[at + 30..at + 30 + name_len]).to_string();
        let start = at + 30 + name_len + extra;
        if name == "3D/3dmodel.model" || name == "Metadata/model_settings.config" {
            let mut text = String::new();
            std::io::Read::read_to_string(
                &mut flate2::read::DeflateDecoder::new(&data[start..start + packed]),
                &mut text,
            )
            .map_err(|e| e.to_string())?;
            found.insert(name, text);
        }
        at = start + packed;
    }
    match (
        found.remove("3D/3dmodel.model"),
        found.remove("Metadata/model_settings.config"),
    ) {
        (Some(m), Some(c)) => Ok((m, c)),
        _ => Err("the project is missing its model or config".into()),
    }
}
