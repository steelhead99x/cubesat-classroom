//! Bambu Studio project files for a Bambu Lab printer with a 256 mm bed (P1S, P1P, X1C, X1E, A1).
//!
//! Every part is placed in its print orientation as its own object (Bambu Studio keeps one
//! instance per object per plate, so copies of one object can't be spread across plates). The
//! coupons are set to 100% infill, and each plate has a preview picture. The file follows Bambu
//! Studio's own project layout (src/libslic3r/Format/bbs_3mf.cpp): per-object meshes in
//! 3D/Objects/, plates and object settings in Metadata/model_settings.config. It carries no
//! printer or filament profile, so Bambu Studio keeps the ones you have selected. The output is
//! the same byte for byte on every run.

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

struct Part {
    name: &'static str,
    /// A generated mesh's name, or an AMSAT upstream file.
    source: &'static str,
    flip: bool,
    solid_infill: bool,
    colour: [f64; 3],
}

const GREY: [f64; 3] = [0.62, 0.65, 0.70];

const PARTS: [Part; 10] = [
    Part {
        name: "Payload plate",
        source: "payload-plate",
        flip: false,
        solid_infill: false,
        colour: GREY,
    },
    Part {
        name: "Fit plate",
        source: "fit-plate",
        flip: false,
        solid_infill: false,
        colour: [0.22, 0.24, 0.27],
    },
    Part {
        name: "Pin coupon base",
        source: "coupon-pin-base",
        flip: false,
        solid_infill: true,
        colour: [0.55, 0.60, 0.68],
    },
    Part {
        name: "Pin coupon tab",
        source: "coupon-pin-tab",
        flip: false,
        solid_infill: true,
        colour: GREY,
    },
    Part {
        name: "Hook coupon base",
        source: "coupon-hook-base",
        flip: false,
        solid_infill: true,
        colour: [0.55, 0.60, 0.68],
    },
    Part {
        name: "Coupon board",
        source: "coupon-board",
        flip: false,
        solid_infill: false,
        colour: [0.10, 0.45, 0.35],
    },
    Part {
        name: "Side",
        source: "upstream/cubesatsim_frame_side.v2.0.stl",
        flip: false,
        solid_infill: false,
        colour: GREY,
    },
    Part {
        name: "Side with camera hole",
        source: "upstream/cubesatsim_frame_side_camera.v2.0.stl",
        flip: false,
        solid_infill: false,
        colour: GREY,
    },
    Part {
        name: "Top",
        source: "upstream/cubesatsim_frame_top_bottom.v2.0.stl",
        flip: true,
        solid_infill: false,
        colour: GREY,
    },
    Part {
        name: "Bottom with columns",
        source: "bottom-with-columns",
        flip: false,
        solid_infill: false,
        colour: GREY,
    },
];

/// File, title, and plates: each plate a name and rows of parts, top to bottom.
type Plates = &'static [(&'static str, &'static [&'static [&'static str]])];
const PROJECTS: [(&str, &str, Plates); 3] = [
    (
        "cubesat-kit.3mf",
        "CubeSat Classroom kit",
        &[
            (
                "Payload plate and bottom",
                &[&["Payload plate", "Bottom with columns"]],
            ),
            (
                "Sides and top",
                &[&["Side", "Side with camera hole"], &["Top"]],
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
            &[
                &["Pin coupon base", "Pin coupon tab"],
                &["Hook coupon base", "Coupon board"],
            ],
        )],
    ),
];

fn part(name: &str) -> &'static Part {
    PARTS
        .iter()
        .find(|p| p.name == name)
        .expect("every placed part is listed in PARTS")
}

/// Each part's mesh, centred on the bed and standing on it.
fn load_meshes(generated: &[(&str, Mesh)]) -> Result<BTreeMap<&'static str, Mesh>, String> {
    let mut out = BTreeMap::new();
    for p in &PARTS {
        let mut mesh = match p.source.strip_prefix("upstream/") {
            Some(file) => upstream(file)?,
            None => generated
                .iter()
                .find(|(n, _)| *n == p.source)
                .map(|(_, m)| m.clone())
                .ok_or_else(|| format!("{} wasn't built", p.source))?,
        };
        if !mesh.is_closed() {
            return Err(format!("{}: not a closed solid", p.source));
        }
        if p.flip {
            // Half a turn about x.
            mesh.verts
                .iter_mut()
                .for_each(|v| *v = [v[0], -v[1], -v[2]]);
        }
        let (lo, hi) = mesh.bounds();
        out.insert(
            p.name,
            mesh.translated([-(lo[0] + hi[0]) / 2.0, -(lo[1] + hi[1]) / 2.0, -lo[2]]),
        );
    }
    Ok(out)
}

/// Centre each row on the bed and stack the rows top to bottom. Returns (name, x, y).
fn layout(
    rows: &[&[&'static str]],
    meshes: &BTreeMap<&str, Mesh>,
) -> Result<Vec<(&'static str, f64, f64)>, String> {
    let size = |n: &str| meshes[n].extents();
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
    let mut scene: Vec<(&Mesh, [f64; 3])> = vec![(&bed, [0.17, 0.18, 0.20])];
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

struct Placed {
    label: String,
    part: &'static str,
    plate: usize,
    x: f64,
    y: f64,
}

/// One object per placed part; repeated parts get numbered names.
fn place_objects(meshes: &BTreeMap<&str, Mesh>, plates: Plates) -> Result<Vec<Placed>, String> {
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
    title: &str,
    plates: Plates,
    meshes: &BTreeMap<&str, Mesh>,
) -> Result<(Vec<u8>, Vec<Placed>), String> {
    let objects = place_objects(meshes, plates)?;
    // Bambu numbering: each object's mesh (its one part) takes an id, the object the next one.
    let ids: Vec<(usize, usize, usize)> = (0..objects.len())
        .map(|i| (2 * i + 1, 2 * i + 2, i + 1))
        .collect();
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut rels = Vec::new();
    for (o, &(part_id, _, num)) in objects.iter().zip(&ids) {
        let path = format!("3D/Objects/object_{num}.model");
        let uuid = format!("{:08x}-81cb-4c03-9d28-80fed5dfa1dc", num << 16);
        let body = format!(
            "{MODEL_HEAD} <resources>\n{} </resources>\n <build/>\n</model>\n",
            mesh_xml(&meshes[o.part], part_id, &uuid)
        );
        files.insert(path.clone(), body.into_bytes());
        rels.push(path);
    }

    let mut model = String::from(MODEL_HEAD);
    let meta = BTreeMap::from([
        ("Application", APP.to_string()),
        ("BambuStudio:3mfVersion", "1".to_string()),
        ("CreationDate", DATE.to_string()),
        (
            "Description",
            "AMSAT CubeSatSim v2.0 frame with an Arduino Uno R3 and Nano 33 BLE Sense payload. Frame by Alan \
             Johnston, KU2Y (AMSAT). GPL-3.0."
                .to_string(),
        ),
        ("Designer", "CubeSat Classroom".to_string()),
        ("License", "GPL-3.0".to_string()),
        ("ModificationDate", DATE.to_string()),
        ("Title", title.to_string()),
    ]);
    for (k, v) in &meta {
        let _ = writeln!(model, " <metadata name=\"{k}\">{}</metadata>", escape(v));
    }
    model.push_str(" <resources>\n");
    for &(part_id, obj_id, num) in &ids {
        let _ = write!(
            model,
            "  <object id=\"{obj_id}\" p:UUID=\"{num:08x}-61cb-4c03-9d28-80fed5dfa1dc\" type=\"model\">\n   \
             <components>\n    <component p:path=\"/3D/Objects/object_{num}.model\" objectid=\"{part_id}\" \
             p:UUID=\"{:08x}-b206-40ff-9872-83e8017abed1\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"/>\n   \
             </components>\n  </object>\n",
            num << 16
        );
    }
    model.push_str(" </resources>\n <build p:UUID=\"2c7c17d8-22b5-4d84-8835-1976022ea369\">\n");
    for (o, &(_, obj_id, _)) in objects.iter().zip(&ids) {
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
    for (o, &(part_id, obj_id, _)) in objects.iter().zip(&ids) {
        let faces = meshes[o.part].tris.len();
        let label = escape(&o.label);
        let _ = writeln!(cfg, "  <object id=\"{obj_id}\">");
        let _ = writeln!(cfg, "    <metadata key=\"name\" value=\"{label}\"/>");
        let _ = writeln!(cfg, "    <metadata key=\"extruder\" value=\"1\"/>");
        if part(o.part).solid_infill {
            let _ = writeln!(
                cfg,
                "    <metadata key=\"sparse_infill_density\" value=\"100%\"/>"
            );
        }
        let _ = write!(
            cfg,
            "    <metadata face_count=\"{faces}\"/>\n    <part id=\"{part_id}\" subtype=\"normal_part\">\n      \
             <metadata key=\"name\" value=\"{label}\"/>\n      \
             <metadata key=\"matrix\" value=\"1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1\"/>\n      \
             <mesh_stat face_count=\"{faces}\" edges_fixed=\"0\" degenerate_facets=\"0\" facets_removed=\"0\" \
             facets_reversed=\"0\" backwards_edges=\"0\"/>\n    </part>\n  </object>\n"
        );
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
        for (o, &(_, obj_id, num)) in objects.iter().zip(&ids) {
            if o.plate == p {
                let _ = write!(
                    cfg,
                    "    <model_instance>\n      <metadata key=\"object_id\" value=\"{obj_id}\"/>\n      \
                     <metadata key=\"instance_id\" value=\"0\"/>\n      \
                     <metadata key=\"identify_id\" value=\"{}\"/>\n    </model_instance>\n",
                    1000 + num
                );
            }
        }
        cfg.push_str("  </plate>\n");
    }
    cfg.push_str("</config>\n");
    files.insert("Metadata/model_settings.config".into(), cfg.into_bytes());

    for p in 0..plates.len() {
        let scene: Vec<(Mesh, [f64; 3])> = objects
            .iter()
            .filter(|o| o.plate == p)
            .map(|o| {
                (
                    meshes[o.part].translated([o.x, o.y, 0.0]),
                    part(o.part).colour,
                )
            })
            .collect();
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
    meshes: &BTreeMap<&str, Mesh>,
) -> Result<(), String> {
    let attr = |line: &str, key: &str| -> Option<String> {
        let at = line.find(&format!("{key}=\""))? + key.len() + 2;
        Some(line[at..].split('"').next()?.to_string())
    };
    let mut position = BTreeMap::new();
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
            let index: usize = obj.parse::<usize>().map_err(|e| e.to_string())? / 2 - 1;
            let o = &objects[index];
            let [w, dpt, _] = meshes[o.part].extents();
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
    Ok(())
}

/// Write every Bambu Studio project into `dir/p1s/`.
pub fn write_projects(dir: &Path, generated: &[(&str, Mesh)]) -> Result<(), String> {
    let meshes = load_meshes(generated)?;
    let out = dir.join("p1s");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for (file, title, plates) in PROJECTS {
        let (data, objects) = project(title, plates, &meshes)?;
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
