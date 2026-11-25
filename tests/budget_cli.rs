//! End-to-end checks for the `link-budget` and `energy-budget` subcommands. The golden
//! files are shared with the mission-budget crate and were captured from the Python
//! tools these subcommands replace.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn golden(name: &str) -> String {
    let path: PathBuf = root().join("crates/mission-budget/tests/golden").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn station(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubesat-classroom"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("run cubesat-classroom")
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "exit {:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("UTF-8 output")
}

#[test]
fn link_budget_default_table() {
    let out = station(&["link-budget", "hardware/rf-example.json"]);
    assert_eq!(stdout(&out), golden("link-budget-default.txt"));
}

#[test]
fn link_budget_documented_command() {
    // The exact command in docs/mission.md.
    let out = station(&[
        "link-budget",
        "hardware/rf-example.json",
        "--altitude-km",
        "550",
        "--elevation-deg",
        "15",
    ]);
    assert_eq!(stdout(&out), golden("link-budget-default.txt"));
}

#[test]
fn link_budget_custom_geometry() {
    let out = station(&[
        "link-budget",
        "hardware/rf-example.json",
        "--altitude-km",
        "400",
        "--elevation-deg",
        "42.5",
        "--radial-speed-km-s",
        "3.25",
    ]);
    assert_eq!(stdout(&out), golden("link-budget-400km-42.5deg.txt"));
}

#[test]
fn link_budget_json_has_every_band() {
    let out = station(&["link-budget", "hardware/rf-example.json", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let names: Vec<_> = value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["SubGHz", "S-band", "X-band"]);
    assert_eq!(value["elevation_deg"], 15.0);
    assert!(value["rows"][0]["margin_db"].as_f64().unwrap() > 0.0);
}

#[test]
fn link_budget_rejects_bad_geometry_before_reading_the_file() {
    let out = station(&[
        "link-budget",
        "does-not-exist.json",
        "--elevation-deg",
        "-5",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("elevation 0–90"), "{stderr}");
}

#[test]
fn energy_budget_report() {
    let out = station(&["energy-budget"]);
    assert_eq!(stdout(&out), golden("energy-budget.txt"));
}

#[test]
fn energy_budget_json_flags_the_camera_payload() {
    let out = station(&["energy-budget", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let budgets = value.as_array().unwrap();
    assert_eq!(budgets.len(), 6);
    let enough: Vec<_> = budgets.iter().map(|b| b["enough"].clone()).collect();
    assert_eq!(
        enough,
        [
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
            true.into(),
            false.into(),
            true.into()
        ]
    );
    assert_eq!(budgets[4]["cells"], 4);
}
