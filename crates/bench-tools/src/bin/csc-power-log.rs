//! Log and summarise radio power for the CSC-R1 bench quests.
//!
//! ```text
//! csc-power-log record --port /dev/ttyACM0 --minutes 10 --label "C5, 5 GHz, streaming" --out c5-5g.csv
//! csc-power-log summary c5-5g.csv --megabytes 210
//! csc-power-log summary s3-24.csv c5-5g.csv --delivered s3-24.csv=180 --delivered c5-5g.csv=210
//! csc-power-log summary *.csv
//! ```
//!
//! `summary` also reads a Nordic PPK2 CSV export, with `--volts` for the supply.

use std::process::ExitCode;

use bench_tools::power;
use clap::{Parser, Subcommand};

/// Log and summarise radio power for the CSC-R1 bench quests.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Read the power-logger sketch over USB serial and save a CSV
    Record {
        #[arg(long)]
        port: String,
        #[arg(long, default_value_t = 10.0)]
        minutes: f64,
        /// Shunt resistor in ohms
        #[arg(long, default_value_t = 0.1)]
        shunt: f64,
        #[arg(long)]
        label: String,
        #[arg(long)]
        out: String,
    },
    /// Average mW, energy, and energy per megabyte
    Summary {
        #[arg(required = true)]
        files: Vec<String>,
        /// Data delivered during the run (one file only)
        #[arg(long)]
        megabytes: Option<f64>,
        /// Data delivered during that file's run, as FILE=MB; repeat once per file
        #[arg(long, value_name = "FILE=MB")]
        delivered: Vec<String>,
        /// Supply voltage, for PPK2 exports
        #[arg(long)]
        volts: Option<f64>,
    },
}

fn main() -> ExitCode {
    let out = &mut std::io::stdout();
    let result = match Args::parse().cmd {
        Cmd::Record {
            port,
            minutes,
            shunt,
            label,
            out: file,
        } => power::record(
            &power::Record {
                port,
                minutes,
                shunt,
                label,
                out: file,
            },
            out,
        ),
        Cmd::Summary {
            files,
            megabytes,
            delivered,
            volts,
        } => power::summary(&files, megabytes, &delivered, volts, out),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
