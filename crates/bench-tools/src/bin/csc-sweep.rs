//! Spectrum sweeps with an RTL-SDR (Blog V4 or V3), for the CSC-R1 bench quests.
//!
//! ```text
//! csc-sweep record   --band 2m --label "load, gain 20" --out load.csv --dc-checked
//! csc-sweep summary  load.csv
//! csc-sweep compare  halow-off.csv halow-on.csv
//! csc-sweep response --ref through.csv --dut stub.csv --at 98e6 145.8e6
//! csc-sweep stub     --freq 98e6 --vf 0.66
//! csc-sweep stub     --freq 98e6 --length 0.51
//! ```
//!
//! All levels are relative dB from the dongle, not dBm.

use std::process::ExitCode;

use bench_tools::sweep;
use clap::{Parser, Subcommand};

/// Spectrum sweeps with an RTL-SDR, for the CSC-R1 bench quests. Levels are relative dB, not dBm.
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run rtl_power and save a labelled sweep
    Record {
        #[arg(long, value_parser = ["fm", "2m", "70cm", "halow", "filter2m", "filter70cm"])]
        band: Option<String>,
        #[arg(long)]
        start: Option<f64>,
        #[arg(long)]
        stop: Option<f64>,
        #[arg(long, default_value_t = 10e3)]
        step: f64,
        /// Fixed tuner gain in dB; never auto
        #[arg(long, default_value_t = 20.0)]
        gain: f64,
        #[arg(long, default_value_t = 60)]
        seconds: u32,
        #[arg(long, default_value_t = 2)]
        interval: u32,
        #[arg(long, default_value_t = 0)]
        ppm: i32,
        /// What was connected, in a few words
        #[arg(long)]
        label: String,
        #[arg(long, default_value = "hardware bench", value_parser = ["hardware bench", "received RF"])]
        evidence: String,
        #[arg(long, default_value = "RTL-SDR Blog V4")]
        receiver: String,
        #[arg(long)]
        out: String,
        /// You measured under 0.5 V DC on the SMA centre pin with this dongle
        #[arg(long)]
        dc_checked: bool,
    },
    /// Noise floor and strongest signal
    Summary {
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// Noise-floor change from one sweep to another
    Compare {
        base: String,
        test: String,
        /// Compare even if settings differ
        #[arg(long)]
        force: bool,
    },
    /// Filter response from a noise source
    Response {
        /// Noise source straight into the dongle
        #[arg(long = "ref")]
        reference: String,
        /// Noise source through the device
        #[arg(long)]
        dut: String,
        /// Frequencies to report, Hz
        #[arg(long, num_args = 0..)]
        at: Vec<f64>,
        #[arg(long)]
        out: Option<String>,
        /// Compare even if settings differ
        #[arg(long)]
        force: bool,
    },
    /// Quarter-wave coax stub length or velocity factor
    Stub {
        #[arg(long)]
        freq: f64,
        #[arg(long, default_value_t = 0.66)]
        vf: f64,
        /// Measured length in m: solve for velocity factor
        #[arg(long)]
        length: Option<f64>,
    },
}

fn main() -> ExitCode {
    let out = &mut std::io::stdout();
    let result = match Args::parse().cmd {
        Cmd::Record {
            band,
            start,
            stop,
            step,
            gain,
            seconds,
            interval,
            ppm,
            label,
            evidence,
            receiver,
            out: file,
            dc_checked,
        } => sweep::record(
            &sweep::Record {
                band,
                start,
                stop,
                step,
                gain,
                seconds,
                interval,
                ppm,
                label,
                evidence,
                receiver,
                out: file,
                dc_checked,
            },
            out,
        ),
        Cmd::Summary { files } => sweep::summary(&files, out),
        Cmd::Compare { base, test, force } => sweep::compare(&base, &test, force, out),
        Cmd::Response {
            reference,
            dut,
            at,
            out: file,
            force,
        } => sweep::response(&reference, &dut, &at, file.as_deref(), force, out),
        Cmd::Stub { freq, vf, length } => sweep::stub(freq, vf, length, out),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
