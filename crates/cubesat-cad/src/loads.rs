//! Launch and handling loads on the printed frame: hand calculations, not FEA.
//!
//! Every result is a margin of safety, MS = allowable / (FS x stress) - 1, which
//! must be at least zero. The numbers are estimates for a classroom design review;
//! a shake or drop test of a printed unit is the real evidence.
//!
//! Sources:
//! - GEVS (GSFC-STD-7000A) component random vibration, 14.1 Grms qualification.
//!   The CDS (Rev 14.1 §3) says to use GEVS when a launch provider gives no levels.
//! - Miles' equation for the response of one mode: Grms = sqrt(pi/2 x f x Q x PSD(f)).
//! - NASA-STD-5001B: an ultimate factor of 2.0 for discontinuities in composite and
//!   bonded structures. A print is layers bonded together, so we use 2.0 everywhere.
//! - JAXA J-SSOD: fundamental frequency at least 100 Hz with the rails held.
//! - Classroom rocket (project rockoon doc): AeroTech F39T, thrust-to-weight about
//!   11:1, so about 10 g of boost. Ejection and landing shock is an assumption
//!   (50 g) to replace with a measured value from a flight accelerometer.

use crate::design::Design;
use crate::parts::Assembly;

/// Ultimate factor of safety (NASA-STD-5001B, discontinuity in bonded structure).
pub const FS: f64 = 2.0;
/// Standard gravity, m/s2.
pub const G: f64 = 9.81;
/// Printed PLA allowables, MPa. Assumptions: tests report about 58 MPa along the
/// raster and 31 MPa across it (Fayazbakhsh 2019), and much less between layers.
/// We round down and halve for shear.
pub const PLA_IN_LAYER: f64 = 30.0;
pub const PLA_INTERLAYER: f64 = 15.0;
pub const PLA_E: f64 = 2300.0;
pub const PLA_NU: f64 = 0.35;
/// Solid PLA density, kg/mm3 (an upper bound: more mass is more load).
pub const PLA_KG_PER_MM3: f64 = 1.24e-6;
pub const UNO_KG: f64 = 0.025;
pub const NANO_KG: f64 = 0.005;

/// Quasi-static design load on each axis, g (a common CubeSat design requirement).
pub const QUASI_STATIC_G: f64 = 20.0;
/// Classroom rocket boost, g (F39T, thrust-to-weight about 11).
pub const ROCKET_BOOST_G: f64 = 10.0;
/// Ejection and landing shock, g. An assumption: measure it.
pub const ROCKET_SHOCK_G: f64 = 50.0;
/// Amplification at resonance for Miles' equation.
pub const Q: f64 = 10.0;
/// Lowest fundamental frequency allowed, Hz (JAXA J-SSOD).
pub const MIN_FREQ_HZ: f64 = 100.0;

/// The classroom envelope: the worst of the quasi-static load, rocket boost and
/// rocket shock.
pub fn classroom_g() -> f64 {
    QUASI_STATIC_G.max(ROCKET_BOOST_G).max(ROCKET_SHOCK_G)
}

/// GEVS component qualification random-vibration level at `f` Hz, g2/Hz.
pub fn gevs_psd(f: f64) -> f64 {
    let f = f.clamp(20.0, 2000.0);
    if f < 50.0 {
        0.026 * (f / 20.0).powi(2) // +6 dB/octave
    } else if f <= 800.0 {
        0.16
    } else {
        0.16 * (800.0 / f).powi(2) // -6 dB/octave
    }
}

/// Peak (3 sigma) response of a mode at `f` Hz to the GEVS level, g.
pub fn gevs_peak_g(f: f64) -> f64 {
    3.0 * (std::f64::consts::FRAC_PI_2 * f * Q * gevs_psd(f)).sqrt()
}

/// Natural frequency of a mass on a spring, Hz. `k` in N/mm, `m` in kg.
pub fn frequency(k: f64, m: f64) -> f64 {
    (k * 1000.0 / m).sqrt() / (2.0 * std::f64::consts::PI)
}

/// One structural item: its stress for each g of acceleration, what it is allowed,
/// and the frequency of the mode that loads it.
#[derive(Clone, Debug)]
pub struct Item {
    pub id: &'static str,
    pub what: &'static str,
    pub mpa_per_g: f64,
    pub allowable: f64,
    pub freq_hz: f64,
    pub basis: String,
}

impl Item {
    pub fn margin(&self, g: f64) -> f64 {
        self.allowable / (FS * self.mpa_per_g * g) - 1.0
    }
    pub fn margin_classroom(&self) -> f64 {
        self.margin(classroom_g())
    }
    pub fn margin_gevs(&self) -> f64 {
        self.margin(gevs_peak_g(self.freq_hz))
    }
}

/// A fundamental-frequency estimate.
#[derive(Clone, Debug)]
pub struct Mode {
    pub id: &'static str,
    pub what: &'static str,
    pub hz: f64,
    pub basis: String,
}

/// Masses carried by the payload columns and by the -Z end plate, kg.
pub struct Masses {
    pub payload: f64,
    pub payload_plate: f64,
    pub end_bottom: f64,
}

impl Masses {
    pub fn new(a: &Assembly) -> Self {
        let kg = |m: &manifold_csg::Manifold| m.volume() * PLA_KG_PER_MM3;
        let payload_plate = kg(&a.payload);
        Self {
            payload: payload_plate + kg(&a.strap) + UNO_KG + NANO_KG,
            payload_plate,
            end_bottom: kg(&a.end_bottom),
        }
    }
}

fn circle_i(d: f64) -> f64 {
    std::f64::consts::PI * d.powi(4) / 64.0
}

/// Shear area of a split peg's lip where it bears on the part it holds, mm2.
fn lip_shear_area(d: &Design) -> f64 {
    (std::f64::consts::PI * d.peg_shank - 2.0 * d.peg_slot) * d.peg_lip
}

/// Fundamental-frequency estimates.
pub fn modes(d: &Design, m: &Masses) -> Vec<Mode> {
    let mut out = Vec::new();

    // Payload on four columns, each a cantilever from the -Z end plate.
    let l = d.column_length;
    let k_col = 3.0 * PLA_E * circle_i(d.column_d) / l.powi(3);
    out.push(Mode {
        id: "FREQ-payload",
        what: "Payload sways on its four columns",
        hz: frequency(4.0 * k_col, m.payload),
        basis: format!(
            "4 cantilevers, {:.0} mm diameter, {l:.0} mm long, {:.0} g payload",
            d.column_d,
            m.payload * 1000.0
        ),
    });

    // -Z end plate bending with the payload on it: a simply supported square plate
    // between the snap rows, centre load (Timoshenko: w = 0.0116 P a2 / D).
    let a = 2.0 * d.peg_y.max(d.cover_peg[0]);
    let flex = PLA_E * d.end_t.powi(3) / (12.0 * (1.0 - PLA_NU * PLA_NU));
    let k_plate = flex / (0.0116 * a * a);
    let mass = m.payload + 0.5 * m.end_bottom;
    out.push(Mode {
        id: "FREQ-end-plate",
        what: "-Z end plate flexes under the payload",
        hz: frequency(k_plate, mass),
        basis: format!(
            "{:.1} mm plate, {a:.0} mm between snaps, {:.0} g on it",
            d.end_t,
            mass * 1000.0
        ),
    });

    // Payload plate bending between the columns, boards in the middle.
    let span = d.columns.iter().map(|c| c[0]).fold(f64::MIN, f64::max)
        - d.columns.iter().map(|c| c[0]).fold(f64::MAX, f64::min);
    let i = d.plate[1] * d.plate_t.powi(3) / 12.0;
    let k = 48.0 * PLA_E * i / span.powi(3);
    let mass = UNO_KG + NANO_KG + 0.5 * m.payload_plate;
    out.push(Mode {
        id: "FREQ-payload-plate",
        what: "Payload plate flexes between its columns",
        hz: frequency(k, mass),
        basis: format!(
            "{:.1} mm plate as a beam over {span:.0} mm, {:.0} g on it",
            d.plate_t,
            mass * 1000.0
        ),
    });

    // Uno on four posts (the lower, wider part dominates).
    let k_post = 3.0 * PLA_E * circle_i(7.0) / d.uno_post_h.powi(3);
    out.push(Mode {
        id: "FREQ-uno",
        what: "Uno sways on its posts",
        hz: frequency(4.0 * k_post, UNO_KG),
        basis: format!("4 posts, 7 mm diameter, {:.0} mm tall", d.uno_post_h),
    });

    // Nano on its two rails, swaying across them.
    let rail_i = 30.0 * 3.0f64.powi(3) / 12.0;
    let k_rail = 3.0 * PLA_E * rail_i / d.nano_rail_h.powi(3);
    out.push(Mode {
        id: "FREQ-nano",
        what: "Nano sways on its rails",
        hz: frequency(2.0 * k_rail, NANO_KG + 0.002),
        basis: format!("2 rails, 3 x 30 mm, {:.0} mm tall", d.nano_rail_h),
    });
    out
}

/// Structural items with their stress per g.
pub fn items(d: &Design, m: &Masses, modes: &[Mode]) -> Vec<Item> {
    let hz = |id: &str| {
        modes
            .iter()
            .find(|f| f.id == id)
            .map_or(MIN_FREQ_HZ, |f| f.hz)
    };
    let newtons = |kg: f64| kg * G; // per g
    let mut out = Vec::new();

    // Columns bending at the root under a sideways load. They print standing up, so
    // the root bends across the layers: interlayer allowable.
    let per_col = newtons(m.payload) / 4.0;
    let moment = per_col * d.column_length;
    let sigma = moment * (d.column_d / 2.0) / circle_i(d.column_d);
    out.push(Item {
        id: "LOAD-columns",
        what: "Payload columns bending at the root (sideways load)",
        mpa_per_g: sigma,
        allowable: PLA_INTERLAYER,
        freq_hz: hz("FREQ-payload"),
        basis: format!(
            "{:.0} g payload on 4 columns, cantilever, interlayer",
            m.payload * 1000.0
        ),
    });

    // Column snaps holding the payload plate when it is pulled off (+Z load).
    let tau = per_col / lip_shear_area(d);
    out.push(Item {
        id: "LOAD-column-snaps",
        what: "Payload-plate snaps: peg lip shear (plate pulled off)",
        mpa_per_g: tau,
        allowable: PLA_INTERLAYER / 2.0,
        freq_hz: hz("FREQ-payload"),
        basis: format!("{:.1} mm2 lip in shear, 4 pegs", lip_shear_area(d)),
    });

    // -Z end plate and payload pulled out of the frame (load towards -Z): the eight
    // end-plate snaps carry it.
    let snaps = d.all_peg_points().len() as f64;
    let pull = newtons(m.payload + m.end_bottom) / snaps;
    out.push(Item {
        id: "LOAD-end-snaps",
        what: "-Z end-plate snaps: peg lip shear (plate pulled out)",
        mpa_per_g: pull / lip_shear_area(d),
        allowable: PLA_INTERLAYER / 2.0,
        freq_hz: hz("FREQ-end-plate"),
        basis: format!("{snaps:.0} snaps share the end plate and payload"),
    });

    // The end wall (lug) each snap pulls on: a cantilever from the web, one boss
    // wide. Panels print web down, so the lug root bends across the layers.
    let arm = d.web_inner_y() - d.peg_y.max(d.cover_peg[0]);
    let sigma = 6.0 * pull * arm / (d.boss_d * d.lug_t * d.lug_t);
    out.push(Item {
        id: "LOAD-lugs",
        what: "Panel and cover end walls bending where the snaps pull",
        mpa_per_g: sigma,
        allowable: PLA_INTERLAYER,
        freq_hz: hz("FREQ-end-plate"),
        basis: format!(
            "{arm:.1} mm arm, {:.0} x {:.1} mm section, interlayer",
            d.boss_d, d.lug_t
        ),
    });

    // Uno locating pins in shear (sideways load), printed standing up.
    let pin_area = std::f64::consts::PI * d.uno_pin * d.uno_pin / 4.0;
    out.push(Item {
        id: "LOAD-uno-pins",
        what: "Uno locating pins in shear (sideways load)",
        mpa_per_g: newtons(UNO_KG) / 4.0 / pin_area,
        allowable: PLA_INTERLAYER / 2.0,
        freq_hz: hz("FREQ-uno"),
        basis: format!("4 pins, {:.1} mm diameter, interlayer shear", d.uno_pin),
    });

    // Uno clip lips holding the board down (load towards -Z lifts the board).
    let lip_root = d.clip_w * (d.clip_lip + 1.5);
    out.push(Item {
        id: "LOAD-uno-clips",
        what: "Uno clip lips in shear (board lifting)",
        mpa_per_g: newtons(UNO_KG) / 2.0 / lip_root,
        allowable: PLA_IN_LAYER / 2.0,
        freq_hz: hz("FREQ-uno"),
        basis: format!("2 lips, {lip_root:.0} mm2 root each"),
    });

    // Nano strap pegs holding the Nano and strap down.
    out.push(Item {
        id: "LOAD-nano-strap",
        what: "Nano strap snaps: peg lip shear (board lifting)",
        mpa_per_g: newtons(NANO_KG + 0.002) / 2.0 / lip_shear_area(d),
        allowable: PLA_INTERLAYER / 2.0,
        freq_hz: hz("FREQ-nano"),
        basis: "2 pegs".into(),
    });

    out
}
