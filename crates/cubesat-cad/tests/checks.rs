//! Rev C passes every check, and every check catches the mistake it exists for.

use cubesat_cad::check::{Outcome, verify};
use cubesat_cad::design::Design;

#[test]
fn rev_c_passes_every_check() {
    let report = verify(&Design::default());
    let failures: Vec<_> = report
        .failures()
        .iter()
        .map(|c| format!("{}: {}", c.id, c.detail))
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn deviations_are_reported_not_hidden() {
    let report = verify(&Design::default());
    for id in [
        "CDS-2.2.6",
        "CDS-2.2.12/13",
        "CDS-2.1.7",
        "CDS-2.3.2",
        "CDS-2.3.5",
    ] {
        let check = report.checks.iter().find(|c| c.id == id).expect(id);
        assert_eq!(check.outcome, Outcome::Deviation, "{id}");
    }
}

/// Change the design, then expect this check (and possibly others) to fail.
fn rejects(id: &str, change: impl FnOnce(&mut Design)) {
    let mut d = Design::default();
    change(&mut d);
    let report = verify(&d);
    let failed: Vec<_> = report.failures().iter().map(|c| c.id).collect();
    assert!(
        failed.contains(&id),
        "expected {id} to fail, got {failed:?}"
    );
}

#[test]
fn wrong_envelope() {
    rejects("CDS-2.2.1", |d| d.width = 100.5);
    rejects("CDS-2.2.1", |d| d.length = 114.0);
}

#[test]
fn rails_too_narrow_or_sharp() {
    rejects("CDS-2.2.5", |d| d.rail = 8.0);
    rejects("CDS-2.2.7", |d| d.rail_radius = 0.5);
}

#[test]
fn end_plates_flush_with_rail_ends() {
    rejects("CDS-2.2.8", |d| d.end_recess = 0.0);
}

#[test]
fn window_cuts_into_a_rail() {
    rejects("CDS-2.2.9", |d| d.window = [99.0, 80.0]);
}

#[test]
fn snap_that_no_longer_latches() {
    rejects("FIT-snap", |d| d.snap_hole = 5.8);
    rejects("FIT-snap", |d| d.snap_hole = 5.6);
}

#[test]
fn clip_that_no_longer_holds() {
    rejects("FIT-clip", |d| d.clip_gap = 0.8);
}

#[test]
fn clip_that_would_crack() {
    rejects("FIT-strain", |d| {
        d.clip_t = 2.4;
        d.clip_lip = 1.0;
        d.uno_post_h = 6.0;
    });
}

#[test]
fn payload_plate_into_the_end_walls() {
    rejects("FIT-parts", |d| d.column_length = 5.0);
}

#[test]
fn hole_too_small_for_its_peg() {
    // Pegs and holes share their positions, so a misfit shows up as a peg that
    // can't pass through: the shank is wider than the hole.
    rejects("FIT-align", |d| d.snap_hole = 4.6);
}

#[test]
fn nano_rails_on_the_pins() {
    rejects("FIT-boards", |d| d.nano_row = 8.0);
}

#[test]
fn strap_presses_into_the_nano() {
    rejects("FIT-boards", |d| d.strap_foot_above_pcb = -0.5);
}

#[test]
fn uno_hits_the_frame() {
    rejects("FIT-boards", |d| d.uno_top_margin = -8.0);
}
