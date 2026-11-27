//! Spectrum sweeps with an RTL-SDR (Blog V4 or V3), for the CSC-R1 bench quests.
//!
//! Recording needs `rtl_power` from the RTL-SDR Blog drivers; every other
//! command works on saved CSV files, so a crew can analyse another crew's
//! sweeps without a dongle.
//!
//! All levels are relative dB from the dongle, not dBm. Differences between
//! two sweeps taken with the same dongle, gain and frequencies are meaningful;
//! the absolute numbers are not. `compare` and `response` refuse two sweeps
//! whose receiver, gain, bins or frequency range differ (from the .json labels
//! that `record` writes), unless forced.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::json;

use crate::util::{Fail, csv_line, csv_rows, fail, field, median, sidecar, utc_now, write_json};

/// Speed of light, m/s.
pub const C: f64 = 299_792_458.0;

/// Named bands: start, stop and bin width in Hz. Bins are fine enough to see
/// a satellite channel but coarse enough to sweep in a few seconds.
pub const BANDS: [(&str, f64, f64, f64); 6] = [
    ("fm", 87.0e6, 109.0e6, 25e3),
    ("2m", 143.5e6, 146.5e6, 5e3),
    ("70cm", 434.5e6, 438.5e6, 5e3),
    ("halow", 900.0e6, 930.0e6, 50e3),
    ("filter2m", 80.0e6, 460.0e6, 250e3),
    ("filter70cm", 100.0e6, 960.0e6, 500e3),
];

/// The band's start, stop and bin width.
#[must_use]
pub fn band(name: &str) -> Option<(f64, f64, f64)> {
    BANDS.iter().find(|b| b.0 == name).map(|b| (b.1, b.2, b.3))
}

/// One bin: centre frequency in Hz and mean level in dB.
pub type Point = (f64, f64);

/// Parse rtl_power CSV text into sorted bins, and count the bins dropped because they had
/// zero power in every sweep.
///
/// rtl_power writes one row per hop: date, time, f_low, f_high, step,
/// samples, then one dB value per bin. Repeated sweeps of the same bin are
/// averaged as power, not as dB. A blank field is a missing bin: it keeps its
/// place, so the bins after it keep their frequencies. `-inf` is a bin with
/// zero power and counts as 0 W.
#[must_use]
pub fn parse(text: &str) -> (Vec<Point>, usize) {
    let mut acc: BTreeMap<i64, (f64, u32)> = BTreeMap::new();
    for row in csv_rows(text) {
        if row.len() < 7 || row[0].starts_with('#') {
            continue;
        }
        let (Ok(lo), Ok(step)) = (row[2].trim().parse::<f64>(), row[4].trim().parse::<f64>())
        else {
            continue;
        };
        let values: Result<Vec<f64>, _> = row[6..]
            .iter()
            .map(|v| {
                let v = v.trim();
                if v.is_empty() {
                    Ok(f64::NAN)
                } else {
                    v.parse::<f64>()
                }
            })
            .collect();
        let Ok(values) = values else {
            continue;
        };
        for (i, db) in values.into_iter().enumerate() {
            if db.is_nan() || db == f64::INFINITY {
                continue;
            }
            let fr = (lo + (i as f64 + 0.5) * step).round_ties_even() as i64;
            let entry = acc.entry(fr).or_insert((0.0, 0));
            entry.0 += if db == f64::NEG_INFINITY {
                0.0
            } else {
                10f64.powf(db / 10.0)
            };
            entry.1 += 1;
        }
    }
    let total = acc.len();
    let pts: Vec<Point> = acc
        .into_iter()
        .filter(|(_, (s, _))| *s > 0.0)
        .map(|(fr, (s, n))| (fr as f64, 10.0 * (s / f64::from(n)).log10()))
        .collect();
    let dropped = total - pts.len();
    (pts, dropped)
}

/// Read and parse a sweep file.
///
/// # Errors
/// If the file cannot be read, holds no rtl_power rows, or every bin has zero power.
pub fn load(path: &str) -> Result<Vec<Point>, Fail> {
    let bytes = std::fs::read(path).map_err(|e| Fail(format!("{path}: {e}")))?;
    let (pts, dropped) = parse(&String::from_utf8_lossy(&bytes));
    if pts.is_empty() && dropped == 0 {
        return fail(format!("{path}: no rtl_power rows found"));
    }
    if dropped > 0 {
        eprintln!("{path}: skipped {dropped} bins with zero power in every sweep");
    }
    if pts.is_empty() {
        return fail(format!("{path}: every bin has zero power"));
    }
    Ok(pts)
}

/// Settings that must match before two sweeps can be subtracted.
pub const MUST_MATCH: [&str; 6] = [
    "receiver", "gain_db", "bin_hz", "start_hz", "stop_hz", "ppm",
];

/// The sweep settings in a .json label, if they are complete and valid.
fn settings(meta: &serde_json::Map<String, serde_json::Value>) -> Option<()> {
    let receiver = meta.get("receiver")?.as_str()?;
    if receiver.trim().is_empty() {
        return None;
    }
    let num = |k: &str| {
        meta.get(k)
            .and_then(serde_json::Value::as_f64)
            .filter(|v| v.is_finite())
    };
    for k in &MUST_MATCH[1..] {
        num(k)?;
    }
    let (bin, start, stop) = (num("bin_hz")?, num("start_hz")?, num("stop_hz")?);
    (bin > 0.0 && 0.0 <= start && start < stop).then_some(())
}

/// Stop if two sweeps were taken with different settings (from their .json labels). With
/// `force`, warn instead.
///
/// # Errors
/// If a label is missing or incomplete, or a setting differs, and `force` is off.
#[allow(clippy::float_cmp)] // the recorded settings must match exactly
pub fn check_compatible(a: &str, b: &str, force: bool) -> Result<(), Fail> {
    let (ma, mb) = (sidecar(a), sidecar(b));
    for (path, meta) in [(a, &ma), (b, &mb)] {
        if settings(meta).is_none() {
            let msg = format!("{path}.json is missing or has incomplete or invalid sweep settings");
            if !force {
                return fail(format!(
                    "{msg}. Record with csc-sweep record, or pass --force if you know they match."
                ));
            }
            eprintln!("warning: {msg}");
            return Ok(());
        }
    }
    let show = |m: &serde_json::Map<String, serde_json::Value>, k: &str| {
        m.get(k).map_or_else(|| "None".into(), ToString::to_string)
    };
    let same = |k: &str| match (ma.get(k), mb.get(k)) {
        (Some(x), Some(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => x == y,
            _ => x == y,
        },
        (x, y) => x == y,
    };
    let diff: Vec<String> = MUST_MATCH
        .iter()
        .filter(|k| !same(k))
        .map(|k| format!("{k}: {} vs {}", show(&ma, k), show(&mb, k)))
        .collect();
    if !diff.is_empty() {
        let lines = diff.join("; ");
        if !force {
            return fail(format!(
                "these sweeps can't be compared, settings differ ({lines}). \
                 Re-record with the same settings, or pass --force to override."
            ));
        }
        eprintln!("warning: settings differ ({lines})");
    }
    Ok(())
}

/// Noise floor and strongest bin of a sweep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    /// Number of bins.
    pub bins: usize,
    /// Median level, the noise floor.
    pub floor_median_db: f64,
    /// 10th-percentile level.
    pub floor_p10_db: f64,
    /// Strongest bin level.
    pub peak_db: f64,
    /// Strongest bin frequency.
    pub peak_mhz: f64,
    /// How far the strongest bin stands above the floor.
    pub peak_above_floor_db: f64,
}

/// Summarise a sweep.
///
/// # Panics
/// If `pts` is empty.
#[must_use]
pub fn stats(pts: &[Point]) -> Stats {
    let mut dbs: Vec<f64> = pts.iter().map(|p| p.1).collect();
    dbs.sort_by(f64::total_cmp);
    let peak = max_by(pts, |p| p.1);
    let floor = median(&dbs);
    Stats {
        bins: dbs.len(),
        floor_median_db: floor,
        floor_p10_db: dbs[dbs.len() / 10],
        peak_db: peak.1,
        peak_mhz: peak.0 / 1e6,
        peak_above_floor_db: peak.1 - floor,
    }
}

/// The first item with the largest key, as Python's `max(key=...)`.
fn max_by<T: Copy>(items: &[T], key: impl Fn(&T) -> f64) -> T {
    let mut best = items[0];
    for item in &items[1..] {
        if key(item) > key(&best) {
            best = *item;
        }
    }
    best
}

/// The first item with the smallest key, as Python's `min(key=...)`.
fn min_by<T: Copy>(items: &[T], key: impl Fn(&T) -> f64) -> T {
    let mut best = items[0];
    for item in &items[1..] {
        if key(item) < key(&best) {
            best = *item;
        }
    }
    best
}

/// The bin closest to `f`.
#[must_use]
pub fn nearest(pts: &[Point], f: f64) -> Point {
    min_by(pts, |p| (p.0 - f).abs())
}

/// Match bins of two sweeps that report the same centre frequency. Returns (frequency,
/// level in `a`, level in `b`).
///
/// # Errors
/// If the sweeps share no bins.
pub fn pair(a: &[Point], b: &[Point]) -> Result<Vec<(f64, f64, f64)>, Fail> {
    let bmap: BTreeMap<i64, f64> = b.iter().map(|&(f, db)| (f as i64, db)).collect();
    let out: Vec<(f64, f64, f64)> = a
        .iter()
        .filter_map(|&(f, da)| bmap.get(&(f as i64)).map(|&db| (f, da, db)))
        .collect();
    if out.is_empty() {
        return fail("the sweeps have no shared finite-power bins");
    }
    Ok(out)
}

/// Settings for `record`.
#[derive(Debug, Clone)]
pub struct Record {
    /// Named band, or `None` for start/stop/step.
    pub band: Option<String>,
    /// Start frequency, Hz.
    pub start: Option<f64>,
    /// Stop frequency, Hz.
    pub stop: Option<f64>,
    /// Bin width, Hz.
    pub step: f64,
    /// Fixed tuner gain, dB. Never auto.
    pub gain: f64,
    /// Sweep duration, s.
    pub seconds: u32,
    /// Integration interval, s.
    pub interval: u32,
    /// Frequency correction, ppm.
    pub ppm: i32,
    /// What was connected, in a few words.
    pub label: String,
    /// `hardware bench` or `received RF`.
    pub evidence: String,
    /// The receiver model.
    pub receiver: String,
    /// The CSV to write.
    pub out: String,
    /// The crew measured under 0.5 V DC on the SMA centre pin with this dongle.
    pub dc_checked: bool,
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|dir| {
            let candidate = dir.join(program);
            candidate.is_file() || candidate.with_extension("exe").is_file()
        })
    })
}

/// Run rtl_power and save a labelled sweep.
///
/// # Errors
/// If no band or range was given, rtl_power is missing or fails, or the
/// label file cannot be written.
pub fn record(r: &Record, out: &mut impl Write) -> Result<(), Fail> {
    let (lo, hi, step) = match &r.band {
        Some(name) => band(name).ok_or_else(|| Fail(format!("unknown band {name}")))?,
        None => match (r.start, r.stop) {
            (Some(lo), Some(hi)) => (lo, hi, r.step),
            _ => return fail("record needs --band, or --start and --stop"),
        },
    };
    if !on_path("rtl_power") {
        return fail("rtl_power not found: install the RTL-SDR Blog drivers (see bench/README.md)");
    }
    if !r.dc_checked {
        return fail(
            "measure the DC on the dongle's SMA centre pin first (see bench/README.md, \
             \"Bias tee\"), then add --dc-checked. A dongle can keep its bias tee on even \
             when software turns it off, and DC can damage a load, filter or noise source.",
        );
    }
    if on_path("rtl_biast") {
        // Never feed DC into a load, a stub, a filter or a noise source.
        let r = Command::new("rtl_biast")
            .args(["-b", "0"])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| {
                Fail(format!(
                    "rtl_biast -b 0 failed, so the bias tee may still be on: {e}"
                ))
            })?;
        if !r.status.success() {
            let why = if r.stderr.is_empty() {
                &r.stdout
            } else {
                &r.stderr
            };
            return fail(format!(
                "rtl_biast -b 0 failed, so the bias tee may still be on:\n{}",
                String::from_utf8_lossy(why).trim()
            ));
        }
    } else {
        eprintln!("rtl_biast not found: relying on your DC measurement");
    }
    let mut args = vec![
        "-f".to_string(),
        format!("{}:{}:{}", lo as i64, hi as i64, step as i64),
        "-g".into(),
        py_float(r.gain),
        "-i".into(),
        r.interval.to_string(),
        "-e".into(),
        format!("{}s", r.seconds),
        "-c".into(),
        "0.2".into(),
    ];
    if r.ppm != 0 {
        args.extend(["-p".into(), r.ppm.to_string()]);
    }
    args.push(r.out.clone());
    eprintln!("rtl_power {}", args.join(" "));
    let status = Command::new("rtl_power").args(&args).status()?;
    if !status.success() {
        return fail(format!("rtl_power exited with {status}"));
    }
    let info = json!({
        "label": r.label,
        "evidence": r.evidence,
        "band": r.band.clone().unwrap_or_else(|| "custom".into()),
        "start_hz": lo, "stop_hz": hi, "bin_hz": step,
        "gain_db": r.gain, "seconds": r.seconds, "ppm": r.ppm,
        "receiver": r.receiver,
        "dc_checked": true,
        "utc": utc_now(),
        "levels": "relative dB from the dongle, not dBm",
    });
    write_json(&format!("{}.json", r.out), &info)?;
    writeln!(out, "saved {} and {}.json", r.out, r.out)?;
    Ok(())
}

/// A float as Python's `str()` would print it (`20.0`, `0.5`).
fn py_float(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e16 {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

/// Noise floor and strongest signal of each file.
///
/// # Errors
/// If a file cannot be read.
pub fn summary(files: &[String], out: &mut impl Write) -> Result<(), Fail> {
    for p in files {
        let s = stats(&load(p)?);
        let m = sidecar(p);
        writeln!(
            out,
            "{p}  [{}] {}",
            field(&m, "evidence", "unlabelled"),
            field(&m, "label", "")
        )?;
        writeln!(
            out,
            "  bins {}, floor (median) {:.1} dB, 10th percentile {:.1} dB",
            s.bins, s.floor_median_db, s.floor_p10_db
        )?;
        writeln!(
            out,
            "  strongest {:.1} dB at {:.4} MHz, {:.1} dB above the floor",
            s.peak_db, s.peak_mhz, s.peak_above_floor_db
        )?;
    }
    Ok(())
}

/// Noise-floor change from one sweep to another.
///
/// # Errors
/// If a file cannot be read, the sweeps' settings differ (unless `force`), or they share
/// no bins.
pub fn compare(
    base_path: &str,
    test_path: &str,
    force: bool,
    out: &mut impl Write,
) -> Result<(), Fail> {
    check_compatible(base_path, test_path, force)?;
    let rows = pair(&load(base_path)?, &load(test_path)?)?;
    let diffs: Vec<f64> = rows.iter().map(|r| r.2 - r.1).collect();
    let rise = median(&diffs);
    let worst = max_by(&rows, |r| r.2 - r.1);
    writeln!(
        out,
        "base: {base_path}  {}",
        field(&sidecar(base_path), "label", "")
    )?;
    writeln!(
        out,
        "test: {test_path}  {}",
        field(&sidecar(test_path), "label", "")
    )?;
    writeln!(out, "  {} shared bins", rows.len())?;
    writeln!(
        out,
        "  noise floor change (median of all bins): {rise:+.1} dB"
    )?;
    writeln!(
        out,
        "  largest single-bin rise: {:+.1} dB at {:.4} MHz",
        worst.2 - worst.1,
        worst.0 / 1e6
    )?;
    if rise.abs() < 0.5 {
        writeln!(
            out,
            "  under 0.5 dB: within the run-to-run spread; repeat to be sure"
        )?;
    }
    Ok(())
}

/// Insertion gain of a device: (noise through device) - (noise straight through).
///
/// # Errors
/// If a file cannot be read or written, or the sweeps do not line up.
pub fn response(
    reference: &str,
    dut: &str,
    at: &[f64],
    csv_out: Option<&str>,
    force: bool,
    out: &mut impl Write,
) -> Result<(), Fail> {
    check_compatible(reference, dut, force)?;
    let rows = pair(&load(reference)?, &load(dut)?)?;
    let resp: Vec<Point> = rows.iter().map(|&(fr, r, d)| (fr, d - r)).collect();
    if let Some(path) = csv_out {
        let mut text = csv_line(&["freq_mhz".into(), "response_db".into()]);
        for (fr, g) in &resp {
            text.push_str(&csv_line(&[format!("{:.4}", fr / 1e6), format!("{g:.2}")]));
        }
        std::fs::write(path, text).map_err(|e| Fail(format!("{path}: {e}")))?;
        writeln!(out, "saved {path}")?;
    }
    let best = max_by(&resp, |p| p.1);
    let low = min_by(&resp, |p| p.1);
    writeln!(
        out,
        "  highest response {:+.1} dB at {:.3} MHz",
        best.1,
        best.0 / 1e6
    )?;
    writeln!(
        out,
        "  deepest point    {:+.1} dB at {:.3} MHz",
        low.1,
        low.0 / 1e6
    )?;
    for &f in at {
        let (_, g) = nearest(&resp, f);
        writeln!(out, "  at {:8.3} MHz: {g:+.1} dB", f / 1e6)?;
    }
    let mut gains: Vec<f64> = resp.iter().map(|p| p.1).collect();
    gains.sort_by(f64::total_cmp);
    let floor = median(&gains[..(resp.len() / 20).max(1)]);
    writeln!(
        out,
        "  deepest 5% of bins average {floor:+.1} dB: readings near this are the limit of the method, not the device"
    )?;
    Ok(())
}

/// Quarter-wave open stub length for `freq`, or the velocity factor of a
/// measured stub when `length` (m) is given.
///
/// # Errors
/// Only if `out` cannot be written.
pub fn stub(freq: f64, vf: f64, length: Option<f64>, out: &mut impl Write) -> Result<(), Fail> {
    if let Some(length) = length.filter(|l| *l != 0.0) {
        let vf = length * 4.0 * freq / C;
        writeln!(
            out,
            "a {length:.3} m open stub with its notch at {:.2} MHz has velocity factor {vf:.3}",
            freq / 1e6
        )?;
    } else {
        let l = C / (4.0 * freq) * vf;
        writeln!(
            out,
            "quarter-wave open stub for {:.2} MHz with velocity factor {}: {l:.3} m ({:.1} cm), measured from the tee",
            freq / 1e6,
            py_float(vf),
            l * 100.0
        )?;
        writeln!(
            out,
            "  it also notches odd multiples: {:.1} and {:.1} MHz",
            3.0 * freq / 1e6,
            5.0 * freq / 1e6
        )?;
    }
    Ok(())
}
