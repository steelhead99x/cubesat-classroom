//! Spherical-Earth, vacuum-path, first-order one-way link budget for design review.
//!
//! The model uses a spherical Earth slant range, free-space path loss, transmit power,
//! gain and feed loss, receive G/T, one explicit aggregate loss, the information bit
//! rate and a modem threshold on the same Eb/N0 basis. The Doppler magnitude is driven
//! independently by an assumed radial speed. Everything here is an illustrative
//! sensitivity exercise: it says nothing about channel assignments, licensing, measured
//! performance or flight availability.

use std::fmt::{self, Write as _};

use serde::{Deserialize, Serialize};

use crate::general;

/// Speed of light in vacuum, metres per second.
pub const C_M_S: f64 = 299_792_458.0;
/// Mean Earth radius used for the spherical slant range, kilometres.
pub const EARTH_RADIUS_KM: f64 = 6371.0;
/// Boltzmann's constant in dBW/(K·Hz).
pub const BOLTZMANN_DBW_K_HZ: f64 = -228.599;

/// A set of radio profiles evaluated against one pass geometry, as stored in
/// `hardware/rf-example.json`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Scenario {
    /// Caveat printed above the table.
    pub label: String,
    /// The downlinks to evaluate, in table order.
    pub profiles: Vec<Profile>,
}

/// One downlink: transmitter, path losses, receiver and modem threshold.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Profile {
    /// Short band name for the table, such as `SubGHz`.
    pub name: String,
    /// Carrier centre frequency, MHz.
    pub center_mhz: f64,
    /// Power amplifier output, dBW.
    pub pa_dbw: f64,
    /// Transmit antenna gain, dBi.
    pub tx_gain_dbi: f64,
    /// Loss between the amplifier and the transmit antenna, dB.
    pub tx_feed_loss_db: f64,
    /// Receive figure of merit G/T, dB/K.
    pub rx_gt_db_per_k: f64,
    /// Every other loss in one explicit number (pointing, polarisation, atmosphere), dB.
    pub other_losses_db: f64,
    /// Information bit rate, bit/s.
    pub information_bps: f64,
    /// Eb/N0 the modem needs at the target error rate, dB.
    pub required_ebn0_db: f64,
}

/// Pass geometry and the radial speed assumed for Doppler.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Geometry {
    altitude_km: f64,
    elevation_deg: f64,
    radial_speed_km_s: f64,
}

/// One row of the budget table.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Budget {
    /// Slant range used, km.
    pub slant_km: f64,
    /// Free-space path loss, dB.
    pub fspl_db: f64,
    /// Carrier-to-noise-density ratio, dB·Hz.
    pub cn0_dbhz: f64,
    /// Energy per information bit over noise density, dB.
    pub ebn0_db: f64,
    /// Eb/N0 above the modem threshold, dB. Negative means the link does not close.
    pub margin_db: f64,
    /// Doppler shift magnitude for the assumed radial speed, kHz.
    pub abs_doppler_khz: f64,
}

/// A whole evaluated scenario, for JSON output.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Evaluation {
    /// The scenario's caveat.
    pub label: String,
    /// The geometry evaluated (flattened into the JSON object).
    #[serde(flatten)]
    pub geometry: Geometry,
    /// Slant range for that geometry, km.
    pub slant_km: f64,
    /// One row per profile, in scenario order.
    pub rows: Vec<Row>,
}

/// A named budget row in an [`Evaluation`].
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Row {
    /// The profile's band name.
    pub name: String,
    /// The profile's budget (flattened into the JSON object).
    #[serde(flatten)]
    pub budget: Budget,
}

/// Why a geometry or scenario was rejected.
#[derive(Debug)]
pub enum Error {
    /// Altitude, elevation or radial speed out of range.
    Geometry,
    /// The scenario file is not valid JSON in the expected shape.
    Parse(serde_json::Error),
    /// A profile value would make a logarithm undefined.
    Profile {
        /// The profile's band name.
        name: String,
        /// The offending field.
        field: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry => f.write_str(
                "require positive finite altitude, elevation 0–90 and nonnegative sublight radial speed",
            ),
            Self::Parse(error) => write!(f, "invalid link budget scenario: {error}"),
            Self::Profile { name, field } => {
                write!(f, "profile {name:?}: {field} must be positive and finite")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            _ => None,
        }
    }
}

impl Geometry {
    /// Validate a pass geometry: positive finite altitude, elevation 0–90° and a
    /// nonnegative radial speed below the speed of light.
    ///
    /// # Errors
    ///
    /// [`Error::Geometry`] when any value is out of range or NaN.
    pub fn new(
        altitude_km: f64,
        elevation_deg: f64,
        radial_speed_km_s: f64,
    ) -> Result<Self, Error> {
        // Written so that NaN fails every comparison and is rejected.
        let valid = altitude_km.is_finite()
            && altitude_km > 0.0
            && (0.0..=90.0).contains(&elevation_deg)
            && (0.0..C_M_S / 1000.0).contains(&radial_speed_km_s);
        if valid {
            Ok(Self {
                altitude_km,
                elevation_deg,
                radial_speed_km_s,
            })
        } else {
            Err(Error::Geometry)
        }
    }

    /// Spacecraft altitude, km.
    #[must_use]
    pub fn altitude_km(&self) -> f64 {
        self.altitude_km
    }

    /// Elevation above the horizon, degrees.
    #[must_use]
    pub fn elevation_deg(&self) -> f64 {
        self.elevation_deg
    }

    /// Radial speed assumed for Doppler, km/s.
    #[must_use]
    pub fn radial_speed_km_s(&self) -> f64 {
        self.radial_speed_km_s
    }

    /// Slant range to the spacecraft for this geometry, kilometres.
    #[must_use]
    pub fn slant_km(&self) -> f64 {
        slant_km(self.altitude_km, self.elevation_deg)
    }
}

impl Default for Geometry {
    /// 550 km altitude, 15° elevation, 7 km/s radial speed (the CLI defaults).
    fn default() -> Self {
        Self {
            altitude_km: 550.0,
            elevation_deg: 15.0,
            radial_speed_km_s: 7.0,
        }
    }
}

impl Scenario {
    /// Parse and check a scenario file.
    ///
    /// Unknown keys are ignored, as the Python tool ignored them.
    ///
    /// # Errors
    ///
    /// [`Error::Parse`] for malformed JSON or a missing field, and [`Error::Profile`]
    /// when a frequency or bit rate is not positive and finite.
    pub fn from_json(text: &str) -> Result<Self, Error> {
        let scenario: Self = serde_json::from_str(text).map_err(Error::Parse)?;
        for profile in &scenario.profiles {
            for (field, value) in [
                ("center_mhz", profile.center_mhz),
                ("information_bps", profile.information_bps),
            ] {
                if !(value.is_finite() && value > 0.0) {
                    return Err(Error::Profile {
                        name: profile.name.clone(),
                        field,
                    });
                }
            }
        }
        Ok(scenario)
    }

    /// Evaluate every profile at one geometry.
    #[must_use]
    pub fn evaluate(&self, geometry: &Geometry) -> Evaluation {
        let distance = geometry.slant_km();
        Evaluation {
            label: self.label.clone(),
            geometry: *geometry,
            slant_km: distance,
            rows: self
                .profiles
                .iter()
                .map(|profile| Row {
                    name: profile.name.clone(),
                    budget: budget(profile, distance, geometry.radial_speed_km_s),
                })
                .collect(),
        }
    }

    /// The plain-text table printed by `cubesat-classroom link-budget`.
    #[must_use]
    pub fn report(&self, geometry: &Geometry) -> String {
        let evaluation = self.evaluate(geometry);
        let mut out = String::new();
        // Writing to a String cannot fail.
        let _ = writeln!(out, "{}", evaluation.label);
        let _ = writeln!(
            out,
            "{:.1} km slant at {}° elevation. Doppler uses a separate assumed radial speed.",
            evaluation.slant_km,
            general(geometry.elevation_deg)
        );
        let _ = writeln!(
            out,
            "{:<9} {:>8} {:>10} {:>9} {:>10} {:>14}",
            "Band", "FSPL dB", "C/N0 dBHz", "Eb/N0 dB", "Margin dB", "|Doppler| kHz"
        );
        for Row { name, budget: b } in &evaluation.rows {
            let _ = writeln!(
                out,
                "{name:<9} {:8.1} {:10.1} {:9.1} {:10.1} {:14.1}",
                b.fspl_db, b.cn0_dbhz, b.ebn0_db, b.margin_db, b.abs_doppler_khz
            );
        }
        out
    }
}

/// Slant range from a ground station to a spacecraft at `altitude_km`, seen at
/// `elevation_deg` above the horizon, on a spherical Earth.
#[must_use]
pub fn slant_km(altitude_km: f64, elevation_deg: f64) -> f64 {
    let r = EARTH_RADIUS_KM;
    let projection = r * elevation_deg.to_radians().sin();
    ((r + altitude_km).powi(2) - r.powi(2) + projection.powi(2)).sqrt() - projection
}

/// First-order budget for one profile at `distance_km`, with Doppler from an assumed
/// radial speed.
#[must_use]
pub fn budget(profile: &Profile, distance_km: f64, radial_speed_km_s: f64) -> Budget {
    let frequency_hz = profile.center_mhz * 1e6;
    let fspl_db =
        20.0 * (4.0 * std::f64::consts::PI * distance_km * 1000.0 * frequency_hz / C_M_S).log10();
    let eirp_dbw = profile.pa_dbw + profile.tx_gain_dbi - profile.tx_feed_loss_db;
    let cn0_dbhz =
        eirp_dbw + profile.rx_gt_db_per_k - fspl_db - profile.other_losses_db - BOLTZMANN_DBW_K_HZ;
    let ebn0_db = cn0_dbhz - 10.0 * profile.information_bps.log10();
    Budget {
        slant_km: distance_km,
        fspl_db,
        cn0_dbhz,
        ebn0_db,
        margin_db: ebn0_db - profile.required_ebn0_db,
        // Hz × (km/s) ÷ (m/s) = kHz.
        abs_doppler_khz: frequency_hz * radial_speed_km_s / C_M_S,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uhf() -> Profile {
        Profile {
            name: "SubGHz".into(),
            center_mhz: 436.0,
            pa_dbw: 0.0,
            tx_gain_dbi: 0.0,
            tx_feed_loss_db: 1.0,
            rx_gt_db_per_k: -12.0,
            other_losses_db: 4.0,
            information_bps: 9600.0,
            required_ebn0_db: 9.0,
        }
    }

    #[test]
    fn slant_range_is_altitude_at_zenith_and_longer_at_the_horizon() {
        assert!((slant_km(550.0, 90.0) - 550.0).abs() < 1e-9);
        let horizon = ((EARTH_RADIUS_KM + 550.0).powi(2) - EARTH_RADIUS_KM.powi(2)).sqrt();
        assert!((slant_km(550.0, 0.0) - horizon).abs() < 1e-9);
        assert!(slant_km(550.0, 15.0) > slant_km(550.0, 45.0));
    }

    #[test]
    fn free_space_loss_grows_6_db_per_doubling_of_distance() {
        let near = budget(&uhf(), 1000.0, 0.0);
        let far = budget(&uhf(), 2000.0, 0.0);
        assert!((far.fspl_db - near.fspl_db - 20.0 * 2f64.log10()).abs() < 1e-9);
        assert!((near.margin_db - far.margin_db - 20.0 * 2f64.log10()).abs() < 1e-9);
    }

    #[test]
    fn doppler_is_frequency_times_radial_speed_over_c() {
        let b = budget(&uhf(), 1000.0, 7.0);
        assert!((b.abs_doppler_khz - 436e6 * 7.0 / C_M_S).abs() < 1e-12);
        assert!(budget(&uhf(), 1000.0, 0.0).abs_doppler_khz.abs() < 1e-12);
    }

    #[test]
    fn rejects_out_of_range_geometry() {
        assert!(Geometry::new(550.0, 15.0, 7.0).is_ok());
        assert!(Geometry::new(550.0, 0.0, 0.0).is_ok());
        assert!(Geometry::new(550.0, 90.0, 0.0).is_ok());
        for (alt, elev, speed) in [
            (0.0, 15.0, 7.0),
            (-1.0, 15.0, 7.0),
            (f64::INFINITY, 15.0, 7.0),
            (f64::NAN, 15.0, 7.0),
            (550.0, -0.1, 7.0),
            (550.0, 90.1, 7.0),
            (550.0, f64::NAN, 7.0),
            (550.0, 15.0, -1.0),
            (550.0, 15.0, C_M_S / 1000.0),
            (550.0, 15.0, f64::NAN),
        ] {
            assert!(
                Geometry::new(alt, elev, speed).is_err(),
                "{alt} {elev} {speed}"
            );
        }
    }

    #[test]
    fn rejects_profiles_that_break_the_logarithms() {
        let text = r#"{"label":"x","profiles":[{"name":"bad","center_mhz":0,"pa_dbw":0,"tx_gain_dbi":0,"tx_feed_loss_db":0,"rx_gt_db_per_k":0,"other_losses_db":0,"information_bps":9600,"required_ebn0_db":0}]}"#;
        let error = Scenario::from_json(text).unwrap_err();
        assert!(error.to_string().contains("center_mhz"));
        assert!(matches!(
            Scenario::from_json("{\"label\":\"x\"}"),
            Err(Error::Parse(_))
        ));
    }
}
