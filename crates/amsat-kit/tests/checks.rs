//! The design checks pass, and they can fail: every broken design must be rejected, and every
//! design exactly at a limit must be accepted. These are design checks, not measurements of prints.

use amsat_kit::checks::{boundary_designs, broken_designs, verify};
use amsat_kit::design::Design;

#[test]
fn the_kit_passes() {
    verify(&Design::default()).unwrap();
}

/// Review finding: the kit's fit moved the coupon teeth, so at a 0.2 or 0.3 mm kit fit the 1-dot
/// tab's fingers bent 1.33% and 1.47%, over the 1.2% limit. The coupons must not depend on the
/// kit fit, and every tab must stay within the strain limit at every ladder setting.
#[test]
fn coupons_are_the_same_at_every_kit_fit() {
    use amsat_kit::geom::bounds;
    use amsat_kit::joints::finger_strain_at;
    use amsat_kit::parts::{FIT_LADDER, coupon_pin_tab, ladder_tooth_r};
    let base = Design::default();
    for (kit_fit, _) in FIT_LADDER {
        let d = Design {
            pin_fit: kit_fit,
            ..base.clone()
        };
        for (fit, dots) in FIT_LADDER {
            let a = coupon_pin_tab(&base, fit, dots, None);
            let b = coupon_pin_tab(&d, fit, dots, None);
            assert!(
                (a.volume() - b.volume()).abs() < 1e-6 && bounds(&a) == bounds(&b),
                "kit fit {kit_fit} mm changed the {fit} mm coupon tab"
            );
            let strain = finger_strain_at(&d, ladder_tooth_r(&d, fit));
            assert!(
                strain <= d.strain_limit + 1e-9,
                "kit fit {kit_fit} mm: the {fit} mm tab bends {:.2}%",
                strain * 100.0
            );
        }
    }
}

#[test]
fn designs_at_the_limits_pass() {
    for (label, d) in boundary_designs() {
        assert!(
            verify(&d).is_ok(),
            "rejected a design at the limit: {label}"
        );
    }
}

#[test]
fn broken_designs_fail() {
    let missed: Vec<_> = broken_designs()
        .into_iter()
        .filter(|(_, d)| verify(d).is_ok())
        .map(|(l, _)| l)
        .collect();
    assert!(missed.is_empty(), "accepted broken designs: {missed:?}");
}

use amsat_kit::outdoor::Outdoor;
use amsat_kit::outdoor_checks;

#[test]
fn the_outdoor_cases_pass() {
    let d = Design::default();
    outdoor_checks::verify_uno_nano(&d, &Outdoor::default()).unwrap();
    outdoor_checks::verify_xiao(&d, &Outdoor::xiao()).unwrap();
}

#[test]
fn xiao_restraint_lid_catches_the_plate() {
    outdoor_checks::xiao_retention_lid(&Design::default(), &Outdoor::xiao()).unwrap();
}

#[test]
fn two_snap_service_plate_fits_the_stock_outdoor_case() {
    outdoor_checks::verify_service_plate(&Design::default(), &Outdoor::default()).unwrap();
}

#[test]
fn both_outdoor_battery_passages_clear_the_plug_gauge() {
    use amsat_kit::{outdoor, parts, xiao};
    let d = Design::default();
    for (case, stock, boards) in [
        (
            Outdoor::default(),
            parts::payload_plate(&d, None, 0.0).unwrap(),
            outdoor_checks::uno_nano(&d),
        ),
        (
            Outdoor::xiao(),
            xiao::xiao_plate(&d, None, 0.0),
            xiao::boards(&d),
        ),
    ] {
        let plate = outdoor::battery_lead_plate(&d, &stock);
        outdoor_checks::verify_battery_lead(&d, &case, &plate, &boards).unwrap();
        let z = d.plate_t / 2.0;
        let blockage = outdoor::battery_lead_gauge(&d, &case)
            .slice_to_cross_section(case.plate_z(&d) + z)
            .extrude(0.5)
            .translate(0.0, 0.0, z);
        assert!(
            outdoor_checks::verify_battery_lead(&d, &case, &plate.union(&blockage), &boards)
                .is_err()
        );
    }
}

#[test]
fn glove_coupon_closes_releases_and_has_a_floor() {
    use amsat_kit::outdoor;
    let d = Design::default();
    let o = outdoor::glove_seal_case();
    outdoor_checks::rules(&o).unwrap();
    let cup = outdoor::latch_coupon_box(&o);
    assert!(
        o.cavity_outline()
            .difference(&cup.slice_to_cross_section(1.0))
            .area()
            < 1e-3
    );
    outdoor_checks::lid_latches(&o, &cup, &|f| outdoor::lid_in_place(&d, &o, f)).unwrap();
}

#[test]
fn both_outdoor_latch_coupons_pass() {
    let d = Design::default();
    outdoor_checks::verify_latch_coupon(&d, &Outdoor::default()).expect("Uno/Nano latch coupon");
    outdoor_checks::verify_latch_coupon(&d, &Outdoor::xiao()).expect("XIAO latch coupon");
}

#[test]
fn broken_outdoor_designs_fail() {
    let missed: Vec<_> = outdoor_checks::broken_designs()
        .into_iter()
        .filter(|(_, d, o, xiao)| {
            if *xiao {
                outdoor_checks::verify_xiao(d, o).is_ok()
            } else {
                outdoor_checks::verify_uno_nano(d, o).is_ok()
            }
        })
        .map(|(l, ..)| l)
        .collect();
    assert!(
        missed.is_empty(),
        "accepted broken outdoor designs: {missed:?}"
    );
}
