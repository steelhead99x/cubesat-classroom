//! The design checks pass, and they can fail: every broken design must be rejected, and every
//! design exactly at a limit must be accepted. These are design checks, not measurements of prints.

use amsat_kit::checks::{boundary_designs, broken_designs, verify};
use amsat_kit::design::Design;

#[test]
fn the_kit_passes() {
    verify(&Design::default()).unwrap();
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
