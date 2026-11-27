//! Printed Arduino payload for the AMSAT CubeSatSim v2.0 frame: CAD, checks, print files.
//!
//! The kit is the four AMSAT frame pieces (one with standoff columns added) and one payload plate
//! that holds an Arduino Uno R3 and a Nano 33 BLE Sense with lips and snap hooks printed as part
//! of it. No loose parts, screws, springs or metal.
//!
//! - [`design`]: every dimension, in one `Design`
//! - [`geom`]: geometry helpers and mesh input/output
//! - [`boards`]: the Uno and Nano from Arduino's CAD
//! - [`joints`]: snap pin, snap hook and fixed lip
//! - [`parts`]: the printed parts
//! - [`checks`]: fit and strength checks, and the broken designs the tests use
//! - [`bambu`]: Bambu Studio project files with preview pictures
//! - [`drawing`]: SVG drawings for the README
//!
//! A derivative of the AMSAT CubeSatSim frame by Alan Johnston, KU2Y, licensed GPL-3.0 like the
//! original.

pub mod bambu;
pub mod battery;
pub mod boards;
pub mod checks;
pub mod design;
pub mod drawing;
pub mod geom;
pub mod joints;
pub mod outdoor;
pub mod outdoor_checks;
pub mod parts;
pub mod xiao;
