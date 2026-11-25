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
pub fn usable_wh(cells: u32) -> f64 {
    f64::from(cells) * AA_LITHIUM_WH * REGULATOR_EFFICIENCY * COLD_DERATE
}

/// One electrical load and where its figure comes from (in the name).
#[derive(Clone, Debug, PartialEq)]
pub struct Load {
    pub name: String,
    pub watts: f64,
}

impl Load {
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
    pub title: String,
    pub loads: Vec<Load>,
    pub hours: f64,
    /// Lithium AA cells in the pack, if this budget runs from one.
    pub cells: Option<u32>,
    /// Extra line printed after the budget.
    pub note: Option<String>,
}

impl Budget {
    pub fn new(title: impl Into<String>, loads: Vec<Load>, hours: f64) -> Self {
        Self {
            title: title.into(),
            loads,
            hours,
            cells: None,
            note: None,
        }
    }

    pub fn with_cells(mut self, cells: u32) -> Self {
        self.cells = Some(cells);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Total load, watts.
    pub fn total_w(&self) -> f64 {
        self.loads.iter().map(|load| load.watts).sum()
    }

    /// Energy needed for the run time with the planning margin, watt-hours.
    pub fn need_wh(&self) -> f64 {
        self.total_w() * self.hours * MARGIN
    }

    /// Usable pack energy, if this budget has a pack (zero cells counts as no pack).
    pub fn usable_wh(&self) -> Option<f64> {
        self.cells.filter(|&cells| cells > 0).map(usable_wh)
    }

    /// Whether the pack covers [`Budget::need_wh`], if this budget has a pack.
    pub fn enough(&self) -> Option<bool> {
        self.usable_wh().map(|have| have >= self.need_wh())
    }

    /// The text block printed for this budget, starting with a blank line.
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
            let _ = writeln!(
                out,
                "  {cells} lithium AA after derating: {have:.1} Wh, about {:.0} h of run time",
                have / watts
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
    fn zero_cells_counts_as_no_pack() {
        let budget = Budget::new("x", vec![Load::new("a", 1.0)], 1.0).with_cells(0);
        assert_eq!(budget.enough(), None);
        assert!(!budget.render().contains("lithium AA"));
    }
}
