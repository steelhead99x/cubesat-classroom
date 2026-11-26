//! Log and summarise radio power for the CSC-R1 bench quests.
//!
//! `record` reads the power-logger sketch over USB serial. `summary` works on
//! saved CSV files: ours, or a Nordic Power Profiler (PPK2) export whose header
//! has `Timestamp(ms)` and `Current(uA)` columns, with `--volts` for the supply.

use std::io::Write;

use serde_json::json;

use crate::util::{Fail, csv_line, csv_rows, fail, field, mean, sidecar, utc_now, write_json};

/// INA226 shunt-voltage register LSB, volts.
pub const SHUNT_LSB_V: f64 = 2.5e-6;
/// INA226 bus-voltage register LSB, volts.
pub const BUS_LSB_V: f64 = 1.25e-3;

/// Settings for `record`.
#[derive(Debug, Clone)]
pub struct Record {
    /// Serial device, such as `/dev/ttyACM0`.
    pub port: String,
    /// How long to log.
    pub minutes: f64,
    /// Shunt resistor, ohms.
    pub shunt: f64,
    /// What was measured, in a few words.
    pub label: String,
    /// The CSV to write.
    pub out: String,
}

/// One logger line `t_ms,shunt_raw,bus_raw` as (t_ms, mA, V).
#[must_use]
pub fn decode(line: &str, shunt_ohm: f64) -> Option<(i64, f64, f64)> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let t: i64 = parts[0].trim().parse().ok()?;
    let sh: i64 = parts[1].trim().parse().ok()?;
    let bus: i64 = parts[2].trim().parse().ok()?;
    let i_ma = sh as f64 * SHUNT_LSB_V / shunt_ohm * 1e3;
    Some((t, i_ma, bus as f64 * BUS_LSB_V))
}

#[cfg(unix)]
mod port {
    #![allow(unsafe_code)]
    //! A raw 115200-baud serial port through termios.

    use std::fs::{File, OpenOptions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;

    pub fn open(path: &str) -> std::io::Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY)
            .open(path)?;
        let fd = file.as_raw_fd();
        // SAFETY: `fd` is an open descriptor owned by `file` for this whole
        // block, and `tio` is a plain struct that tcgetattr fills in.
        unsafe {
            let mut tio: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &raw mut tio) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            libc::cfmakeraw(&raw mut tio);
            libc::cfsetispeed(&raw mut tio, libc::B115200);
            libc::cfsetospeed(&raw mut tio, libc::B115200);
            // Return after 2 s with no data, like pyserial's timeout=2.
            tio.c_cc[libc::VMIN] = 0;
            tio.c_cc[libc::VTIME] = 20;
            if libc::tcsetattr(fd, libc::TCSANOW, &raw const tio) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(file)
    }
}

/// Read the logger sketch for `minutes` and save a CSV plus a `.json` label.
///
/// # Errors
/// If the port or output cannot be opened, or on a platform without termios.
pub fn record(r: &Record, out: &mut impl Write) -> Result<(), Fail> {
    #[cfg(unix)]
    {
        use std::io::Read;
        let mut port = port::open(&r.port).map_err(|e| Fail(format!("{}: {e}", r.port)))?;
        let mut file =
            std::fs::File::create(&r.out).map_err(|e| Fail(format!("{}: {e}", r.out)))?;
        file.write_all(
            csv_line(&[
                "t_ms".into(),
                "current_ma".into(),
                "bus_v".into(),
                "power_mw".into(),
            ])
            .as_bytes(),
        )?;
        let end = std::time::Instant::now() + std::time::Duration::from_secs_f64(r.minutes * 60.0);
        let mut pending: Vec<u8> = Vec::new();
        let mut buf = [0u8; 512];
        let mut n = 0u64;
        while std::time::Instant::now() < end {
            let got = port.read(&mut buf)?;
            pending.extend_from_slice(&buf[..got]);
            while let Some(at) = pending.iter().position(|&b| b == b'\n') {
                let raw: Vec<u8> = pending.drain(..=at).collect();
                let line = String::from_utf8_lossy(&raw).trim().to_string();
                if line.is_empty() {
                    continue;
                }
                if line.starts_with('#') {
                    eprintln!("{line}");
                    continue;
                }
                let Some((t, i_ma, v)) = decode(&line, r.shunt) else {
                    continue;
                };
                file.write_all(
                    csv_line(&[
                        t.to_string(),
                        format!("{i_ma:.3}"),
                        format!("{v:.4}"),
                        format!("{:.2}", i_ma * v),
                    ])
                    .as_bytes(),
                )?;
                n += 1;
                if n.is_multiple_of(200) {
                    eprintln!("{n} readings, now {i_ma:.1} mA at {v:.3} V");
                }
            }
        }
        let info = json!({
            "label": r.label, "evidence": "hardware bench", "meter": "INA226",
            "shunt_ohm": r.shunt, "minutes": r.minutes,
            "utc": utc_now(),
        });
        write_json(&format!("{}.json", r.out), &info)?;
        writeln!(out, "saved {n} readings to {}", r.out)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (r, out);
        fail("record needs macOS or Linux (a termios serial port); use summary on a saved CSV")
    }
}

/// Readings from our logger CSV or a PPK2 export as (t_s, power_mw).
///
/// # Errors
/// If the file is empty, has unknown columns, or is a PPK2 export without
/// `volts`.
pub fn load(path: &str, volts: Option<f64>) -> Result<Vec<(f64, f64)>, Fail> {
    let bytes = std::fs::read(path).map_err(|e| Fail(format!("{path}: {e}")))?;
    let rows = csv_rows(&String::from_utf8_lossy(&bytes));
    let Some((header, body)) = rows.split_first() else {
        return fail(format!("{path}: empty"));
    };
    if body.is_empty() {
        return fail(format!("{path}: empty"));
    }
    let col = |name: &str| header.iter().position(|h| h == name);
    let num = |row: &Vec<String>, i: usize| -> Result<f64, Fail> {
        row.get(i)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .ok_or_else(|| Fail(format!("{path}: unreadable row {}", row.join(","))))
    };
    let mut out = Vec::with_capacity(body.len());
    if let (Some(t), Some(p)) = (col("t_ms"), col("power_mw")) {
        for row in body {
            out.push((num(row, t)? / 1e3, num(row, p)?));
        }
    } else if let (Some(t), Some(c)) = (col("Timestamp(ms)"), col("Current(uA)")) {
        let Some(volts) = volts.filter(|v| *v != 0.0) else {
            return fail(format!(
                "{path}: a PPK2 export has no voltage; pass --volts"
            ));
        };
        for row in body {
            out.push((num(row, t)? / 1e3, num(row, c)? / 1e3 * volts));
        }
    } else {
        return fail(format!("{path}: unknown columns {header:?}"));
    }
    Ok(out)
}

/// Megabytes delivered per file. Each run delivers its own amount, so `megabytes` (one
/// shared figure) is only for a single file; `delivered` gives `FILE=MB` per file.
///
/// # Errors
/// If `megabytes` is given for several files, a `delivered` item names a file not given or
/// isn't `FILE=MB`, or a volume is negative or not finite. Zero is allowed: a run that
/// delivered nothing, which gets no energy per megabyte.
pub fn delivered_map(
    files: &[String],
    megabytes: Option<f64>,
    delivered: &[String],
) -> Result<std::collections::HashMap<String, f64>, Fail> {
    let mut out = std::collections::HashMap::new();
    if let Some(mb) = megabytes {
        if files.len() != 1 {
            return fail(
                "--megabytes is for one file; each run delivers its own amount. \
                 Use --delivered FILE=MB once per file instead.",
            );
        }
        out.insert(files[0].clone(), mb);
    }
    for item in delivered {
        let Some((name, mb)) = item.rsplit_once('=') else {
            return fail(format!(
                "--delivered {item}: expected FILE=MB with FILE one of the files given"
            ));
        };
        if !files.iter().any(|f| f == name) {
            return fail(format!(
                "--delivered {item}: expected FILE=MB with FILE one of the files given"
            ));
        }
        let Ok(mb) = mb.trim().parse::<f64>() else {
            return fail(format!("--delivered {item}: {mb:?} is not a number"));
        };
        out.insert(name.to_string(), mb);
    }
    for (name, mb) in &out {
        if !mb.is_finite() || *mb < 0.0 {
            return fail(format!(
                "{name}: delivered megabytes must be zero or more, not {mb}"
            ));
        }
    }
    Ok(out)
}

/// Average power, energy and energy per megabyte for each file.
///
/// # Errors
/// If a file cannot be read, or the delivered megabytes are refused (see
/// [`delivered_map`]).
pub fn summary(
    files: &[String],
    megabytes: Option<f64>,
    delivered: &[String],
    volts: Option<f64>,
    out: &mut impl Write,
) -> Result<(), Fail> {
    let mb_for = delivered_map(files, megabytes, delivered)?;
    writeln!(
        out,
        "file | label | minutes | average mW | energy J | mJ per MB"
    )?;
    for p in files {
        let pts = load(p, volts)?;
        if pts.len() < 2 {
            writeln!(out, "{p}: too few readings")?;
            continue;
        }
        // Trapezoid integral; skips gaps longer than 1 s instead of bridging them.
        let (mut e_mj, mut dur) = (0.0, 0.0);
        for w in pts.windows(2) {
            let ((t0, p0), (t1, p1)) = (w[0], w[1]);
            let dt = t1 - t0;
            if dt > 0.0 && dt <= 1.0 {
                e_mj += f64::midpoint(p0, p1) * dt;
                dur += dt;
            }
        }
        let avg = if dur > 0.0 {
            e_mj / dur
        } else {
            mean(&pts.iter().map(|x| x.1).collect::<Vec<_>>())
        };
        let label = field(&sidecar(p), "label", "").to_string();
        // The megabytes were delivered over the whole run, gaps included, so
        // energy per megabyte has to cover the same span: the gaps count at
        // the run's average power.
        let span = pts[pts.len() - 1].0 - pts[0].0;
        let gap = (span - dur).max(0.0);
        // With no interval short enough to integrate, nothing was measured:
        // the average is only the mean of isolated readings, too weak to state
        // an efficiency from.
        let per_mb = match mb_for.get(p) {
            Some(&mb) if mb > 0.0 && dur > 0.0 => format!("{:.0}", avg * span / mb),
            _ => "-".into(),
        };
        writeln!(
            out,
            "{p} | {label} | {:.1} | {avg:.0} | {:.1} | {per_mb}",
            dur / 60.0,
            e_mj / 1e3
        )?;
        if dur == 0.0 {
            writeln!(
                out,
                "  {p}: no two readings were within 1 s of each other, so no energy was measured; \
                 mJ per MB is left out. Log at least one reading a second."
            )?;
        } else if gap > 0.0 {
            writeln!(
                out,
                "  {p}: {gap:.1} s of the {span:.1} s run had no readings; mJ per MB counts that time at the average power"
            )?;
        }
    }
    if mb_for.len() < files.len() {
        writeln!(
            out,
            "for energy per megabyte, give each file its delivered megabytes: \
             --delivered FILE=MB (or --megabytes for a single file)"
        )?;
    }
    Ok(())
}
