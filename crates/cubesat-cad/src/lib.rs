//! Rev C: a printed 1U CubeSat frame for a classroom, with its CAD, its checks
//! against the CubeSat Design Specification, and its Bambu P1S print files, all in Rust.
//!
//! - [`design::Design`] holds every dimension.
//! - [`parts`] models each part where it sits in the assembled CubeSat.
//! - [`loads`] estimates launch and handling loads, margins and frequencies.
//! - [`check::verify`] runs the CDS, loads, fit and printability checks.
//! - [`export::write_all`] writes the STL files, P1S plates and report.

pub mod check;
pub mod design;
pub mod export;
pub mod geom;
pub mod loads;
pub mod mesh;
pub mod parts;
