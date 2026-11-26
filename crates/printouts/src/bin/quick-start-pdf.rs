//! Build the one-page printable quick start,
//! `output/pdf/cubesat-classroom-quick-start.pdf`.
//!
//! ```text
//! cargo run --locked -p printouts --bin quick-start-pdf
//! ```

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = Path::new("output/pdf/cubesat-classroom-quick-start.pdf");
    let result = printouts::quick_start::build(&root).and_then(|(pdf, installer)| {
        let path = root.join(out);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, pdf).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(installer)
    });
    match result {
        Ok(installer) => {
            println!("Wrote {} with installer {installer}", out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
