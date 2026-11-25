//! End-to-end checks for the station's offline calculators: argument parsing
//! (including negative numbers), validation messages and output shape.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn station(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubesat-classroom"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .expect("run cubesat-classroom")
}

fn ok_stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "exit {:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("UTF-8 output")
}

fn fails_with(args: &[&str], message: &str) {
    let out = station(args);
    assert!(!out.status.success(), "{args:?} should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(message), "{args:?}: {stderr}");
}

/// A fresh, empty folder under the system temp directory.
fn scratch(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("cubesat-cli-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const PASSES: [&str; 7] = [
    "passes",
    "--tle",
    "tests/fixtures/iss.tle",
    "--lat",
    "39.95",
    "--lon",
    "-75.16",
];

#[test]
fn passes_json_with_old_elements_warns_that_they_are_unusable() {
    let mut args = PASSES.to_vec();
    args.push("--json");
    let out = station(&args);
    let value: serde_json::Value = serde_json::from_str(&ok_stdout(&out)).unwrap();
    for pass in value.as_array().unwrap() {
        assert_eq!(pass["satellite"], "ISS (ZARYA)");
        assert!(pass["elements_age_days"].as_f64().unwrap() > 60.0);
        assert!(pass["rise_unix"].as_i64() < pass["set_unix"].as_i64());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not usable predictions"), "{stderr}");
}

#[test]
fn passes_table_puts_the_warning_first() {
    let out = ok_stdout(&station(&PASSES));
    assert!(
        out.starts_with("Warning: orbital elements for ISS (ZARYA)"),
        "{out}"
    );
}

#[test]
fn passes_rejects_bad_observer_and_filters() {
    let base = &PASSES[..3];
    fails_with(
        &[base, &["--lat", "91", "--lon", "0"]].concat(),
        "latitude must be between -90 and 90",
    );
    fails_with(
        &[base, &["--lat", "0", "--lon", "-181"]].concat(),
        "longitude must be between -180 and 180",
    );
    fails_with(
        &[base, &["--lat", "0", "--lon", "0", "--hours", "0"]].concat(),
        "hours must be between 0 and 240",
    );
    fails_with(
        &[
            base,
            &["--lat", "0", "--lon", "0", "--satellite", "SWISSCUBE"],
        ]
        .concat(),
        "no satellites",
    );
}

#[test]
fn flight_profile_json_is_labelled_and_lands() {
    let out = ok_stdout(&station(&["flight-profile", "--json", "--step-s", "10"]));
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["source"], "desk simulation");
    let points = value["points"].as_array().unwrap();
    assert_eq!(points.first().unwrap()["phase"], "climb");
    assert_eq!(points.last().unwrap()["phase"], "landed");
    let micro = value["profile"]["microgravity_s"].as_f64().unwrap();
    assert!(
        (150.0..=240.0).contains(&micro),
        "{micro} s of microgravity"
    );
}

#[test]
fn flight_profile_writes_a_csv_timeline() {
    let dir = scratch("flight");
    let csv = dir.join("nested/flight.csv");
    let out = ok_stdout(&station(&[
        "flight-profile",
        "--output",
        csv.to_str().unwrap(),
        "--step-s",
        "60",
    ]));
    assert!(out.contains("Desk simulation"), "{out}");
    let text = std::fs::read_to_string(&csv).unwrap();
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("elapsed_s,phase,altitude_m,felt_g,cabin_pressure_pa,source")
    );
    let rows: Vec<_> = lines.collect();
    assert!(rows.len() > 60, "{} rows", rows.len());
    assert!(rows.iter().all(|r| r.ends_with(",desk simulation")));
    assert!(rows.last().unwrap().starts_with("4200.0,landed,0.0,"));
    std::fs::remove_dir_all(Path::new(&dir)).unwrap();
}

#[test]
fn downlink_sim_reports_a_desk_simulation() {
    let out = station(&["downlink-sim", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&ok_stdout(&out)).unwrap();
    assert_eq!(value["evidence"], "desk-simulation");
    assert_eq!(value["transmits"], false);
    assert_eq!(value["burst"]["recovered_without_interleave"], 3);
    assert_eq!(value["burst"]["recovered_with_interleave"], 4);
    let rows = value["scenarios"].as_array().unwrap();
    let row = |name: &str| rows.iter().find(|row| row["name"] == name).unwrap();
    assert_eq!(row("ten-percent-no-parity")["recovered"], 364);
    assert_eq!(row("ten-percent-one-parity")["recovered"], 395);
    assert_eq!(row("forty-percent-one-receiver")["recovered"], 182);
    assert_eq!(row("forty-percent-three-receivers")["recovered"], 285);
    let text = ok_stdout(&station(&["downlink-sim"]));
    assert!(text.contains("does not transmit"));
    assert!(text.contains("Beta LoRa board"));
}

#[test]
fn flight_profile_rejects_impossible_inputs() {
    fails_with(
        &["flight-profile", "--step-s", "0"],
        "step must be between 0.1 and 60 seconds",
    );
    fails_with(
        &["flight-profile", "--total-min", "30"],
        "total flight time is too short",
    );
    fails_with(
        &["flight-profile", "--json", "--output", "x.csv"],
        "cannot be used with",
    );
}
