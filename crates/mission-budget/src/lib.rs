//! Design-review budgets for CubeSat Classroom.
//!
//! These are planning numbers for a classroom design review, not measurements or
//! flight predictions:
//!
//! - [`link`]: spherical-Earth, vacuum-path, first-order one-way downlink budget
//!   (formerly `scripts/link_budget.py`).
//! - [`energy`]: battery energy budgets for the ground station and balloon payloads
//!   (formerly `tools/energy_budget.py`).
//!
//! Both modules keep the arithmetic and the printed text of the original Python tools,
//! and golden tests in `tests/` compare their output with what those tools printed.

pub mod energy;
mod format;
pub mod link;

pub use format::general;
