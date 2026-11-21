//! Satellite pass prediction for a ground station.
//!
//! Reads orbital elements in the standard three-line TLE format (as published by
//! CelesTrak and AMSAT), propagates them with SGP4, and lists when each satellite
//! rises above the horizon, how high it climbs, and when it sets, for an observer
//! at a given latitude, longitude and altitude.
//!
//! Accuracy is classroom grade: a few seconds to a minute for fresh elements. The
//! Earth rotation uses mean sidereal time and ignores polar motion. Elements more
//! than two weeks old are flagged, because predictions drift as they age.

use std::f64::consts::PI;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

const WGS84_A_KM: f64 = 6378.137;
const WGS84_F: f64 = 1.0 / 298.257_223_563;
const COARSE_STEP_S: f64 = 10.0;
const STALE_DAYS: f64 = 14.0;

pub(crate) struct PassArgs {
    pub tle: PathBuf,
    pub satellite: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    pub hours: f64,
    pub min_elevation: f64,
    pub json: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Observer {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_km: f64,
}

/// Where a satellite appears from the observer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Look {
    pub elevation_deg: f64,
    pub azimuth_deg: f64,
    pub range_km: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct Pass {
    pub satellite: String,
    pub rise_unix: i64,
    pub rise_azimuth_deg: f64,
    pub max_unix: i64,
    pub max_elevation_deg: f64,
    pub set_unix: i64,
    pub set_azimuth_deg: f64,
    pub duration_s: i64,
    /// Age of the orbital elements at the start of the search. Over 14 days is unreliable.
    pub elements_age_days: f64,
}

/// Greenwich mean sidereal time in radians for a Unix time in seconds.
pub(crate) fn gmst(unix_s: f64) -> f64 {
    let days = unix_s / 86_400.0 + 2_440_587.5 - 2_451_545.0;
    (280.460_618_37 + 360.985_647_366_29 * days)
        .to_radians()
        .rem_euclid(2.0 * PI)
}

/// TEME (the frame SGP4 uses) to Earth-fixed, rotating by sidereal time.
pub(crate) fn teme_to_ecef(p: [f64; 3], unix_s: f64) -> [f64; 3] {
    let (s, c) = gmst(unix_s).sin_cos();
    [c * p[0] + s * p[1], -s * p[0] + c * p[1], p[2]]
}

pub(crate) fn observer_ecef(o: Observer) -> [f64; 3] {
    let e2 = WGS84_F * (2.0 - WGS84_F);
    let (lat, lon) = (o.lat_deg.to_radians(), o.lon_deg.to_radians());
    let n = WGS84_A_KM / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    [
        (n + o.alt_km) * lat.cos() * lon.cos(),
        (n + o.alt_km) * lat.cos() * lon.sin(),
        (n * (1.0 - e2) + o.alt_km) * lat.sin(),
    ]
}

/// Elevation, azimuth and range of an Earth-fixed point from the observer.
pub(crate) fn look(o: Observer, sat_ecef: [f64; 3]) -> Look {
    let obs = observer_ecef(o);
    let r = [
        sat_ecef[0] - obs[0],
        sat_ecef[1] - obs[1],
        sat_ecef[2] - obs[2],
    ];
    let (lat, lon) = (o.lat_deg.to_radians(), o.lon_deg.to_radians());
    let east = -lon.sin() * r[0] + lon.cos() * r[1];
    let north = -lat.sin() * lon.cos() * r[0] - lat.sin() * lon.sin() * r[1] + lat.cos() * r[2];
    let up = lat.cos() * lon.cos() * r[0] + lat.cos() * lon.sin() * r[1] + lat.sin() * r[2];
    Look {
        elevation_deg: up.atan2(east.hypot(north)).to_degrees(),
        azimuth_deg: east.atan2(north).to_degrees().rem_euclid(360.0),
        range_km: (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt(),
    }
}

struct Satellite {
    name: String,
    epoch_unix: f64,
    constants: sgp4::Constants,
}

impl Satellite {
    fn look(&self, o: Observer, unix_s: f64) -> Option<Look> {
        let minutes = (unix_s - self.epoch_unix) / 60.0;
        let p = self
            .constants
            .propagate(sgp4::MinutesSinceEpoch(minutes))
            .ok()?;
        Some(look(o, teme_to_ecef(p.position, unix_s)))
    }

    fn elevation(&self, o: Observer, unix_s: f64) -> f64 {
        self.look(o, unix_s).map_or(-90.0, |l| l.elevation_deg)
    }
}

/// Bisect to the second at which elevation crosses zero between `a` and `b`.
fn crossing(sat: &Satellite, o: Observer, mut a: f64, mut b: f64) -> f64 {
    let rising = sat.elevation(o, a) < 0.0;
    while b - a > 1.0 {
        let mid = (a + b) / 2.0;
        if (sat.elevation(o, mid) < 0.0) == rising {
            a = mid
        } else {
            b = mid
        }
    }
    b
}

fn find_passes(sat: &Satellite, o: Observer, start: f64, end: f64, min_el: f64) -> Vec<Pass> {
    let mut passes = Vec::new();
    let mut t = start;
    let mut prev = sat.elevation(o, t);
    // If already above the horizon at the start, the first rise is "now".
    let mut rise = (prev >= 0.0).then_some(start);
    while t < end {
        let next = (t + COARSE_STEP_S).min(end);
        let el = sat.elevation(o, next);
        if prev < 0.0 && el >= 0.0 {
            rise = Some(crossing(sat, o, t, next));
        } else if prev >= 0.0
            && el < 0.0
            && let Some(r) = rise.take()
        {
            let set = crossing(sat, o, t, next);
            if let Some(p) = summarise(sat, o, r, set, min_el) {
                passes.push(p);
            }
        }
        prev = el;
        t = next;
    }
    passes
}

fn summarise(sat: &Satellite, o: Observer, rise: f64, set: f64, min_el: f64) -> Option<Pass> {
    let (mut max_t, mut max_el) = (rise, -90.0);
    let mut t = rise;
    while t <= set {
        let el = sat.elevation(o, t);
        if el > max_el {
            (max_t, max_el) = (t, el)
        }
        t += 5.0;
    }
    if max_el < min_el {
        return None;
    }
    Some(Pass {
        satellite: sat.name.clone(),
        rise_unix: rise.round() as i64,
        rise_azimuth_deg: sat.look(o, rise)?.azimuth_deg,
        max_unix: max_t.round() as i64,
        max_elevation_deg: max_el,
        set_unix: set.round() as i64,
        set_azimuth_deg: sat.look(o, set)?.azimuth_deg,
        duration_s: (set - rise).round() as i64,
        elements_age_days: 0.0,
    })
}

fn load(tle: &str, filter: Option<&str>) -> Result<Vec<Satellite>> {
    let elements =
        sgp4::parse_3les(tle).context("read TLE file (three-line format: name, line 1, line 2)")?;
    let filter = filter.map(str::to_lowercase);
    elements
        .into_iter()
        .filter(|e| match (&filter, &e.object_name) {
            (Some(f), Some(name)) => name.to_lowercase().contains(f.as_str()),
            (Some(_), None) => false,
            (None, _) => true,
        })
        .map(|e| {
            let dt = e.datetime.and_utc();
            let epoch_unix = dt.timestamp() as f64 + f64::from(dt.timestamp_subsec_nanos()) / 1e9;
            let name = e
                .object_name
                .clone()
                .unwrap_or_else(|| format!("NORAD {}", e.norad_id));
            let constants = sgp4::Constants::from_elements(&e)
                .with_context(|| format!("elements for {name}"))?;
            Ok(Satellite {
                name,
                epoch_unix,
                constants,
            })
        })
        .collect()
}

/// UTC date and time for a Unix timestamp, without a date library.
pub(crate) fn utc(unix: i64) -> String {
    let days = unix.div_euclid(86_400);
    let secs = unix.rem_euclid(86_400);
    // Howard Hinnant's civil-from-days algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

fn compass(az: f64) -> &'static str {
    const POINTS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    POINTS[((az + 22.5).rem_euclid(360.0) / 45.0) as usize % 8]
}

pub(crate) fn run(args: PassArgs) -> Result<()> {
    ensure!(
        (-90.0..=90.0).contains(&args.lat),
        "latitude must be between -90 and 90"
    );
    ensure!(
        (-180.0..=180.0).contains(&args.lon),
        "longitude must be between -180 and 180"
    );
    ensure!(
        args.hours > 0.0 && args.hours <= 240.0,
        "hours must be between 0 and 240"
    );
    ensure!(
        args.alt_m.is_finite() && (-500.0..=9000.0).contains(&args.alt_m),
        "altitude must be between -500 and 9000 metres"
    );
    ensure!(
        (-90.0..=90.0).contains(&args.min_elevation),
        "minimum elevation must be between -90 and 90"
    );
    let text = std::fs::read_to_string(&args.tle)
        .with_context(|| format!("read {}", args.tle.display()))?;
    let sats = load(&text, args.satellite.as_deref())?;
    ensure!(
        !sats.is_empty(),
        "no satellites in {} match",
        args.tle.display()
    );

    let now = super::unix_ms()? as f64 / 1000.0;
    let o = Observer {
        lat_deg: args.lat,
        lon_deg: args.lon,
        alt_km: args.alt_m / 1000.0,
    };
    let mut stale = Vec::new();
    let mut passes: Vec<Pass> = Vec::new();
    for s in &sats {
        let age = (now - s.epoch_unix) / 86_400.0;
        if s.look(o, now).is_none() {
            eprintln!(
                "Warning: could not compute where {} is now. Its elements may be too old, or it may have re-entered.",
                s.name
            );
        }
        if age > STALE_DAYS {
            stale.push(format!("{} ({age:.0} days)", s.name));
        }
        for mut p in find_passes(s, o, now, now + args.hours * 3600.0, args.min_elevation) {
            p.elements_age_days = (age * 10.0).round() / 10.0;
            passes.push(p);
        }
    }
    let warning = (!stale.is_empty()).then(|| {
        format!(
            "Warning: old orbital elements for {}. Times may be wrong by minutes or more. Download fresh elements.",
            stale.join(", ")
        )
    });
    passes.sort_by_key(|p| p.rise_unix);

    if args.json {
        if let Some(w) = &warning {
            eprintln!("{w}");
        }
        println!("{}", serde_json::to_string_pretty(&passes)?);
        return Ok(());
    }
    if let Some(w) = &warning {
        println!("{w}\n");
    }
    if passes.is_empty() {
        println!(
            "No passes above {}° in the next {} hours.",
            args.min_elevation, args.hours
        );
        return Ok(());
    }
    println!(
        "{:<18} {:<19} {:<5} {:>7} {:<19} {:<5} {:>7}",
        "Satellite", "Rises (UTC)", "Look", "Highest", "Sets (UTC)", "Look", "Minutes"
    );
    for p in passes {
        println!(
            "{:<18} {:<19} {:<5} {:>6.0}° {:<19} {:<5} {:>7.1}",
            p.satellite.chars().take(18).collect::<String>(),
            utc(p.rise_unix),
            compass(p.rise_azimuth_deg),
            p.max_elevation_deg,
            utc(p.set_unix),
            compass(p.set_azimuth_deg),
            p.duration_s as f64 / 60.0
        );
    }
    println!("Times are UTC. Direction is where to look as the satellite rises and sets.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISS: &str = include_str!("../tests/fixtures/iss.tle");

    #[test]
    fn overhead_point_is_at_zenith() {
        let o = Observer {
            lat_deg: 40.0,
            lon_deg: -75.0,
            alt_km: 0.0,
        };
        let ground = observer_ecef(o);
        let norm = (ground[0].powi(2) + ground[1].powi(2) + ground[2].powi(2)).sqrt();
        // A point 500 km out along the geocentric radius is within a fraction of a degree of zenith.
        let above = ground.map(|c| c * (norm + 500.0) / norm);
        let l = look(o, above);
        assert!(l.elevation_deg > 89.0, "{l:?}");
        assert!((l.range_km - 500.0).abs() < 5.0);
    }

    #[test]
    fn formats_utc_dates() {
        assert_eq!(utc(0), "1970-01-01 00:00:00");
        assert_eq!(utc(951_782_400), "2000-02-29 00:00:00");
        assert_eq!(utc(1_795_132_800), "2026-11-20 00:00:00");
        assert_eq!(compass(0.0), "N");
        assert_eq!(compass(130.0), "SE");
    }

    #[test]
    fn finds_plausible_iss_passes_near_epoch() {
        let sats = load(ISS, Some("iss")).unwrap();
        let s = &sats[0];
        let o = Observer {
            lat_deg: 39.95,
            lon_deg: -75.16,
            alt_km: 0.01,
        };
        let passes = find_passes(s, o, s.epoch_unix, s.epoch_unix + 86_400.0, 0.0);
        // A low-orbit station passes a mid-latitude site several times a day.
        assert!((3..=8).contains(&passes.len()), "{} passes", passes.len());
        for p in &passes {
            assert!(p.duration_s > 60 && p.duration_s < 15 * 60, "{p:?}");
            assert!(p.max_elevation_deg > 0.0 && p.max_elevation_deg <= 90.0);
            assert!(p.rise_unix < p.max_unix && p.max_unix < p.set_unix);
        }
    }
}
