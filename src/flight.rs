//! Desk simulation of a suborbital research flight on an air-launched spaceplane.
//!
//! The model turns a handful of published figures into a second-by-second
//! timeline so a class can predict what a sensor logger should record, then
//! compare the prediction with real flight data afterwards.
//!
//! Defaults come from Virgin Galactic's public material: the May 2024 Payload
//! User's Guide (release near 45,000 ft, a rocket burn of about 60 s, about
//! 3 minutes of microgravity, cabin held near a 5,000 to 6,000 ft equivalent)
//! and the published facts for the 'Galactic 07' flight of June 8, 2024
//! (release at 44,562 ft, apogee 54.4 miles, 70 minutes from take-off to landing).
//!
//! Everything this module prints is a desk simulation. It assumes a straight
//! vertical climb at constant thrust, no air drag while coasting and a smooth
//! descent. Speed is not carried over from the coast into re-entry, so the
//! re-entry curve shows the shape of the descent, not its real rate. The host
//! does not publish g-loads, so the boost figure is this
//! model's estimate and re-entry loads are left blank. It is not flight data
//! and not a flight plan.

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

const G0: f64 = 9.806_65;
const FT: f64 = 0.3048;
const MILE: f64 = 1609.344;

pub(crate) struct FlightArgs {
    pub output: Option<PathBuf>,
    pub release_ft: f64,
    pub apogee_miles: f64,
    pub burn_s: f64,
    pub climb_min: f64,
    pub total_min: f64,
    pub cabin_ft: f64,
    pub step_s: f64,
    pub json: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase {
    Climb,
    Boost,
    Microgravity,
    Reentry,
    Glide,
    Landed,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Climb => "climb under the carrier aircraft",
            Phase::Boost => "rocket boost",
            Phase::Microgravity => "microgravity coast",
            Phase::Reentry => "re-entry",
            Phase::Glide => "glide to the runway",
            Phase::Landed => "landed",
        }
    }
}

/// The flight reduced to the times and heights where one phase hands over to the next.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Profile {
    pub release_s: f64,
    pub burnout_s: f64,
    pub apogee_s: f64,
    pub reentry_s: f64,
    pub glide_s: f64,
    pub landing_s: f64,
    pub release_m: f64,
    pub burnout_m: f64,
    pub apogee_m: f64,
    pub glide_m: f64,
    pub burnout_speed_m_s: f64,
    pub boost_felt_g: f64,
    pub microgravity_s: f64,
    pub cabin_pressure_pa: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Point {
    pub elapsed_s: f64,
    pub phase: Phase,
    pub altitude_m: f64,
    /// What an accelerometer along the flight path would read, in g. Empty where the model has no basis.
    pub felt_g: Option<f64>,
    pub cabin_pressure_pa: f64,
}

/// Pressure at a height in the International Standard Atmosphere (valid below 11 km).
pub(crate) fn isa_pressure_pa(height_m: f64) -> f64 {
    101_325.0 * (1.0 - 2.255_77e-5 * height_m).powf(5.255_88)
}

impl Profile {
    pub fn new(a: &FlightArgs) -> Result<Self> {
        let release_m = a.release_ft * FT;
        let apogee_m = a.apogee_miles * MILE;
        ensure!(
            (16_500.0..=65_000.0).contains(&a.release_ft),
            "release height must be between 16,500 and 65,000 ft"
        );
        ensure!(
            apogee_m > release_m + 20_000.0 && apogee_m <= 150_000.0,
            "apogee must be well above release and at most about 93 miles"
        );
        ensure!(
            (20.0..=180.0).contains(&a.burn_s),
            "burn must be between 20 and 180 seconds"
        );
        ensure!(
            (0.0..=10_000.0).contains(&a.cabin_ft),
            "cabin altitude must be between 0 and 10,000 ft"
        );
        ensure!(
            (10.0..=180.0).contains(&a.climb_min),
            "climb must be between 10 and 180 minutes"
        );
        ensure!(
            a.total_min.is_finite() && a.total_min <= 300.0,
            "total flight time must be at most 300 minutes"
        );

        // Constant acceleration for the burn, then a drag-free coast:
        //   apogee = release + v*burn/2 + v^2/(2g). Solve for the burnout speed v.
        let rise = apogee_m - release_m;
        let half = a.burn_s / 2.0;
        let v = G0 * (-half + (half * half + 2.0 * rise / G0).sqrt());
        let burnout_m = release_m + v * half;
        let coast = v / G0;

        let release_s = a.climb_min * 60.0;
        let burnout_s = release_s + a.burn_s;
        let apogee_s = burnout_s + coast;
        let reentry_s = apogee_s + coast;
        // The guide says the feather comes down near 50,000 ft. Allow three minutes to get there.
        let glide_m = 50_000.0 * FT;
        let glide_s = reentry_s + 180.0;
        let landing_s = a.total_min * 60.0;
        ensure!(
            landing_s > glide_s + 300.0,
            "total flight time is too short for this climb and apogee"
        );

        Ok(Self {
            release_s,
            burnout_s,
            apogee_s,
            reentry_s,
            glide_s,
            landing_s,
            release_m,
            burnout_m,
            apogee_m,
            glide_m,
            burnout_speed_m_s: v,
            boost_felt_g: (v / a.burn_s + G0) / G0,
            microgravity_s: 2.0 * coast,
            cabin_pressure_pa: isa_pressure_pa(a.cabin_ft * FT),
        })
    }

    pub fn at(&self, t: f64) -> Point {
        let smooth = |x: f64| x * x * (3.0 - 2.0 * x);
        let (phase, altitude_m, felt_g) = if t < self.release_s {
            (
                Phase::Climb,
                self.release_m * smooth(t / self.release_s),
                Some(1.0),
            )
        } else if t < self.burnout_s {
            let dt = t - self.release_s;
            let accel = self.burnout_speed_m_s / (self.burnout_s - self.release_s);
            (
                Phase::Boost,
                self.release_m + 0.5 * accel * dt * dt,
                Some(self.boost_felt_g),
            )
        } else if t < self.reentry_s {
            let dt = t - self.apogee_s;
            (
                Phase::Microgravity,
                self.apogee_m - 0.5 * G0 * dt * dt,
                Some(0.0),
            )
        } else if t < self.glide_s {
            let x = (t - self.reentry_s) / (self.glide_s - self.reentry_s);
            (
                Phase::Reentry,
                // Ease out: fast at first, as the ship arrives falling, then slower in thicker air.
                self.burnout_m + (self.glide_m - self.burnout_m) * (1.0 - (1.0 - x).powi(3)),
                None,
            )
        } else if t < self.landing_s {
            let x = (t - self.glide_s) / (self.landing_s - self.glide_s);
            (Phase::Glide, self.glide_m * (1.0 - x), Some(1.0))
        } else {
            (Phase::Landed, 0.0, Some(1.0))
        };
        Point {
            elapsed_s: t,
            phase,
            altitude_m,
            felt_g,
            cabin_pressure_pa: self.cabin_pressure_pa,
        }
    }

    pub fn points(&self, step_s: f64) -> Vec<Point> {
        let n = (self.landing_s / step_s).floor() as usize;
        let mut points: Vec<Point> = (0..=n).map(|i| self.at(i as f64 * step_s)).collect();
        if points.last().is_some_and(|p| p.phase != Phase::Landed) {
            points.push(self.at(self.landing_s));
        }
        points
    }
}

fn clock(s: f64) -> String {
    let s = s.round() as i64;
    format!("T+{:02}:{:02}", s / 60, s % 60)
}

pub(crate) fn run(args: FlightArgs) -> Result<()> {
    ensure!(
        (0.1..=60.0).contains(&args.step_s),
        "step must be between 0.1 and 60 seconds"
    );
    let p = Profile::new(&args)?;
    let points = p.points(args.step_s);

    if args.json {
        #[derive(Serialize)]
        struct Out<'a> {
            source: &'static str,
            profile: &'a Profile,
            points: &'a [Point],
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&Out {
                source: "desk simulation",
                profile: &p,
                points: &points
            })?
        );
        return Ok(());
    }

    if let Some(path) = &args.output {
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        let mut f =
            std::fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
        writeln!(
            f,
            "elapsed_s,phase,altitude_m,felt_g,cabin_pressure_pa,source"
        )?;
        for pt in &points {
            let g = pt.felt_g.map_or(String::new(), |g| format!("{g:.2}"));
            let phase = serde_json::to_string(&pt.phase)?;
            writeln!(
                f,
                "{:.1},{},{:.1},{},{:.0},desk simulation",
                pt.elapsed_s,
                phase.trim_matches('"'),
                pt.altitude_m,
                g,
                pt.cabin_pressure_pa
            )?;
        }
        println!("Wrote {} rows to {}", points.len(), path.display());
    }

    println!("Desk simulation of a suborbital research flight. Not flight data.\n");
    let rows = [
        (0.0, Phase::Climb, 0.0),
        (p.release_s, Phase::Boost, p.release_m),
        (p.burnout_s, Phase::Microgravity, p.burnout_m),
        (p.reentry_s, Phase::Reentry, p.burnout_m),
        (p.glide_s, Phase::Glide, p.glide_m),
        (p.landing_s, Phase::Landed, 0.0),
    ];
    println!("{:<9} {:<34} {:>10}", "Starts", "Phase", "Height");
    for (t, phase, h) in rows {
        println!(
            "{:<9} {:<34} {:>7.1} km",
            clock(t),
            phase.label(),
            h / 1000.0
        );
    }
    println!(
        "\nHighest point: {:.1} km at {}",
        p.apogee_m / 1000.0,
        clock(p.apogee_s)
    );
    println!(
        "Microgravity: about {:.0} s. A logger sampling once a second gets about {:.0} readings.",
        p.microgravity_s, p.microgravity_s
    );
    println!(
        "Speed at engine cut-off: about {:.0} m/s (model).",
        p.burnout_speed_m_s
    );
    println!(
        "Felt during boost: about {:.1} g (model estimate; the host does not publish g-loads).",
        p.boost_felt_g
    );
    println!(
        "Cabin pressure: about {:.1} kPa. The cabin stays pressurised, so it does not follow the outside air.",
        p.cabin_pressure_pa / 1000.0
    );
    println!(
        "\nTimes of release and landing are assumptions. Ask the flight provider for the real event times."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn galactic_07() -> FlightArgs {
        FlightArgs {
            output: None,
            release_ft: 44_562.0,
            apogee_miles: 54.4,
            burn_s: 60.0,
            climb_min: 50.0,
            total_min: 70.0,
            cabin_ft: 5_500.0,
            step_s: 1.0,
            json: false,
        }
    }

    #[test]
    fn published_figures_give_about_three_minutes_of_microgravity() {
        let p = Profile::new(&galactic_07()).unwrap();
        // The guide quotes about 3 minutes; the published top speed was Mach 2.96 (roughly 900 to 1000 m/s).
        assert!(
            (170.0..=210.0).contains(&p.microgravity_s),
            "{}",
            p.microgravity_s
        );
        assert!(
            (880.0..=1000.0).contains(&p.burnout_speed_m_s),
            "{}",
            p.burnout_speed_m_s
        );
        assert!((p.at(p.apogee_s).altitude_m - 54.4 * MILE).abs() < 1.0);
    }

    #[test]
    fn altitude_is_continuous_and_phases_run_in_order() {
        let p = Profile::new(&galactic_07()).unwrap();
        let pts = p.points(1.0);
        for w in pts.windows(2) {
            assert!(
                (w[1].altitude_m - w[0].altitude_m).abs() < 1_100.0,
                "jump at {}",
                w[1].elapsed_s
            );
            assert!(w[1].phase as u8 >= w[0].phase as u8);
        }
        assert_eq!(pts.first().unwrap().altitude_m, 0.0);
        assert_eq!(pts.last().unwrap().phase, Phase::Landed);
        assert!(
            pts.iter()
                .filter(|p| p.phase == Phase::Reentry)
                .all(|p| p.felt_g.is_none())
        );
    }

    #[test]
    fn cabin_pressure_matches_the_standard_atmosphere() {
        assert!((isa_pressure_pa(0.0) - 101_325.0).abs() < 1.0);
        // 5,000 ft is about 84.3 kPa and 6,000 ft about 81.2 kPa.
        assert!((isa_pressure_pa(5_000.0 * FT) - 84_300.0).abs() < 200.0);
        assert!((isa_pressure_pa(6_000.0 * FT) - 81_200.0).abs() < 200.0);
    }

    #[test]
    fn rejects_impossible_flights() {
        let mut a = galactic_07();
        a.total_min = 55.0;
        assert!(Profile::new(&a).is_err());
        let mut a = galactic_07();
        a.apogee_miles = 5.0;
        assert!(Profile::new(&a).is_err());
        for bad in [f64::INFINITY, f64::NAN, 1e7] {
            let mut a = galactic_07();
            a.total_min = bad;
            assert!(Profile::new(&a).is_err(), "{bad}");
        }
        let mut a = galactic_07();
        a.release_ft = f64::NAN;
        assert!(Profile::new(&a).is_err());
    }

    #[test]
    fn timeline_always_ends_on_the_ground() {
        let p = Profile::new(&galactic_07()).unwrap();
        let pts = p.points(7.3);
        assert_eq!(pts.last().unwrap().phase, Phase::Landed);
        assert_eq!(pts.last().unwrap().altitude_m, 0.0);
    }
}
