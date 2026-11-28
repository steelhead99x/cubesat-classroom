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

use amsat_kit::brand::{self, Face, Spot};
use amsat_kit::camera::{self, Camera};
use amsat_kit::checks::{boundary_designs, broken_designs, verify};
use amsat_kit::design::Design;
use amsat_kit::geom::{Mesh, closed_solid};
use amsat_kit::joints::{contact_r, finger_strain, hook_preload_force, hook_strain, pin_retention};
use amsat_kit::outdoor::{self, Outdoor};
use amsat_kit::parts::{self, kit_dir};
use amsat_kit::{bambu, drawing, labels};
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
        let path = stl.join(format!("{name}.stl"));
        mesh.write_stl(&path, name).map_err(|e| e.to_string())?;
        // Check the file as a slicer reads it: f32 corners, equal ones merged.
        let written = Mesh::read_stl(&path).map_err(|e| e.to_string())?;
        if let Some(e) = written.defects() {
            return Err(format!("{name}.stl has {e}"));
        }
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

/// Every part's two-colour logo: (source, body, logo), for the Bambu projects. A raised logo
/// is added to the part's solid too, so its STL is the same part as the two-colour project.
fn brands(
    d: &Design,
    o: Option<&Outdoor>,
    kit: &bambu::Kit,
    solids: &mut [(&'static str, Manifold)],
) -> Result<Vec<(&'static str, Mesh, Mesh)>, String> {
    // Coupon label tiles first, so the logo search sees each coupon as it prints.
    let tiles = labels::apply(d, solids);
    let mut out: Vec<(&'static str, Mesh, Mesh)> = Vec::new();
    for p in kit.parts {
        if out.iter().any(|(n, ..)| *n == p.source) {
            continue;
        }
        let solid = match p.source.strip_prefix("upstream/") {
            Some(file) => parts::upstream(file)?.to_solid(),
            None => solids
                .iter()
                .find(|(n, _)| *n == p.source)
                .map(|(_, s)| s.clone())
                .ok_or_else(|| format!("{} wasn't built", p.source))?,
        };
        let tile = tiles.iter().find(|(n, ..)| *n == p.source);
        let spot = match (tile, p.source) {
            (Some((_, art, z)), _) => Spot::Label(art.clone(), *z),
            (None, source) => match source {
                "payload-plate" | "fit-plate" => Spot::Fixed(parts::plate_brand(d)?, Face::Top),
                "outdoor-lid" => Spot::Fixed(
                    outdoor::lid_brand(d, o.ok_or("the lid needs its case")?)?,
                    Face::Bed,
                ),
                "battery-plate" | "xiao-plate" => Spot::Search(&[Face::Top, Face::Bed]),
                _ => Spot::Search(&[Face::Bed, Face::Top]),
            },
        };
        // What's mounted on the part, so a raised logo clears it.
        let above = match p.source {
            "payload-plate" | "fit-plate" => Some(outdoor_checks::uno_nano(d)),
            "battery-plate" => Some(battery::cell(d)),
            "xiao-plate" | "coupon-xiao" => Some(xiao::boards(d)),
            _ => None,
        };
        let b = brand::place(d, p.source, &solid, spot, above.as_ref())?;
        println!(
            "logo: {}: {} mm {} on the {} face, {}",
            p.source,
            b.width.round(),
            if b.words { "mark and words" } else { "mark" },
            match b.face {
                Face::Bed => "bed",
                Face::Top => "top",
            },
            if b.raise > 0.0 {
                format!("raised {} mm", b.raise)
            } else {
                "flush".to_string()
            }
        );
        if b.raise > 0.0 {
            let s = solids
                .iter_mut()
                .find(|(n, _)| *n == p.source)
                .ok_or_else(|| format!("{}: only a built part can have a raised logo", p.source))?;
            s.1 = s.1.union(&b.accent);
        }
        let body = closed_solid(p.source, &b.body)?;
        let accent = Mesh::from_solid(&b.accent);
        out.push((p.source, body, accent));
    }
    Ok(out)
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
    let cam = Camera::default();
    camera::verify(d, &cam)?;
    solids.push(("side-with-camera", camera::side_with_camera(d, &cam, 0.0)?));
    solids.push(("coupon-camera", camera::coupon(d, &cam)?));
    let logos = brands(d, None, &bambu::CLASSROOM, &mut solids)?;
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::CLASSROOM, &meshes, &logos)?;
    drawing::write_all(d, &dir)?;
    Ok(())
}

/// The parts both outdoor cases share: box, lid, and the tripod-thread and latch coupons.
fn outdoor_parts(d: &Design, o: &Outdoor) -> Result<Vec<(&'static str, Manifold)>, String> {
    let (latch_box, lid_piece) = outdoor::latch_coupon(d, o)?;
    Ok(vec![
        ("outdoor-box", outdoor::body(d, o, o.thread_clear)),
        ("outdoor-lid", outdoor::lid_to_print(d, o)?),
        ("coupon-tripod-thread", outdoor::thread_coupon(o)),
        ("coupon-latch-box", latch_box),
        ("coupon-latch-lid", lid_piece),
    ])
}

fn report_outdoor(d: &Design, o: &Outdoor) {
    println!(
        "box: {:.0} x {:.0} mm outside the rim, {:.0} mm tall with the lid; plate {:.1} mm over the \
         back; {} ports",
        o.cavity + 2.0 * (o.wall + o.flange),
        o.cavity + 2.0 * (o.wall + o.flange),
        o.top + o.lid_t,
        o.plate_z(d),
        o.ports.len()
    );
    let (length, id) = o.oring();
    println!(
        "seal: {} mm O-ring squeezed {:.0}%, groove {:.0}% full; groove bottom {:.0} mm round, so an \
         O-ring about {:.0} mm inside diameter; latches bend {:.2}%",
        o.oring_d,
        o.oring_squeeze() * 100.0,
        o.groove_fill() * 100.0,
        length,
        id,
        o.latch_strain() * 100.0
    );
    let c = outdoor::latch_coupon_case(o);
    let (_, coupon_id) = c.oring();
    println!(
        "latch coupon: a {:.0} mm box with two latches; optional O-ring about {:.0} mm inside diameter",
        c.cavity + 2.0 * (c.wall + c.flange),
        coupon_id
    );
}

/// The outdoor ground station for the Uno and Nano.
fn build_outdoor(d: &Design) -> Result<(), String> {
    let o = Outdoor::default();
    let dir = kit_dir().with_file_name("outdoor-station");
    println!("== outdoor station ({})", dir.display());
    outdoor_checks::verify_uno_nano(d, &o)?;
    outdoor_checks::verify_latch_coupon(d, &o)?;
    println!("checks passed");
    report_outdoor(d, &o);
    let mut solids = outdoor_parts(d, &o)?;
    solids.push(("payload-plate", parts::payload_plate(d, None, 0.0)?));
    solids.extend(pin_coupons(d));
    let logos = brands(d, Some(&o), &bambu::OUTDOOR, &mut solids)?;
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::OUTDOOR, &meshes, &logos)
}

/// The outdoor XIAO station.
fn build_xiao(d: &Design) -> Result<(), String> {
    let o = Outdoor::xiao();
    let dir = kit_dir().with_file_name("outdoor-xiao");
    println!("== outdoor XIAO station ({})", dir.display());
    outdoor_checks::verify_xiao(d, &o)?;
    outdoor_checks::verify_latch_coupon(d, &o)?;
    println!("checks passed");
    report_outdoor(d, &o);
    let mut solids = outdoor_parts(d, &o)?;
    solids.push(("xiao-plate", xiao::xiao_plate(d, None, 0.0)));
    solids.push(("coupon-xiao", xiao::coupon(d, 0.0)));
    solids.extend(pin_coupons(d));
    let logos = brands(d, Some(&o), &bambu::OUTDOOR_XIAO, &mut solids)?;
    let meshes = write_meshes(&dir, solids)?;
    bambu::write_projects(&dir, &bambu::OUTDOOR_XIAO, &meshes, &logos)
}

fn check_all(d: &Design) -> Result<(), String> {
    verify(d)?;
    camera::verify(d, &Camera::default())?;
    report(d);
    outdoor_checks::verify_uno_nano(d, &Outdoor::default())?;
    outdoor_checks::verify_latch_coupon(d, &Outdoor::default())?;
    outdoor_checks::verify_xiao(d, &Outdoor::xiao())?;
    outdoor_checks::verify_latch_coupon(d, &Outdoor::xiao())?;
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
    let d0 = Design::default();
    for (label, result) in brand::broken_designs(&d0)? {
        match result {
            Ok(_) => missed.push(label),
            Err(e) => println!("rejected {label}: {e}"),
        }
    }
    for (label, c) in camera::broken_designs() {
        match camera::verify(&d0, &c) {
            Ok(()) => missed.push(label),
            Err(e) => println!("rejected {label}: {e}"),
        }
    }
    for (label, o) in outdoor_checks::broken_latch_coupons() {
        match outdoor_checks::verify_latch_coupon(&d0, &o) {
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

fn build_xiao_retention(d: &Design, dir: &std::path::Path) -> Result<(), String> {
    let o = Outdoor::xiao();
    outdoor_checks::verify_xiao(d, &o)?;
    let lid = outdoor_checks::xiao_retention_lid(d, &o)?;
    write_meshes(
        dir,
        vec![("xiao-retention-lid", lid.rotate(180.0, 0.0, 0.0))],
    )?;
    Ok(())
}

fn raised_test_label(
    d: &Design,
    solid: &Manifold,
    name: &'static str,
    lines: &'static [&'static str],
) -> Result<Manifold, String> {
    let label = labels::Label {
        part: name,
        lines,
        side: labels::Side::MinusY,
        thick: None,
    };
    let (body, art, z) = labels::add(d, solid, &label);
    let b = brand::place(d, name, &body, Spot::Label(art, z), None)?;
    Ok(body.union(&b.accent))
}

fn build_outdoor_battery_fit(d: &Design, dir: &std::path::Path) -> Result<(), String> {
    verify(d)?;
    let uno = Outdoor::default();
    let xiao_case = Outdoor::xiao();
    outdoor_checks::verify_uno_nano(d, &uno)?;
    outdoor_checks::verify_xiao(d, &xiao_case)?;
    outdoor_checks::verify_service_plate(d, &uno)?;
    let station_plate = outdoor::service_plate(d, None)?;
    let xiao_plate = outdoor::battery_lead_plate(d, &xiao::xiao_plate(d, None, 0.0));
    let pocket = outdoor::battery_pocket_coupon(d, &uno);
    if (outdoor::battery_pocket_coupon(d, &xiao_case).volume() - pocket.volume()).abs() > 1e-6 {
        return Err("the outdoor cases no longer share the same battery pocket".to_owned());
    }
    for (name, o, plate, boards) in [
        (
            "outdoor Uno/Nano",
            &uno,
            &station_plate,
            outdoor_checks::uno_nano(d),
        ),
        ("HaLow XIAO", &xiao_case, &xiao_plate, xiao::boards(d)),
    ] {
        outdoor_checks::verify_battery_lead(d, o, plate, &boards)
            .map_err(|e| format!("{name}: {e}"))?;
    }
    let pocket = raised_test_label(
        d,
        &pocket,
        "stock-pocket",
        &["STOCK POCKET", "CHECK MOVEMENT"],
    )?;
    let snap = raised_test_label(
        d,
        &battery::battery_plate(d, None, 0.0)?,
        "snap-fit",
        &["BATTERY SNAP", "CHECK BOTH ENDS"],
    )?;
    write_meshes(
        dir,
        vec![
            ("outdoor-battery-stock-pocket", pocket),
            ("outdoor-battery-snap-fit", snap),
            ("outdoor-station-battery-lead-plate", station_plate),
            ("halow-xiao-battery-lead-plate", xiao_plate),
        ],
    )?;
    Ok(())
}

fn build_glove_seal(d: &Design, dir: &std::path::Path) -> Result<(), String> {
    let o = outdoor::glove_seal_case();
    let cup = outdoor::latch_coupon_box(&o);
    outdoor_checks::rules(&o)?;
    outdoor_checks::lid_latches(&o, &cup, &|f| outdoor::lid_in_place(d, &o, f))?;
    let cup = raised_test_label(
        d,
        &cup,
        "glove-seal",
        &["CUBESAT CLASSROOM", "GLOVE LATCH", "SEAL TEST"],
    )?;
    let (_, ring_id) = o.oring();
    println!("coupon O-ring: 2.5 mm section, approximate ID {ring_id:.1} mm");
    write_meshes(
        dir,
        vec![
            ("outdoor-glove-seal-cup", cup),
            ("outdoor-glove-seal-lid", outdoor::lid_to_print(d, &o)?),
        ],
    )?;
    Ok(())
}

fn build_outdoor_antenna_fit(d: &Design, dir: &std::path::Path) -> Result<(), String> {
    let wall = Outdoor::default().wall;
    let mut parts = Vec::new();
    for (name, lines, diameters) in [
        (
            "outdoor-standard-sma-fit",
            &["SMA BULKHEAD", "HOLES IN MM"][..],
            &outdoor::SMA_FIT_DIAMETERS[..],
        ),
        (
            "outdoor-large-alfa-fit",
            &["LARGE ANTENNA", "HOLES IN MM"][..],
            &outdoor::ALFA_FIT_DIAMETERS[..],
        ),
    ] {
        let plate = outdoor::antenna_fit_plate(diameters, wall);
        let (body, mut art, z) = labels::add(
            d,
            &plate,
            &labels::Label {
                part: name,
                lines,
                side: labels::Side::MinusY,
                thick: Some(wall),
            },
        );
        for (x, dia) in outdoor::antenna_fit_positions(diameters) {
            art = art.union(&amsat_kit::text::line(&dia.to_string(), 5.2).translate(x, -16.2));
        }
        let b = brand::place(d, name, &body, Spot::Label(art, z), None)?;
        parts.push((name, body.union(&b.accent)));
    }
    write_meshes(dir, parts)?;
    Ok(())
}

fn main() -> ExitCode {
    let d = Design::default();
    let result = match std::env::args().nth(1).as_deref() {
        None | Some("build") => build_classroom(&d)
            .and_then(|()| build_outdoor(&d))
            .and_then(|()| build_xiao(&d))
            .and_then(|()| {
                build_outdoor_antenna_fit(&d, &kit_dir().with_file_name("outdoor-antenna-fit"))
            }),
        Some("check") => check_all(&d),
        Some("self-test") => self_test(),
        Some("xiao-retention") => match std::env::args_os().nth(2) {
            Some(dir) => build_xiao_retention(&d, std::path::Path::new(&dir)),
            None => Err("xiao-retention needs an output directory".to_owned()),
        },
        Some("outdoor-battery-fit") => match std::env::args_os().nth(2) {
            Some(dir) => build_outdoor_battery_fit(&d, std::path::Path::new(&dir)),
            None => Err("outdoor-battery-fit needs an output directory".to_owned()),
        },
        Some("outdoor-glove-seal") => match std::env::args_os().nth(2) {
            Some(dir) => build_glove_seal(&d, std::path::Path::new(&dir)),
            None => Err("outdoor-glove-seal needs an output directory".to_owned()),
        },
        Some("outdoor-antenna-fit") => match std::env::args_os().nth(2) {
            Some(dir) => build_outdoor_antenna_fit(&d, std::path::Path::new(&dir)),
            None => Err("outdoor-antenna-fit needs an output directory".to_owned()),
        },
        Some(other) => Err(format!(
            "unknown command {other}: use build, check, self-test, xiao-retention, outdoor-battery-fit, outdoor-glove-seal or outdoor-antenna-fit"
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
