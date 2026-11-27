//! `amsat-kit`: build, check and export the CubeSat Classroom cases: the classroom kit on the
//! AMSAT CubeSatSim frame, the outdoor ground station (Uno and Nano), and the outdoor XIAO station.
//!
//! ```sh
//! cargo run --release -p amsat-kit              # check, then write each case's stl/ and p1s/
//! cargo run --release -p amsat-kit -- check     # checks only
//! cargo run --release -p amsat-kit -- self-test # every broken design must fail the checks
//! ```

use std::path::Path;
use std::process::ExitCode;

use amsat_kit::checks::{boundary_designs, broken_designs, verify};
use amsat_kit::design::Design;
use amsat_kit::geom::{Mesh, closed_solid};
use amsat_kit::joints::{contact_r, finger_strain, hook_preload_force, hook_strain, pin_retention};
use amsat_kit::outdoor::{self, Outdoor};
use amsat_kit::parts::{self, kit_dir};
use amsat_kit::{bambu, drawing};
use amsat_kit::{battery, outdoor_checks, xiao};
use manifold_csg::Manifold;

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

/// Write each solid as an STL standing on z = 0, and report it.
fn write_meshes(
    dir: &Path,
    solids: Vec<(&'static str, Manifold)>,
) -> Result<Vec<(&'static str, Mesh)>, String> {
    let stl = dir.join("stl");
    std::fs::create_dir_all(&stl).map_err(|e| e.to_string())?;
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
    Ok(meshes)
}

fn pin_coupons(d: &Design) -> Vec<(&'static str, Manifold)> {
    let tab = |i: usize| {
        let (fit, dots) = parts::FIT_LADDER[i];
        parts::coupon_pin_tab(d, fit, dots, None)
    };
    vec![
        ("coupon-pin-base", parts::coupon_pin_base(d)),
        ("coupon-pin-tab", tab(0)),
        ("coupon-pin-tab-2-dots", tab(1)),
        ("coupon-pin-tab-3-dots", tab(2)),
    ]
}

/// The classroom kit: the AMSAT frame with the payload plate and the battery plate.
fn build_classroom(d: &Design) -> Result<(), String> {
    let dir = kit_dir();
    println!("== classroom kit ({})", dir.display());
    verify(d)?;
    report(d);
    let mut solids = vec![
        // The end piece prints face down, so its columns point up. Print two: one is the top.
        (
            "bottom-with-columns",
            parts::bottom_with_columns(d)?.rotate(180.0, 0.0, 0.0),
        ),
        ("payload-plate", parts::payload_plate(d, None, 0.0)?),
        ("battery-plate", battery::battery_plate(d, None, 0.0)?),
        ("fit-plate", parts::fit_plate(d)?),
    ];
    solids.extend(pin_coupons(d));
    solids.push(("coupon-nano-base", parts::coupon_nano_base(d, 0.0)));
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::CLASSROOM, &meshes)?;
    drawing::write_all(d, &dir)?;
    Ok(())
}

/// The parts both outdoor cases share: box, lid, gasket and the tripod-thread coupon.
fn outdoor_parts(d: &Design, o: &Outdoor) -> Result<Vec<(&'static str, Manifold)>, String> {
    Ok(vec![
        ("outdoor-box", outdoor::body(d, o, o.thread_clear)),
        ("outdoor-lid", outdoor::lid(d, o)?),
        ("outdoor-gasket", outdoor::gasket(o)),
        ("coupon-tripod-thread", outdoor::thread_coupon(o)),
    ])
}

fn report_outdoor(d: &Design, o: &Outdoor) {
    println!(
        "box: {:.0} x {:.0} mm outside the rim, {:.0} mm tall with the lid; gasket squeezed {:.0}%; \
         plate {:.1} mm over the back; {} ports",
        o.cavity + 2.0 * (o.wall + o.flange),
        o.cavity + 2.0 * (o.wall + o.flange),
        o.top + o.lid_t,
        outdoor::gasket_squeeze(o) * 100.0,
        o.plate_z(d),
        o.ports.len()
    );
}

/// The outdoor ground station for the Uno and Nano.
fn build_outdoor(d: &Design) -> Result<(), String> {
    let o = Outdoor::default();
    let dir = kit_dir().with_file_name("outdoor-station");
    println!("== outdoor station ({})", dir.display());
    outdoor_checks::verify_uno_nano(d, &o)?;
    println!("checks passed");
    report_outdoor(d, &o);
    let mut solids = outdoor_parts(d, &o)?;
    solids.push(("payload-plate", parts::payload_plate(d, None, 0.0)?));
    solids.extend(pin_coupons(d));
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::OUTDOOR, &meshes)
}

/// The outdoor XIAO station.
fn build_xiao(d: &Design) -> Result<(), String> {
    let o = Outdoor::xiao();
    let dir = kit_dir().with_file_name("outdoor-xiao");
    println!("== outdoor XIAO station ({})", dir.display());
    outdoor_checks::verify_xiao(d, &o)?;
    println!("checks passed");
    report_outdoor(d, &o);
    let mut solids = outdoor_parts(d, &o)?;
    solids.push(("xiao-plate", xiao::xiao_plate(d, None, 0.0)));
    solids.push(("coupon-xiao", xiao::coupon(d, 0.0)));
    solids.extend(pin_coupons(d));
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::OUTDOOR_XIAO, &meshes)
}

fn check_all(d: &Design) -> Result<(), String> {
    verify(d)?;
    report(d);
    outdoor_checks::verify_uno_nano(d, &Outdoor::default())?;
    outdoor_checks::verify_xiao(d, &Outdoor::xiao())?;
    println!("outdoor cases: checks passed");
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
    for (label, d, o, xiao) in outdoor_checks::broken_designs() {
        let result = if xiao {
            outdoor_checks::verify_xiao(&d, &o)
        } else {
            outdoor_checks::verify_uno_nano(&d, &o)
        };
        match result {
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
        None | Some("build") => build_classroom(&d)
            .and_then(|()| build_outdoor(&d))
            .and_then(|()| build_xiao(&d)),
        Some("check") => check_all(&d),
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
