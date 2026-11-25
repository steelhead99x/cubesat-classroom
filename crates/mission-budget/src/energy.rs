//! Battery energy budgets for the ground station and balloon payloads.
//!
//! Every power figure names its source in `docs/ground-station.md` or
//! `docs/balloon-lab.md`. Figures marked "estimate" have no measurement behind them
//! yet: replace them with your own bench readings (a USB power meter is enough) and
//! rerun. Budgets are planning numbers, not measured results.
//!
//! One Energizer L91 lithium AA is rated 3500 mAh at 100 mA and 3400 mAh at 1000 mA
//! (Energizer datasheet). The budget counts 3000 mAh at about 1.4 V under load to leave
//! room. Four cells in series give about 6 V fresh, falling as they drain; a buck-boost
//! regulator turns that into a steady 3.3 V (payload A) or 5 V (payload B). Eight cells
//! are two strings of four in parallel ("4s2p"), so the voltage stays the same and the
//! capacity doubles.

use std::fmt::Write as _;

use serde::Serialize;

use crate::general;

/// Usable energy in one lithium AA: 3000 mAh at 1.4 V.
pub const AA_LITHIUM_WH: f64 = 3.0 * 1.4;
/// Estimate for a small buck-boost regulator.
pub const REGULATOR_EFFICIENCY: f64 = 0.85;
/// Estimate: keep 25% in reserve for cold and ageing.
pub const COLD_DERATE: f64 = 0.75;
/// Plan for twice the expected run time.
pub const MARGIN: f64 = 2.0;
/// Preparation, about 3 h of flight, recovery.
pub const HOURS: f64 = 6.0;
/// RTL-SDR Blog V4: 250–270 mA at 5 V.
pub const RTL_SDR_W: f64 = 0.27 * 5.0;

/// Energy left in `cells` lithium AA cells after regulator losses and cold derating.
#[must_use]
pub fn usable_wh(cells: u32) -> f64 {
    f64::from(cells) * AA_LITHIUM_WH * REGULATOR_EFFICIENCY * COLD_DERATE
}

/// One electrical load and where its figure comes from (in the name).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Load {
    /// What the load is, with the source of its figure.
    pub name: String,
    /// Average power draw, W.
    pub watts: f64,
}

impl Load {
    /// A load drawing `watts` on average.
    pub fn new(name: impl Into<String>, watts: f64) -> Self {
        Self {
            name: name.into(),
            watts,
        }
    }
}

/// A set of loads run for a number of hours, optionally from a lithium AA pack.
#[derive(Clone, Debug, PartialEq)]
pub struct Budget {
    /// Heading printed above the loads.
    pub title: String,
    /// Every load that runs at once.
    pub loads: Vec<Load>,
    /// Planned run time before the margin, h.
    pub hours: f64,
    /// Lithium AA cells in the pack, if this budget runs from one.
    pub cells: Option<u32>,
    /// Extra line printed after the budget.
    pub note: Option<String>,
}

/// Machine-readable figures for one [`Budget`], for `energy-budget --json`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Summary {
    /// The budget's heading.
    pub title: String,
    /// Every load, in order.
    pub loads: Vec<Load>,
    /// Sum of the loads, W.
    pub total_w: f64,
    /// Planned run time before the margin, h.
    pub hours: f64,
    /// Run-time margin factor.
    pub margin: f64,
    /// Energy needed including the margin, Wh.
    pub need_wh: f64,
    /// Lithium AA cells in the pack, if any.
    pub cells: Option<u32>,
    /// Usable pack energy after derating, Wh.
    pub usable_wh: Option<f64>,
    /// Expected run time on the pack, h.
    pub run_time_h: Option<f64>,
    /// Whether the pack covers `need_wh`.
    pub enough: Option<bool>,
    /// The note printed after the budget, if any.
    pub note: Option<String>,
}

impl Budget {
    /// A budget with no battery pack and no note.
    #[must_use]
    pub fn new(title: impl Into<String>, loads: Vec<Load>, hours: f64) -> Self {
        Self {
            title: title.into(),
            loads,
            hours,
            cells: None,
            note: None,
        }
    }

    /// Run this budget from a pack of `cells` lithium AA cells.
    #[must_use]
    pub fn with_cells(mut self, cells: u32) -> Self {
        self.cells = Some(cells);
        self
    }

    /// Print `note` after this budget.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Total load, watts.
    #[must_use]
    pub fn total_w(&self) -> f64 {
        self.loads.iter().map(|load| load.watts).sum()
    }

    /// Energy needed for the run time with the planning margin, watt-hours.
    #[must_use]
    pub fn need_wh(&self) -> f64 {
        self.total_w() * self.hours * MARGIN
    }

    /// Usable pack energy, if this budget has a pack (zero cells counts as no pack).
    #[must_use]
    pub fn usable_wh(&self) -> Option<f64> {
        self.cells.filter(|&cells| cells > 0).map(usable_wh)
    }

    /// Whether the pack covers [`Budget::need_wh`], if this budget has a pack.
    #[must_use]
    pub fn enough(&self) -> Option<bool> {
        self.usable_wh().map(|have| have >= self.need_wh())
    }

    /// Expected run time on the pack, hours. `None` without a pack or with no load.
    #[must_use]
    pub fn run_time_h(&self) -> Option<f64> {
        let watts = self.total_w();
        self.usable_wh()
            .filter(|_| watts > 0.0)
            .map(|have| have / watts)
    }

    /// The figures behind [`Budget::render`], for JSON output.
    #[must_use]
    pub fn summary(&self) -> Summary {
        Summary {
            title: self.title.clone(),
            loads: self.loads.clone(),
            total_w: self.total_w(),
            hours: self.hours,
            margin: MARGIN,
            need_wh: self.need_wh(),
            cells: self.cells.filter(|&cells| cells > 0),
            usable_wh: self.usable_wh(),
            run_time_h: self.run_time_h(),
            enough: self.enough(),
            note: self.note.clone(),
        }
    }

    /// The text block printed for this budget, starting with a blank line.
    #[must_use]
    pub fn render(&self) -> String {
        let watts = self.total_w();
        let need = self.need_wh();
        let mut out = String::new();
        // Writing to a String cannot fail.
        let _ = writeln!(out, "\n{}", self.title);
        for load in &self.loads {
            let _ = writeln!(out, "  {:<54} {:5.2} W", load.name, load.watts);
        }
        let _ = writeln!(out, "  {:<54} {watts:5.2} W", "Total");
        let _ = writeln!(
            out,
            "  Energy for {} h with {}x margin: {need:.1} Wh",
            general(self.hours),
            general(MARGIN)
        );
        if let (Some(cells), Some(have)) = (self.cells, self.usable_wh()) {
            let run_time = self
                .run_time_h()
                .map_or_else(|| "unlimited".to_owned(), |h| format!("{h:.0}"));
            let _ = writeln!(
                out,
                "  {cells} lithium AA after derating: {have:.1} Wh, about {run_time} h of run time"
            );
            let verdict = if have >= need {
                "  OK"
            } else {
                "  NOT ENOUGH: add cells or cut the load"
            };
            let _ = writeln!(out, "{verdict}");
        }
        if let Some(note) = &self.note {
            let _ = writeln!(out, "{note}");
        }
        out
    }
}

/// The ground station and balloon payload budgets from the build guides.
#[must_use]
pub fn classroom_budgets() -> Vec<Budget> {
    let pi_zero = || Load::new("Raspberry Pi Zero 2 W (0.6-1.2 W, use top)", 1.2);
    let rtl_sdr = || Load::new("RTL-SDR Blog V4", RTL_SDR_W);
    let camera = || vec![pi_zero(), Load::new("Camera module (estimate)", 0.3)];
    vec![
        Budget::new(
            "Ground station: Pi 4 + RTL-SDR, decoding at full CPU load",
            vec![
                Load::new("Raspberry Pi 4, full CPU load (pidramble 6.4 W)", 6.4),
                rtl_sdr(),
            ],
            HOURS,
        )
        .with_note(format!(
            "  A 12 V 20 Ah battery stores {} Wh before converter losses.",
            12 * 20
        )),
        Budget::new(
            "Ground station: Pi Zero 2 W + RTL-SDR (runs this project's station, not SatNOGS)",
            vec![pi_zero(), rtl_sdr()],
            HOURS,
        ),
        Budget::new(
            "Classroom sensors: Arduino Uno (about 40 mA at 5 V) from USB",
            vec![Load::new("Arduino Uno with TMP36 and BH1750", 0.040 * 5.0)],
            HOURS,
        ),
        Budget::new(
            "Balloon payload A: XIAO logger with GPS",
            vec![
                Load::new(
                    "XIAO ESP32-C3, Wi-Fi off (25 mA modem-sleep at 3.3 V)",
                    0.025 * 3.3,
                ),
                Load::new("u-blox MAX-M10S GPS (under 25 mW tracking)", 0.025),
                Load::new("BME280 pressure + temperature (estimate)", 0.003),
                Load::new("microSD writes, average (estimate)", 0.020 * 3.3),
            ],
            HOURS,
        )
        .with_cells(4),
        Budget::new(
            "Balloon payload B: Pi Zero 2 W camera logger",
            camera(),
            HOURS,
        )
        .with_cells(4),
        Budget::new("Balloon payload B with eight cells", camera(), HOURS).with_cells(8),
    ]
}

/// The full report printed by `cubesat-classroom energy-budget`.
#[must_use]
pub fn report() -> String {
    classroom_budgets().iter().map(Budget::render).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derating_applies_regulator_and_cold_losses() {
        assert!((usable_wh(4) - 4.0 * 4.2 * 0.85 * 0.75).abs() < 1e-12);
        assert!((usable_wh(8) - 2.0 * usable_wh(4)).abs() < 1e-12);
    }

    #[test]
    fn camera_payload_needs_eight_cells() {
        let budgets = classroom_budgets();
        let verdicts: Vec<_> = budgets.iter().map(Budget::enough).collect();
        assert_eq!(
            verdicts,
            [None, None, None, Some(true), Some(false), Some(true)]
        );
    }

    #[test]
    fn summary_matches_the_rendered_figures() {
        let budgets = classroom_budgets();
        let a = budgets[3].summary();
        assert_eq!(a.cells, Some(4));
        assert_eq!(a.enough, Some(true));
        assert!((a.need_wh - a.total_w * HOURS * MARGIN).abs() < 1e-12);
        assert!((a.run_time_h.unwrap() - a.usable_wh.unwrap() / a.total_w).abs() < 1e-12);
        assert_eq!(budgets[0].summary().run_time_h, None);
    }

    #[test]
    fn a_pack_with_no_load_never_runs_out() {
        let budget = Budget::new("idle", Vec::new(), 6.0).with_cells(4);
        assert_eq!(budget.run_time_h(), None);
        assert_eq!(budget.enough(), Some(true));
        assert!(budget.render().contains("about unlimited h"));
    }

    #[test]
    fn zero_cells_counts_as_no_pack() {
        let budget = Budget::new("x", vec![Load::new("a", 1.0)], 1.0).with_cells(0);
        assert_eq!(budget.enough(), None);
        assert!(!budget.render().contains("lithium AA"));
    }
}
