//! The Rust budgets must print exactly what the Python tools they replace printed.
//! The golden files were captured from `scripts/link_budget.py` and
//! `tools/energy_budget.py` (Python 3.12) before the port.

use mission_budget::{energy, link};

const RF_EXAMPLE: &str = include_str!("../../../hardware/rf-example.json");

fn scenario() -> link::Scenario {
    link::Scenario::from_json(RF_EXAMPLE).expect("hardware/rf-example.json parses")
}

#[test]
fn link_budget_defaults_match_python() {
    assert_eq!(
        scenario().report(&link::Geometry::default()),
        include_str!("golden/link-budget-default.txt")
    );
}

#[test]
fn link_budget_custom_geometry_matches_python() {
    let geometry = link::Geometry::new(400.0, 42.5, 3.25).unwrap();
    assert_eq!(
        scenario().report(&geometry),
        include_str!("golden/link-budget-400km-42.5deg.txt")
    );
}

#[test]
fn link_budget_grazing_elevation_matches_python() {
    let geometry = link::Geometry::new(550.0, 0.00001, 0.0).unwrap();
    assert_eq!(
        scenario().report(&geometry),
        include_str!("golden/link-budget-grazing.txt")
    );
}

#[test]
fn energy_budget_matches_python() {
    assert_eq!(energy::report(), include_str!("golden/energy-budget.txt"));
}
