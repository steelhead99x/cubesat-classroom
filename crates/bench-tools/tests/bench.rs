//! Self-test for the bench tools on synthetic files (desk simulation). No hardware needed.

use std::fmt::Write as _;
use std::path::PathBuf;

use bench_tools::{power, sweep};

/// A fresh scratch folder per test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bench-tools-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path(dir: &std::path::Path, name: &str) -> String {
    dir.join(name).to_string_lossy().into_owned()
}

/// A small deterministic Gaussian source (xorshift64* and Box-Muller).
struct Noise(u64);

impl Noise {
    fn uniform(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let x = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        ((x >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }

    fn gauss(&mut self, sigma: f64) -> f64 {
        let (u, v) = (self.uniform(), self.uniform());
        sigma * (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

/// rtl_power-style rows: 1 MHz hops, `level(f)` dB plus small noise.
fn fake_sweep(
    file: &str,
    lo: f64,
    hi: f64,
    step: f64,
    level: impl Fn(f64) -> f64,
    notch: Option<&dyn Fn(f64) -> f64>,
) {
    let mut rnd = Noise(1);
    let mut text = String::new();
    for _ in 0..3 {
        let mut hop = lo;
        while hop < hi {
            let n = ((1e6f64).min(hi - hop) / step) as usize;
            let mut row = format!(
                "2026-11-26,12:00:00,{},{},{step},100",
                hop as i64,
                (hop + n as f64 * step) as i64
            );
            for i in 0..n {
                let fr = hop + (i as f64 + 0.5) * step;
                let mut db = level(fr) + rnd.gauss(0.3);
                if let Some(notch) = notch {
                    db += notch(fr);
                }
                let _ = write!(row, ",{db:.2}");
            }
            text.push_str(&row);
            text.push_str("\r\n");
            hop += n as f64 * step;
        }
    }
    std::fs::write(file, text).unwrap();
}

/// The .json label `csc-sweep record` writes, for settings checks.
fn label(file: &str, lo: f64, hi: f64, step: f64, gain: f64) {
    let json = serde_json::json!({
        "label": "test", "evidence": "desk simulation", "receiver": "RTL-SDR Blog V4",
        "start_hz": lo, "stop_hz": hi, "bin_hz": step, "gain_db": gain, "ppm": 0,
    });
    std::fs::write(format!("{file}.json"), json.to_string()).unwrap();
}

fn refused(run: impl FnOnce(&mut Vec<u8>) -> Result<(), bench_tools::util::Fail>) -> String {
    let mut buf = Vec::new();
    run(&mut buf).expect_err("refused").to_string()
}

fn output(run: impl FnOnce(&mut Vec<u8>) -> Result<(), bench_tools::util::Fail>) -> String {
    let mut buf = Vec::new();
    run(&mut buf).expect("command ran");
    String::from_utf8(buf).unwrap()
}

fn logger_csv(file: &str, times_ms: impl IntoIterator<Item = i64>, power_mw: &str) {
    let mut text = String::from("t_ms,current_ma,bus_v,power_mw\r\n");
    for t in times_ms {
        let _ = write!(text, "{t},200.000,5.0000,{power_mw}\r\n");
    }
    std::fs::write(file, text).unwrap();
}

fn row_for<'a>(out: &'a str, file: &str) -> &'a str {
    out.lines()
        .find(|line| line.starts_with(&format!("{file} |")))
        .unwrap_or_else(|| panic!("no row for {file} in\n{out}"))
}

#[test]
fn sweeps() {
    let d = scratch("sweeps");
    let (base, test) = (path(&d, "off.csv"), path(&d, "on.csv"));
    fake_sweep(&base, 144e6, 146e6, 5e3, |_| -40.0, None);
    fake_sweep(
        &test,
        144e6,
        146e6,
        5e3,
        |f| -37.0 + if (f - 145.8e6).abs() < 6e3 { 20.0 } else { 0.0 },
        None,
    );
    label(&base, 144e6, 146e6, 5e3, 20.0);
    label(&test, 144e6, 146e6, 5e3, 20.0);
    let pts = sweep::load(&base).unwrap();
    assert_eq!(pts.len(), 400);
    assert!((sweep::stats(&pts).floor_median_db + 40.0).abs() < 0.3);
    let out = output(|o| sweep::compare(&base, &test, false, o));
    assert!(out.contains("+3.0 dB"), "{out}");
    assert!(
        out.contains("145.7975") || out.contains("145.8025"),
        "{out}"
    );
    let out = output(|o| sweep::summary(std::slice::from_ref(&test), o));
    assert!(
        out.contains("145.7975") || out.contains("145.8025"),
        "{out}"
    );
}

#[test]
fn response_and_stub() {
    let d = scratch("response");
    let (reference, dut) = (path(&d, "ref.csv"), path(&d, "dut.csv"));
    // The dongle's own slope cancels.
    let flat = |f: f64| -30.0 - 3.0 * (f - 80e6) / 300e6;
    fake_sweep(&reference, 80e6, 120e6, 250e3, flat, None);
    // A notch at 98 MHz, 25 dB deep.
    let notch = |f: f64| -25.0 * (-((f - 98e6) / 1.5e6).powi(2)).exp();
    fake_sweep(&dut, 80e6, 120e6, 250e3, flat, Some(&notch));
    label(&reference, 80e6, 120e6, 250e3, 20.0);
    label(&dut, 80e6, 120e6, 250e3, 20.0);
    let out = output(|o| sweep::response(&reference, &dut, &[90e6, 98e6], None, false, o));
    let deepest = out.lines().find(|l| l.contains("deepest point")).unwrap();
    let mhz: f64 = deepest
        .split(" at ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!((mhz - 98.0).abs() < 0.5, "{deepest}");
    let line = out.lines().find(|l| l.contains("at   98.000")).unwrap();
    let db: f64 = line
        .split(':')
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!((-26.5..-23.5).contains(&db), "{line}");

    let out = output(|o| sweep::stub(98e6, 0.66, None, o));
    assert!(out.contains("0.505 m"), "{out}");
    let out = output(|o| sweep::stub(98e6, 0.66, Some(0.505), o));
    assert!(out.contains("0.66"), "{out}");

    let csv = path(&d, "resp.csv");
    let _ = output(|o| sweep::response(&reference, &dut, &[], Some(&csv), false, o));
    let saved = std::fs::read_to_string(&csv).unwrap();
    assert!(saved.starts_with("freq_mhz,response_db\r\n"), "{saved}");
}

#[test]
fn power_summary() {
    let d = scratch("power");
    let p = path(&d, "p.csv");
    // 60 s at 50 ms, 500 mW.
    let mut text = String::from("t_ms,current_ma,bus_v,power_mw\r\n");
    for i in 0..=1200 {
        let _ = write!(text, "{},100,5.0,500.0\r\n", i * 50);
    }
    std::fs::write(&p, text).unwrap();
    let out = output(|o| power::summary(std::slice::from_ref(&p), Some(10.0), &[], None, o));
    assert!(out.contains("| 1.0 | 500 | 30.0 | 3000"), "{out}");

    let ppk = path(&d, "ppk.csv");
    let mut text = String::from("Timestamp(ms),Current(uA)\r\n");
    for i in 0..=10_000 {
        // 1 ms steps, 20 mA.
        let _ = write!(text, "{},20000\r\n", f64::from(i) * 0.01 * 100.0);
    }
    std::fs::write(&ppk, text).unwrap();
    let out = output(|o| power::summary(std::slice::from_ref(&ppk), None, &[], Some(3.3), o));
    assert!(out.contains("| 66 |"), "{out}");
    let mut buf = Vec::new();
    let refused =
        power::summary(std::slice::from_ref(&ppk), None, &[], None, &mut buf).unwrap_err();
    assert!(refused.0.contains("pass --volts"));
}

#[test]
fn power_per_mb_covers_gaps() {
    // A 10 s gap was once left out of the energy while the megabytes covered the
    // whole run, so energy per MB came out low. 1 W steady for 30 s with a 10 s
    // hole in the readings: the run used about 30 J, so 30 MB is about 1000 mJ
    // per MB, not 667.
    let d = scratch("gap");
    let p = path(&d, "gap.csv");
    logger_csv(
        &p,
        (0..=10_000)
            .step_by(100)
            .chain((20_000..=30_000).step_by(100)),
        "1000.00",
    );
    let out = output(|o| power::summary(std::slice::from_ref(&p), Some(30.0), &[], None, o));
    assert_eq!(
        row_for(&out, &p).rsplit(" | ").next().unwrap().trim(),
        "1000",
        "{out}"
    );
    assert!(
        out.contains("10.0 s of the 30.0 s run had no readings"),
        "{out}"
    );
}

#[test]
fn power_per_mb_needs_measured_time() {
    // Three readings 10 s apart once gave 0.0 J next to 1000 mJ per MB, though
    // no interval was integrated. With nothing measured, the efficiency is left out.
    let d = scratch("sparse");
    let p = path(&d, "sparse.csv");
    logger_csv(&p, [0, 10_000, 20_000], "1000.00");
    let out = output(|o| power::summary(std::slice::from_ref(&p), Some(20.0), &[], None, o));
    assert_eq!(
        row_for(&out, &p).rsplit(" | ").next().unwrap().trim(),
        "-",
        "{out}"
    );
    assert!(out.contains("no energy was measured"), "{out}");
}

#[test]
fn logger_lines_decode() {
    // 40000 counts across 0.1 ohm is 1000 mA; 4000 bus counts is 5 V.
    let (t, ma, v) = power::decode("1234,40000,4000", 0.1).unwrap();
    assert_eq!(t, 1234);
    assert!((ma - 1000.0).abs() < 1e-9 && (v - 5.0).abs() < 1e-9);
    assert!(power::decode("# INA226 ready", 0.1).is_none());
    assert!(power::decode("1,2", 0.1).is_none());
}

#[test]
fn compare_refuses_mismatched_settings() {
    // Review finding (ported from #59): 20 dB and 30 dB sweeps gave a "+10.0 dB" noise rise
    // with no warning.
    let d = scratch("mismatch");
    let (a, b) = (path(&d, "g20.csv"), path(&d, "g30.csv"));
    fake_sweep(&a, 144e6, 146e6, 5e3, |_| -40.0, None);
    fake_sweep(&b, 144e6, 146e6, 5e3, |_| -30.0, None);
    label(&a, 144e6, 146e6, 5e3, 20.0);
    label(&b, 144e6, 146e6, 5e3, 30.0);
    assert!(refused(|o| sweep::compare(&a, &b, false, o)).contains("gain_db"));
    assert!(refused(|o| sweep::response(&a, &b, &[], None, false, o)).contains("gain_db"));
    let out = output(|o| sweep::compare(&a, &b, true, o)); // an explicit override still works
    assert!(out.contains("+10.0 dB"), "{out}");
    std::fs::remove_file(format!("{b}.json")).unwrap();
    assert!(refused(|o| sweep::compare(&a, &b, false, o)).contains("incomplete or invalid"));
}

#[test]
fn sparse_bins_and_incomplete_labels() {
    let d = scratch("sparse");
    let (a, b) = (path(&d, "a.csv"), path(&d, "b.csv"));
    for (file, levels) in [(&a, "-10,-inf,-inf,-10"), (&b, "-inf,-50,-50,-inf")] {
        std::fs::write(file, format!("date,time,0,40,10,1,{levels}\n")).unwrap();
        std::fs::write(format!("{file}.json"), r#"{"label": "incomplete"}"#).unwrap();
    }
    assert!(refused(|o| sweep::compare(&a, &b, false, o)).contains("incomplete or invalid"));
    assert!(
        refused(|o| sweep::response(&a, &b, &[], None, false, o)).contains("incomplete or invalid")
    );
    label(&a, 0.0, 40.0, 10.0, 20.0);
    label(&b, 0.0, 40.0, 10.0, 20.0);
    assert!(refused(|o| sweep::compare(&a, &b, false, o)).contains("no shared"));
}

#[test]
fn zero_power_bins() {
    // rtl_power writes -inf for a zero-power bin: it counts as 0 W, and a bin with zero
    // power in every sweep is skipped.
    let d = scratch("inf");
    let p = path(&d, "inf.csv");
    std::fs::write(
        &p,
        "2026-11-26,12:00:00,144000000,144020000,5000,100,-40.0,-inf,-41.0,-39.0\r\n\
         2026-11-26,12:00:02,144000000,144020000,5000,100,-40.0,-inf,-inf,-39.0\r\n",
    )
    .unwrap();
    let pts = sweep::load(&p).unwrap();
    assert_eq!(pts.len(), 3, "{pts:?}");
    let third = pts
        .iter()
        .find(|p| (p.0 - 144_012_500.0).abs() < 0.5)
        .unwrap()
        .1;
    assert!(
        (third - (-41.0 - 10.0 * 2f64.log10())).abs() < 0.01,
        "{pts:?}"
    );
    let out = output(|o| sweep::summary(std::slice::from_ref(&p), o));
    assert!(out.contains("floor (median)"), "{out}");
}

#[test]
fn blank_bins_keep_their_place() {
    // Review finding: blank fields were dropped before numbering the bins, so every bin
    // after a blank one was reported 5 kHz low.
    let (pts, _) = sweep::parse("d,t,144000000,144020000,5000,1,-40,,-41,-42\n");
    let freqs: Vec<f64> = pts.iter().map(|p| p.0).collect();
    assert_eq!(
        freqs,
        [144_002_500.0, 144_012_500.0, 144_017_500.0],
        "{pts:?}"
    );
    assert!((pts[1].1 + 41.0).abs() < 1e-9);
}

#[cfg(unix)]
#[test]
fn record_needs_dc_check_and_working_biast() {
    use std::os::unix::fs::PermissionsExt;
    // Review finding (ported from #59): a failed bias-tee disable was ignored and the sweep
    // went ahead.
    let d = scratch("record");
    let bin = d.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let tool = |name: &str, body: &str| {
        let p = bin.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    tool(
        "rtl_power",
        r#"for a; do last="$a"; done; echo "2026-11-26, 12:00:00, 144000000, 144010000, 5000, 1, -40, -40" > "$last""#,
    );
    let out = path(&d, "s.csv");
    let record = |extra: &[&str]| {
        let _ = std::fs::remove_file(&out);
        let path_var = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let r = std::process::Command::new(env!("CARGO_BIN_EXE_csc-sweep"))
            .args(["record", "--band", "2m", "--label", "load", "--out", &out])
            .args(extra)
            .env("PATH", path_var)
            .output()
            .unwrap();
        (
            r.status.success(),
            std::path::Path::new(&out).exists(),
            String::from_utf8_lossy(&r.stderr).into_owned(),
        )
    };
    tool("rtl_biast", "exit 0");
    let (ok, made, err) = record(&[]);
    assert!(!ok && !made && err.contains("--dc-checked"), "{err}");
    let (ok, made, err) = record(&["--dc-checked"]);
    assert!(ok && made, "{err}");
    tool("rtl_biast", "echo 'usb_claim_interface error' >&2; exit 1");
    let (ok, made, err) = record(&["--dc-checked"]);
    assert!(
        !ok && !made && err.contains("bias tee may still be on"),
        "{err}"
    );
}

#[test]
fn power_megabytes_per_file() {
    // Review finding (ported from #59): one --megabytes figure was applied to every file.
    let d = scratch("per-file");
    let (a, b) = (path(&d, "a.csv"), path(&d, "b.csv"));
    logger_csv(&a, (0..=1200).map(|i| i * 50), "500.0");
    logger_csv(&b, (0..=1200).map(|i| i * 50), "1000.0");
    let files = [a.clone(), b.clone()];
    assert!(refused(|o| power::summary(&files, Some(10.0), &[], None, o)).contains("one file"));
    let out = output(|o| {
        power::summary(
            &files,
            None,
            &[format!("{a}=10"), format!("{b}=60")],
            None,
            o,
        )
    });
    assert!(row_for(&out, &a).ends_with("| 3000"), "{out}"); // 30 J / 10 MB
    assert!(row_for(&out, &b).ends_with("| 1000"), "{out}"); // 60 J / 60 MB
    assert!(
        refused(|o| power::summary(&files, None, &["other.csv=5".into()], None, o))
            .contains("one of the files")
    );
    // A negative or nonfinite volume gave a negative or meaningless mJ/MB.
    let one = std::slice::from_ref(&a);
    for bad in ["-1", "nan", "inf", "-inf"] {
        let item = format!("{a}={bad}");
        assert!(
            refused(|o| power::summary(one, None, std::slice::from_ref(&item), None, o))
                .contains("zero or more"),
            "{bad}"
        );
    }
    // Zero is a run that delivered nothing: allowed, with no per-MB figure.
    let out = output(|o| power::summary(one, None, &[format!("{a}=0")], None, o));
    assert!(row_for(&out, &a).ends_with("| -"), "{out}");
}
