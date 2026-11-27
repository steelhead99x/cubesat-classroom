use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use super::live::read_fresh_integer;
use super::unix_ms;

/// One channel's reading this second: a fresh value, an old one, or none.
enum Reading<T> {
    Fresh(T),
    Stale,
    Error,
}

impl<T: ToString> Reading<T> {
    /// Reads a sensor file with the station's freshness check: a file not rewritten within the
    /// station's age limit is stale, and its old value is not logged as if it were new.
    fn read(path: &std::path::Path, now: u64, check: impl Fn(&T) -> bool) -> Self
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        match read_fresh_integer::<T>(path, now) {
            Ok((v, _)) if check(&v) => Self::Fresh(v),
            Ok(_) => Self::Error,
            Err(e) if e.to_string().contains(" is stale:") => Self::Stale,
            Err(_) => Self::Error,
        }
    }

    fn cell(&self) -> String {
        match self {
            Self::Fresh(v) => v.to_string(),
            _ => String::new(),
        }
    }

    fn problem(&self, name: &str) -> Option<String> {
        match self {
            Self::Fresh(_) => None,
            Self::Stale => Some(format!("{name}_stale")),
            Self::Error => Some(format!("{name}_error")),
        }
    }
}

/// `ok`, or each channel's problem joined with `+` (both failing to read stays `both_error`).
fn status(pressure: &Reading<u32>, temperature: &Reading<i32>) -> String {
    if matches!((pressure, temperature), (Reading::Error, Reading::Error)) {
        return "both_error".into();
    }
    let problems: Vec<_> = [
        pressure.problem("pressure"),
        temperature.problem("temperature"),
    ]
    .into_iter()
    .flatten()
    .collect();
    if problems.is_empty() {
        "ok".into()
    } else {
        problems.join("+")
    }
}

pub(crate) async fn log(sensor_dir: PathBuf, output: PathBuf, samples: u32) -> Result<()> {
    let pressure_path = sensor_dir.join("pressure_pa");
    let temperature_path = sensor_dir.join("temperature_mc");
    let mut file = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .with_context(|| format!("create new log at {}", output.display()))?,
    );
    writeln!(
        file,
        "sequence,captured_unix_ms,elapsed_ms,pressure_pa,temperature_mc,status"
    )?;
    file.flush()?;
    let start = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    for sequence in 0..samples {
        tokio::select! {
            _ = tick.tick() => {}
            signal = tokio::signal::ctrl_c() => { signal?; break; }
        }
        let timestamp = unix_ms()?;
        let elapsed = start.elapsed().as_millis();
        let now = timestamp;
        let pressure = Reading::<u32>::read(&pressure_path, now, |pa| *pa > 0);
        let temperature = Reading::<i32>::read(&temperature_path, now, |_| true);
        writeln!(
            file,
            "{sequence},{timestamp},{elapsed},{},{},{}",
            pressure.cell(),
            temperature.cell(),
            status(&pressure, &temperature)
        )?;
        file.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_sensor_files_are_marked_and_left_empty() {
        let dir = std::env::temp_dir().join(format!("balloon-stale-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (p, t) = (dir.join("pressure_pa"), dir.join("temperature_mc"));
        std::fs::write(&p, "101325\n").unwrap();
        std::fs::write(&t, "21500\n").unwrap();
        let now = unix_ms().unwrap();
        let fresh = (
            Reading::<u32>::read(&p, now, |v| *v > 0),
            Reading::<i32>::read(&t, now, |_| true),
        );
        assert_eq!(status(&fresh.0, &fresh.1), "ok");
        assert_eq!(fresh.0.cell(), "101325");

        // The same files a day later: their values must not be logged as new readings.
        let later = now + 24 * 3600 * 1000;
        let old = (
            Reading::<u32>::read(&p, later, |v| *v > 0),
            Reading::<i32>::read(&t, later, |_| true),
        );
        assert_eq!(status(&old.0, &old.1), "pressure_stale+temperature_stale");
        assert_eq!((old.0.cell(), old.1.cell()), (String::new(), String::new()));

        std::fs::write(&p, "0\n").unwrap();
        std::fs::remove_file(&t).unwrap();
        let bad = (
            Reading::<u32>::read(&p, now, |v| *v > 0),
            Reading::<i32>::read(&t, now, |_| true),
        );
        assert_eq!(status(&bad.0, &bad.1), "both_error");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
