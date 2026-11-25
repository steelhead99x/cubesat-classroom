//! Triangle meshes: extraction from Manifold, integrity checks, mass properties,
//! overhang measurement, and STL and 3MF output.

use std::collections::HashMap;
use std::io::{self, Write};

use manifold_csg::Manifold;

/// An indexed triangle mesh in mm.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

impl Mesh {
    /// Extract the mesh of a solid, failing if the kernel reports an error.
    pub fn from_solid(name: &str, solid: &Manifold) -> Result<Self, String> {
        solid
            .status()
            .map_err(|e| format!("{name}: geometry error {e:?}"))?;
        let (props, n_props, tris) = solid.to_mesh_f64();
        let vertices = props.chunks(n_props).map(|p| [p[0], p[1], p[2]]).collect();
        let triangles = tris
            .chunks(3)
            .map(|t| {
                [
                    usize::try_from(t[0]).unwrap(),
                    usize::try_from(t[1]).unwrap(),
                    usize::try_from(t[2]).unwrap(),
                ]
            })
            .collect();
        Ok(Self {
            vertices,
            triangles,
        })
    }

    /// Every edge is shared by exactly two triangles, in opposite directions:
    /// a closed, consistently wound surface a slicer can trust.
    pub fn is_closed(&self) -> bool {
        let mut edges: HashMap<(usize, usize), i32> = HashMap::new();
        for t in &self.triangles {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                *edges.entry((a, b)).or_default() += 1;
            }
        }
        edges
            .iter()
            .all(|(&(a, b), &n)| n == 1 && edges.get(&(b, a)) == Some(&1))
    }

    pub fn bounds(&self) -> ([f64; 3], [f64; 3]) {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for v in &self.vertices {
            for i in 0..3 {
                lo[i] = lo[i].min(v[i]);
                hi[i] = hi[i].max(v[i]);
            }
        }
        (lo, hi)
    }

    /// Volume (mm3) and centroid, from signed tetrahedra to the origin.
    pub fn volume_and_centroid(&self) -> (f64, [f64; 3]) {
        let mut vol = 0.0;
        let mut c = [0.0; 3];
        for t in &self.triangles {
            let (a, b, d) = (
                self.vertices[t[0]],
                self.vertices[t[1]],
                self.vertices[t[2]],
            );
            let v = (a[0] * (b[1] * d[2] - b[2] * d[1]) - a[1] * (b[0] * d[2] - b[2] * d[0])
                + a[2] * (b[0] * d[1] - b[1] * d[0]))
                / 6.0;
            vol += v;
            for i in 0..3 {
                c[i] += v * (a[i] + b[i] + d[i]) / 4.0;
            }
        }
        (vol, c.map(|x| x / vol))
    }

    /// Area (mm2) of surface facing down steeper than `max_angle_deg` from vertical,
    /// higher than `skip_mm` above the bed: what would need support when printed
    /// in this orientation.
    pub fn overhang_area(&self, max_angle_deg: f64, skip_mm: f64) -> f64 {
        let (lo, _) = self.bounds();
        let limit = -(max_angle_deg.to_radians().cos());
        let mut area = 0.0;
        for t in &self.triangles {
            let (a, b, d) = (
                self.vertices[t[0]],
                self.vertices[t[1]],
                self.vertices[t[2]],
            );
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
            let n = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len == 0.0 {
                continue;
            }
            let zc = (a[2] + b[2] + d[2]) / 3.0;
            if n[2] / len < limit && zc > lo[2] + skip_mm {
                area += len / 2.0;
            }
        }
        area
    }

    /// Move so the lowest point is at z = 0.
    pub fn on_bed(mut self) -> Self {
        let (lo, _) = self.bounds();
        for v in &mut self.vertices {
            v[2] -= lo[2];
        }
        self
    }

    pub fn translated(mut self, by: [f64; 3]) -> Self {
        for v in &mut self.vertices {
            for i in 0..3 {
                v[i] += by[i];
            }
        }
        self
    }

    /// Binary STL. The header is fixed so output is byte-for-byte reproducible.
    pub fn write_stl(&self, mut out: impl Write, name: &str) -> io::Result<()> {
        let mut header = [b' '; 80];
        let text = format!("CubeSat Classroom Rev C {name}");
        let n = text.len().min(80);
        header[..n].copy_from_slice(&text.as_bytes()[..n]);
        out.write_all(&header)?;
        out.write_all(&u32::try_from(self.triangles.len()).unwrap().to_le_bytes())?;
        for t in &self.triangles {
            let (a, b, d) = (
                self.vertices[t[0]],
                self.vertices[t[1]],
                self.vertices[t[2]],
            );
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let w = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
            let mut n = [
                u[1] * w[2] - u[2] * w[1],
                u[2] * w[0] - u[0] * w[2],
                u[0] * w[1] - u[1] * w[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 0.0 {
                n = n.map(|x| x / len);
            }
            for v in [n, a, b, d] {
                for x in v {
                    #[allow(clippy::cast_possible_truncation)]
                    out.write_all(&(x as f32).to_le_bytes())?;
                }
            }
            out.write_all(&[0, 0])?;
        }
        Ok(())
    }
}

/// A 3MF package with several named objects, already placed on the plate.
pub fn write_3mf(path: &std::path::Path, objects: &[(String, Mesh)]) -> io::Result<()> {
    use std::fmt::Write as _;
    let mut model = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xml:lang=\"en-US\" \
         xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n<resources>\n",
    );
    for (i, (name, mesh)) in objects.iter().enumerate() {
        let _ = writeln!(
            model,
            "<object id=\"{}\" type=\"model\" name=\"{name}\"><mesh><vertices>",
            i + 1
        );
        for v in &mesh.vertices {
            let _ = writeln!(
                model,
                "<vertex x=\"{:.5}\" y=\"{:.5}\" z=\"{:.5}\"/>",
                v[0], v[1], v[2]
            );
        }
        model.push_str("</vertices><triangles>\n");
        for t in &mesh.triangles {
            let _ = writeln!(
                model,
                "<triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>",
                t[0], t[1], t[2]
            );
        }
        model.push_str("</triangles></mesh></object>\n");
    }
    model.push_str("</resources>\n<build>\n");
    for i in 0..objects.len() {
        let _ = writeln!(model, "<item objectid=\"{}\"/>", i + 1);
    }
    model.push_str("</build>\n</model>\n");

    let files = [
        (
            "[Content_Types].xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
             <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
             <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/></Types>\n"
                .to_string(),
        ),
        (
            "_rels/.rels",
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/></Relationships>\n"
                .to_string(),
        ),
        ("3D/3dmodel.model", model),
    ];
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path)?);
    // A fixed timestamp keeps the package reproducible.
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, content) in files {
        zip.start_file(name, options).map_err(io::Error::other)?;
        zip.write_all(content.as_bytes())?;
    }
    zip.finish().map_err(io::Error::other)?;
    Ok(())
}
