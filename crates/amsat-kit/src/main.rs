//! `amsat-kit`: build, check and export the AMSAT CubeSatSim payload kit.
//!
//! ```sh
//! cargo run --release -p amsat-kit              # check, then write stl/, p1s/ and the drawings
//! cargo run --release -p amsat-kit -- check     # checks only
//! cargo run --release -p amsat-kit -- self-test # every broken design must fail the checks
//! ```

use std::path::Path;
use std::process::ExitCode;

use amsat_kit::checks::{boundary_designs, broken_designs, verify};
use amsat_kit::design::Design;
use amsat_kit::geom::closed_solid;
use amsat_kit::joints::{contact_r, finger_strain, hook_preload_force, hook_strain, pin_retention};
use amsat_kit::parts::{self, kit_dir};
use amsat_kit::{bambu, drawing};

fn report(d: &Design) {
    println!("checks passed");
    println!(
        "snap pins: finger strain {:.2}% (limit {:.1}%), preload {:.2} mm, {:.0} N to pull the plate off \
         (payload at {:.0} g: {:.0} N)",
        finger_strain(d) * 100.0,
        d.strain_limit * 100.0,
        contact_r(d) - d.tooth_r,
        d.snap_columns.len() as f64 * pin_retention(d, d.plate_t),
        d.design_g,
        d.payload_load()
    );
    println!(
        "board hooks: finger strain {:.2}%, reach {} mm, underside {:.0} degrees (friction angle {:.0}), \
         about {:.1} N down each",
        hook_strain(d) * 100.0,
        d.hook_reach,
        d.hook_angle,
        d.friction_angle(),
        hook_preload_force(d)
    );
}

fn build(d: &Design, dir: &Path) -> Result<(), String> {
    verify(d)?;
    report(d);
    let stl = dir.join("stl");
    std::fs::create_dir_all(&stl).map_err(|e| e.to_string())?;
    let tab = |i: usize| {
        let (fit, dots) = parts::FIT_LADDER[i];
        parts::coupon_pin_tab(d, fit, dots, None)
    };
    let solids = [
        // The end piece prints face down, so its columns point up.
        (
            "bottom-with-columns",
            parts::bottom_with_columns(d)?.rotate(180.0, 0.0, 0.0),
        ),
        ("payload-plate", parts::payload_plate(d, None, 0.0)?),
        ("fit-plate", parts::fit_plate(d)?),
        ("coupon-pin-base", parts::coupon_pin_base(d)),
        ("coupon-pin-tab", tab(0)),
        ("coupon-pin-tab-2-dots", tab(1)),
        ("coupon-pin-tab-3-dots", tab(2)),
        ("coupon-nano-base", parts::coupon_nano_base(d, 0.0)),
    ];
    let mut meshes = Vec::new();
    for (name, solid) in solids {
        let mesh = closed_solid(name, &solid)?;
        let (lo, _) = mesh.bounds();
        let mesh = mesh.translated([0.0, 0.0, -lo[2]]);
        mesh.write_stl(&stl.join(format!("{name}.stl")), name)
            .map_err(|e| e.to_string())?;
        let e = mesh.extents();
        println!(
            "{name}: [{:.1}, {:.1}, {:.1}] mm, {:.1} cm3, closed, {} triangles",
            e[0],
            e[1],
            e[2],
            mesh.volume() / 1000.0,
            mesh.tris.len()
        );
        meshes.push((name, mesh));
    }
    bambu::write_projects(dir, &meshes)?;
    drawing::write_all(d, dir)?;
    Ok(())
}

fn self_test() -> Result<(), String> {
    for (label, d) in boundary_designs() {
        verify(&d)
            .map_err(|e| format!("the checks rejected a design at the limit, {label}: {e}"))?;
        println!("accepted {label}");
    }
    let mut missed = Vec::new();
    for (label, d) in broken_designs() {
        match verify(&d) {
            Ok(()) => missed.push(label),
            Err(e) => println!("rejected {label}: {e}"),
        }
    }
    if missed.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the checks accepted broken designs: {}",
            missed.join(", ")
        ))
    }
}

fn main() -> ExitCode {
    let d = Design::default();
    let result = match std::env::args().nth(1).as_deref() {
        None | Some("build") => build(&d, &kit_dir()),
        Some("check") => verify(&d).map(|()| report(&d)),
        Some("self-test") => self_test(),
        Some(other) => Err(format!(
            "unknown command {other}: use build, check or self-test"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
