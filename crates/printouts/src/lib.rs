//! Printable handouts for CubeSat Classroom.
//!
//! `cargo run --locked -p printouts --bin quick-start-pdf` rebuilds
//! `output/pdf/cubesat-classroom-quick-start.pdf` from the site settings and
//! `docs/quick-start.md`. It uses the PDF base-14 fonts, so it needs no font
//! files and no Python.

pub mod layout;
pub mod metrics;
pub mod pdf;
pub mod quick_start;
