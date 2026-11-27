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
