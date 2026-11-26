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

// ---- Uno latches and lock keys (the AMSAT kit's broken designs, ported) ----

#[test]
fn latch_hooks_too_short_to_hold_the_uno() {
    rejects("FIT-latch", |d| d.latch.reach = 0.4);
}

#[test]
fn stiff_latch_arm_that_would_crack() {
    rejects("FIT-latch", |d| d.latch.t = 3.0);
}

#[test]
fn lock_key_too_loose_to_stop_a_latch_opening() {
    rejects("FIT-latch", |d| d.key.clear = 0.3);
}

#[test]
fn latch_moved_into_the_usb_b_connector() {
    rejects("FIT-uno", |d| d.latch.places[0].1 = 38.0);
}

#[test]
fn uno_moved_onto_the_nano_clamp() {
    // Down 8 mm, its DC jack and bottom latch reach the clamp's anchor pillar.
    rejects("FIT-uno", |d| d.uno_top_margin += 8.0);
}

// ---- Quarter-turn fasteners ----

#[test]
fn washer_arms_too_thick_to_flex() {
    rejects("FIT-qt", |d| d.washer.t = 2.2);
}

#[test]
fn washer_exactly_at_the_strain_limit_passes() {
    let mut d = Design::default();
    let l = cubesat_cad::fasteners::washer_arm_length(&d);
    let crest = cubesat_cad::fasteners::washer_crest(&d);
    d.washer.t = d.pla_strain_limit * l * l / (3.0 * crest);
    let failed: Vec<_> = verify(&d).failures().iter().map(|c| c.id).collect();
    assert!(!failed.contains(&"FIT-qt"), "{failed:?}");
}

#[test]
fn no_detent_so_a_stud_could_turn_loose() {
    rejects("FIT-qt", |d| d.qt.det = 0.0);
}

#[test]
fn too_little_cam_draw_in_to_hold_the_plate_down() {
    rejects("FIT-qt", |d| d.qt.lift = 0.3);
}

#[test]
fn stud_tip_too_short_behind_the_bar() {
    rejects("FIT-qt", |d| d.qt.tip = 0.6);
}

#[test]
fn plate_stud_washer_on_a_latch() {
    rejects("FIT-studs", |d| d.columns[1] = [28.0, -0.5]);
}

#[test]
fn column_under_a_uno_post() {
    rejects("FIT-studs", |d| d.columns[2] = [-18.0, -12.0]);
}

// ---- Nano card cage ----

#[test]
fn v_grooves_too_shallow_to_grip_the_nano() {
    rejects("FIT-nano", |d| d.nano.v_depth = 0.4);
}

#[test]
fn nano_parts_against_the_groove_wall() {
    rejects("FIT-nano", |d| d.nano.v_depth = 4.0);
}

#[test]
fn nano_too_close_to_the_plate_edge_for_its_plug() {
    rejects("FIT-nano", |d| d.nano.x0 = -22.0);
}

#[test]
fn nano_plug_into_the_corner_rail() {
    // 0.5 mm towards the plate edge, the USB plug hits the -X -Y rail.
    rejects("FIT-nano", |d| d.nano.y0 = -40.8);
}

#[test]
fn clamp_pillar_moved_into_the_nano() {
    rejects("FIT-nano", |d| d.nano.anchor = [-14.0, -33.0]);
}

#[test]
fn no_end_play_for_the_nano() {
    rejects("FIT-nano", |d| d.nano.end_play = -0.3);
}

// ---- Frame ----

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
fn uno_hits_the_frame() {
    rejects("FIT-boards", |d| d.uno_top_margin = -8.0);
}
