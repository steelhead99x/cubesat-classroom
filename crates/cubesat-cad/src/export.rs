//! Write STL files, P1S plates (3MF) and the compliance report.

use std::fs;
use std::io::{self, BufWriter};
use std::path::Path;

use crate::check::{BED, Report, print_parts};
use crate::design::Design;
use crate::mesh::{Mesh, write_3mf};

/// Plates in print order, each a list of (object name, part name).
pub const PLATES: &[(&str, &[(&str, &str)])] = &[
    (
        "00-snap-test",
        &[
            ("snap-test-peg", "snap-test-peg"),
            ("snap-test-wall", "snap-test-wall"),
        ],
    ),
    (
        "01-side-panels",
        &[
            ("side-panel-a", "side-panel"),
            ("side-panel-b", "side-panel"),
        ],
    ),
    (
        "02-end-plates",
        &[
            ("end-plate-bottom", "end-plate-bottom"),
            ("end-plate-top", "end-plate-top"),
        ],
    ),
    (
        "03-payload",
        &[
            ("payload-plate", "payload-plate"),
            ("nano-clamp", "nano-clamp"),
        ],
    ),
    (
        "04-fasteners",
        &[
            ("lock-key-1", "lock-key"),
            ("lock-key-2", "lock-key"),
            ("lock-key-3", "lock-key"),
            ("lock-key-4", "lock-key"),
            ("plate-stud-1", "qt-stud-plate"),
            ("plate-stud-2", "qt-stud-plate"),
            ("plate-stud-3", "qt-stud-plate"),
            ("plate-stud-4", "qt-stud-plate"),
            ("clamp-stud-1", "qt-stud-clamp"),
            ("clamp-stud-2", "qt-stud-clamp"),
            ("washer-1", "qt-washer"),
            ("washer-2", "qt-washer"),
            ("washer-3", "qt-washer"),
            ("washer-4", "qt-washer"),
            ("washer-5", "qt-washer"),
            ("washer-6", "qt-washer"),
        ],
    ),
];
const GAP: f64 = 10.0;
const MARGIN: f64 = 5.0;

/// Write `stl/`, `p1s/` and `cds-report.md` under `out`.
pub fn write_all(d: &Design, report: &Report, out: &Path) -> io::Result<()> {
    let stl_dir = out.join("stl");
    let p1s_dir = out.join("p1s");
    fs::create_dir_all(&stl_dir)?;
    fs::create_dir_all(&p1s_dir)?;

    let mut meshes = Vec::new();
    for (name, part) in print_parts(d) {
        let mesh = Mesh::from_solid(name, &part)
            .map_err(io::Error::other)?
            .on_bed();
        mesh.write_stl(
            BufWriter::new(fs::File::create(stl_dir.join(format!("{name}.stl")))?),
            name,
        )?;
        meshes.push((name, mesh));
    }

    for (plate, objects) in PLATES {
        let picked: Vec<(String, Mesh)> = objects
            .iter()
            .map(|(object, part)| {
                let mesh = meshes
                    .iter()
                    .find(|(n, _)| n == part)
                    .expect("known part")
                    .1
                    .clone();
                ((*object).to_string(), mesh)
            })
            .collect();
        let placed = arrange(plate, picked)?;
        write_3mf(&p1s_dir.join(format!("{plate}.3mf")), &placed)?;
    }

    // The assembled CubeSat, for viewing: every part where it sits, in CDS coordinates.
    let assembly = crate::parts::Assembly::new(d).all(d);
    Mesh::from_solid("assembly", &assembly)
        .map_err(io::Error::other)?
        .write_stl(
            BufWriter::new(fs::File::create(out.join("assembly.stl"))?),
            "assembly",
        )?;
    fs::write(out.join("cds-report.md"), report_markdown(report))?;
    Ok(())
}

/// Lay parts out in rows, each row centred, the block centred on the 256 mm bed.
fn arrange(plate: &str, parts: Vec<(String, Mesh)>) -> io::Result<Vec<(String, Mesh)>> {
    let size = |m: &Mesh| {
        let (lo, hi) = m.bounds();
        (hi[0] - lo[0], hi[1] - lo[1])
    };
    // Fill rows left to right, starting a new row when the next part won't fit.
    let mut rows: Vec<Vec<(String, Mesh)>> = vec![Vec::new()];
    let mut width = 0.0;
    for (name, mesh) in parts {
        let (w, _) = size(&mesh);
        let row = rows.last_mut().expect("at least one row");
        let next = if row.is_empty() { w } else { width + GAP + w };
        if !row.is_empty() && next > BED - 2.0 * MARGIN {
            rows.push(vec![(name, mesh)]);
            width = w;
        } else {
            row.push((name, mesh));
            width = next;
        }
    }
    let depths: Vec<f64> = rows
        .iter()
        .map(|r| r.iter().map(|(_, m)| size(m).1).fold(0.0, f64::max))
        .collect();
    let total_depth = depths.iter().sum::<f64>() + GAP * (depths.len() as f64 - 1.0);
    let mut y = (BED + total_depth) / 2.0;
    let mut placed = Vec::new();
    for (row, depth) in rows.into_iter().zip(depths) {
        let total: f64 =
            row.iter().map(|(_, m)| size(m).0).sum::<f64>() + GAP * (row.len() as f64 - 1.0);
        let mut x = (BED - total) / 2.0;
        y -= depth;
        for (name, mesh) in row {
            let (lo, _) = mesh.bounds();
            let (w, dep) = size(&mesh);
            let moved = mesh.translated([x - lo[0], y + (depth - dep) / 2.0 - lo[1], -lo[2]]);
            let (mlo, mhi) = moved.bounds();
            if mlo[0] < MARGIN || mlo[1] < MARGIN || mhi[0] > BED - MARGIN || mhi[1] > BED - MARGIN
            {
                return Err(io::Error::other(format!(
                    "{plate}: {name} does not fit the P1S plate"
                )));
            }
            placed.push((name, moved));
            x += w + GAP;
        }
        y -= GAP;
    }
    Ok(placed)
}

fn report_markdown(report: &Report) -> String {
    format!(
        "# Rev C compliance report\n\n\
         Generated by `cargo run -p cubesat-cad --release -- build hardware/rev-c`. Do not edit by hand.\n\n\
         Checked against the CubeSat Design Specification (CDS) Rev 14.1 (Cal Poly, 2022-02-09), \
         with the processes in NASA's CubeSat 101 (October 2017). \"Deviation\" marks a requirement a printed \
         classroom frame does not meet; a flight version would need a different material or part, or the launch \
         provider's approval. These are CAD checks: measure the printed parts before relying on them.\n\n{}",
        report.markdown()
    )
}
