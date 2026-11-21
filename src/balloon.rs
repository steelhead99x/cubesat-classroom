use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

use super::{read_integer, unix_ms};

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
        let pressure = read_integer::<u32>(&pressure_path).and_then(|pa| {
            ensure!(pa > 0, "pressure must be positive");
            Ok(pa)
        });
        let temperature = read_integer::<i32>(&temperature_path);
        let status = match (&pressure, &temperature) {
            (Ok(_), Ok(_)) => "ok",
            (Err(_), Ok(_)) => "pressure_error",
            (Ok(_), Err(_)) => "temperature_error",
            (Err(_), Err(_)) => "both_error",
        };
        let cell = |value: Option<String>| value.unwrap_or_default();
        writeln!(
            file,
            "{sequence},{timestamp},{elapsed},{},{},{status}",
            cell(pressure.ok().map(|value| value.to_string())),
            cell(temperature.ok().map(|value| value.to_string()))
        )?;
        file.flush()?;
    }
    Ok(())
}
