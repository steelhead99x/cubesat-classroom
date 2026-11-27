//! Bench tools for CubeSat Classroom.
//!
//! - [`serial_image`] rebuilds a photo from the XIAO bench CubeSat's `DOWNLINK`
//!   output and checks it against the byte count and CRC-32 it was sent with.
//! - [`sweep`] records and compares RTL-SDR spectrum sweeps for the CSC-R1
//!   receiver bench quests.
//! - [`power`] logs and summarises radio power from the INA226 logger sketch
//!   or a Nordic PPK2 export.
//!
//! Every result these tools print is hardware bench (or received RF) evidence
//! only when it came from a real capture. Run on a synthetic file, it is a desk
//! simulation.

pub mod power;
pub mod serial_image;
pub mod sweep;
pub mod util;
