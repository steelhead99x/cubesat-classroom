//! Small geometry helpers on top of manifold-csg, and mesh input and output. Dimensions in mm.

use std::fs;
use std::io;
use std::path::Path;

use manifold_csg::{CrossSection, JoinType, Manifold};

pub const FACETS: i32 = 48;
pub const EPS: f64 = 0.05;

/// Cylinder of diameter `d` (or tapering to `d2`) standing on (x, y, z).
pub fn cyl(d: f64, h: f64, x: f64, y: f64, z: f64) -> Manifold {
    cone(d, d, h, x, y, z)
}

pub fn cone(d: f64, d2: f64, h: f64, x: f64, y: f64, z: f64) -> Manifold {
    Manifold::cylinder(h, d / 2.0, d2 / 2.0, FACETS, false).translate(x, y, z)
}

/// Axis-aligned box from two corners; empty if a side has no length.
pub fn bx(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64) -> Manifold {
    if x1 <= x0 || y1 <= y0 || z1 <= z0 {
        return Manifold::empty();
    }
    Manifold::cube(x1 - x0, y1 - y0, z1 - z0, false).translate(x0, y0, z0)
}

/// Polygon with counter-clockwise winding, whichever way the points were listed.
pub fn poly(points: &[[f64; 2]]) -> CrossSection {
    let n = points.len();
    let area: f64 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    let pts: Vec<[f64; 2]> = if area > 0.0 {
        points.to_vec()
    } else {
        points.iter().rev().copied().collect()
    };
    CrossSection::from_polygons(&[pts])
}

pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> CrossSection {
    poly(&[[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
}

/// Extrude a (u, z) profile across v = -width/2 .. width/2.
pub fn uz_prism(points: &[[f64; 2]], width: f64) -> Manifold {
    poly(points)
        .extrude(width)
        .rotate(90.0, 0.0, 0.0)
        .translate(0.0, width / 2.0, 0.0)
}

/// Ring sector between radii r0 and r1, angles a0 to a1 in degrees.
pub fn arc_band(r0: f64, r1: f64, a0: f64, a1: f64) -> CrossSection {
    let n = (((a1 - a0).abs() / 4.0) as usize).max(4);
    let at = |i: usize| (a0 + (a1 - a0) * i as f64 / n as f64).to_radians();
    let mut pts: Vec<[f64; 2]> = (0..=n)
        .map(|i| [r1 * at(i).cos(), r1 * at(i).sin()])
        .collect();
    pts.extend((0..=n).rev().map(|i| [r0 * at(i).cos(), r0 * at(i).sin()]));
    poly(&pts)
}

pub fn chamfered_square(size: f64, c: f64) -> CrossSection {
    let h = size / 2.0;
    poly(&[
        [-h + c, -h],
        [h - c, -h],
        [h, -h + c],
        [h, h - c],
        [h - c, h],
        [-h + c, h],
        [-h, h - c],
        [-h, -h + c],
    ])
}

pub fn offset(section: &CrossSection, delta: f64, segments: i32) -> CrossSection {
    section.offset(delta, JoinType::Round, 2.0, segments)
}

/// Round a cross-section's convex corners: slit ends become fillets at a finger's root.
pub fn round_corners(section: &CrossSection, r: f64) -> CrossSection {
    offset(&offset(section, -r, 16), r, 16)
}

pub fn union_all(parts: &[Manifold]) -> Manifold {
    if parts.is_empty() {
        Manifold::empty()
    } else {
        Manifold::batch_union(parts)
    }
}

pub fn overlap(a: &Manifold, b: &Manifold) -> f64 {
    a.intersection(b).volume()
}

/// Rotate about a line through `point`, parallel to the x or y axis.
pub fn rotate_about(solid: &Manifold, axis: char, angle: f64, point: [f64; 3]) -> Manifold {
    let moved = solid.translate(-point[0], -point[1], -point[2]);
    let turned = if axis == 'x' {
        moved.rotate(angle, 0.0, 0.0)
    } else {
        moved.rotate(0.0, angle, 0.0)
    };
    turned.translate(point[0], point[1], point[2])
}

pub fn bounds(solid: &Manifold) -> ([f64; 3], [f64; 3]) {
    solid
        .bounding_box()
        .map_or(([0.0; 3], [0.0; 3]), |b| (b.min(), b.max()))
}

/// A triangle mesh with shared vertices.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub verts: Vec<[f64; 3]>,
    pub tris: Vec<[u32; 3]>,
}

impl Mesh {
    pub fn from_solid(solid: &Manifold) -> Self {
        let (props, n, tris) = solid.to_mesh_f32();
        Self {
            verts: props
                .chunks(n)
                .map(|p| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
                .collect(),
            tris: tris.chunks(3).map(|t| [t[0], t[1], t[2]]).collect(),
        }
    }

    pub fn bounds(&self) -> ([f64; 3], [f64; 3]) {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for v in &self.verts {
            for i in 0..3 {
                lo[i] = lo[i].min(v[i]);
                hi[i] = hi[i].max(v[i]);
            }
        }
        (lo, hi)
    }

    pub fn extents(&self) -> [f64; 3] {
        let (lo, hi) = self.bounds();
        [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]]
    }

    pub fn translated(&self, d: [f64; 3]) -> Self {
        Self {
            verts: self
                .verts
                .iter()
                .map(|v| [v[0] + d[0], v[1] + d[1], v[2] + d[2]])
                .collect(),
            tris: self.tris.clone(),
        }
    }

    pub fn normal(&self, t: usize) -> [f64; 3] {
        let [a, b, c] = self.tris[t].map(|i| self.verts[i as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 0.0 {
            [n[0] / len, n[1] / len, n[2] / len]
        } else {
            [0.0; 3]
        }
    }

    pub fn volume(&self) -> f64 {
        self.tris
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| self.verts[i as usize]);
                (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                    + a[2] * (b[0] * c[1] - b[1] * c[0]))
                    / 6.0
            })
            .sum()
    }

    /// Every edge shared by exactly two triangles, in opposite directions, and positive volume.
    pub fn is_closed(&self) -> bool {
        let mut edges = std::collections::HashMap::new();
        for t in &self.tris {
            for k in 0..3 {
                *edges.entry((t[k], t[(k + 1) % 3])).or_insert(0) += 1;
            }
        }
        edges
            .iter()
            .all(|(&(a, b), &n)| n == 1 && edges.get(&(b, a)) == Some(&1))
            && self.volume() > 0.0
    }

    /// Binary STL with a fixed header, so the file is the same on every run.
    pub fn write_stl(&self, path: &Path, name: &str) -> io::Result<()> {
        let mut out = Vec::with_capacity(84 + 50 * self.tris.len());
        let mut header = format!("CubeSat Classroom {name}").into_bytes();
        header.resize(80, b' ');
        out.extend_from_slice(&header);
        out.extend_from_slice(
            &u32::try_from(self.tris.len())
                .unwrap_or(u32::MAX)
                .to_le_bytes(),
        );
        for (t, tri) in self.tris.iter().enumerate() {
            for v in self.normal(t) {
                out.extend_from_slice(&(v as f32).to_le_bytes());
            }
            for i in tri {
                for v in self.verts[*i as usize] {
                    out.extend_from_slice(&(v as f32).to_le_bytes());
                }
            }
            out.extend_from_slice(&[0, 0]);
        }
        fs::write(path, out)
    }

    /// Read a binary STL, merging vertices that are exactly equal.
    pub fn read_stl(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        let bad = || io::Error::other(format!("{}: not a binary STL", path.display()));
        let count = u32::from_le_bytes(
            data.get(80..84)
                .ok_or_else(bad)?
                .try_into()
                .map_err(|_| bad())?,
        ) as usize;
        if data.len() < 84 + 50 * count {
            return Err(bad());
        }
        let mut index = std::collections::HashMap::new();
        let mut mesh = Self {
            verts: Vec::new(),
            tris: Vec::with_capacity(count),
        };
        for t in 0..count {
            let base = 84 + 50 * t + 12;
            let mut tri = [0u32; 3];
            for (k, slot) in tri.iter_mut().enumerate() {
                let at = base + 12 * k;
                let v: [f32; 3] = std::array::from_fn(|i| {
                    f32::from_le_bytes(
                        data[at + 4 * i..at + 4 * i + 4]
                            .try_into()
                            .expect("4 bytes"),
                    )
                });
                let key = v.map(f32::to_bits);
                let next = u32::try_from(mesh.verts.len()).unwrap_or(u32::MAX);
                *slot = *index.entry(key).or_insert_with(|| {
                    mesh.verts.push(v.map(f64::from));
                    next
                });
            }
            mesh.tris.push(tri);
        }
        Ok(mesh)
    }

    pub fn to_solid(&self) -> Manifold {
        let props: Vec<f32> = self
            .verts
            .iter()
            .flat_map(|v| v.map(|x| x as f32))
            .collect();
        let tris: Vec<u32> = self.tris.iter().flatten().copied().collect();
        Manifold::from_mesh_f32(&props, 3, &tris).unwrap_or_else(|_| Manifold::empty())
    }
}

/// The solid as a mesh, or an error if it isn't one clean, closed piece.
pub fn closed_solid(name: &str, solid: &Manifold) -> Result<Mesh, String> {
    solid
        .status()
        .map_err(|e| format!("{name}: CAD error {e:?}"))?;
    let pieces = solid.decompose().len();
    if pieces != 1 {
        return Err(format!("{name}: expected one solid, got {pieces}"));
    }
    let mesh = Mesh::from_solid(solid);
    if !mesh.is_closed() {
        return Err(format!("{name}: mesh is not a closed solid"));
    }
    Ok(mesh)
}
