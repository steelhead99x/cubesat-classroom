//! `cubesat-cad check` runs the checks; `cubesat-cad build <folder>` also writes the files.
//! `cubesat-cad archive <hardware folder>` rebuilds the archived Rev A and Rev B files.

use std::path::PathBuf;
use std::process::ExitCode;

use cubesat_cad::check::{Outcome, verify};
use cubesat_cad::design::Design;
use cubesat_cad::export::write_all;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("archive") {
        let out = PathBuf::from(args.get(1).map_or("hardware", String::as_str));
        return match cubesat_cad::archive::write_all(&out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        };
    }
    let design = Design::default();
    let report = verify(&design);
    for c in &report.checks {
        let mark = match c.outcome {
            Outcome::Pass => "pass",
            Outcome::Fail => "FAIL",
            Outcome::Deviation => "dev ",
            Outcome::NotApplicable => "n/a ",
        };
        println!("{mark}  {:<14} {}", c.id, c.detail);
    }
    if !report.passed() {
        eprintln!(
            "Rev C failed {} check(s); no files written.",
            report.failures().len()
        );
        return ExitCode::FAILURE;
    }
    match args.first().map(String::as_str) {
        Some("check") | None => ExitCode::SUCCESS,
        Some("build") => {
            let out = PathBuf::from(args.get(1).map_or("hardware/rev-c", String::as_str));
            match write_all(&design, &report, &out) {
                Ok(()) => {
                    println!("wrote {}", out.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{e}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(other) => {
            eprintln!(
                "usage: cubesat-cad [check | build <folder> | archive <hardware folder>] (got {other})"
            );
            ExitCode::FAILURE
        }
    }
}
