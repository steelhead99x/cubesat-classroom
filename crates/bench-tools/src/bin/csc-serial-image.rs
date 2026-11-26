//! Rebuild a photo sent by the XIAO bench CubeSat's DOWNLINK command.
//!
//! ```text
//! csc-serial-image capture.txt photo.jpg
//! ```
//!
//! Exit status: 0 intact, 1 incomplete or damaged.

use std::io::Read;
use std::process::ExitCode;

use clap::Parser;

/// Rebuild a photo sent by the XIAO bench CubeSat's DOWNLINK command.
#[derive(Parser)]
struct Args {
    /// Saved Serial Monitor text, or - for standard input
    log: String,
    /// Where to write the JPEG
    output: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let mut bytes = Vec::new();
    let read = if args.log == "-" {
        std::io::stdin().read_to_end(&mut bytes).map(|_| ())
    } else {
        std::fs::read(&args.log).map(|b| bytes = b)
    };
    if let Err(e) = read {
        eprintln!("{}: {e}", args.log);
        return ExitCode::FAILURE;
    }
    let text = String::from_utf8_lossy(&bytes);
    match bench_tools::serial_image::run(&text, &args.output, &mut std::io::stdout()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
