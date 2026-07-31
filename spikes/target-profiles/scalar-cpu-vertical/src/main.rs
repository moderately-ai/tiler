//! A bounded scalar CPU backend vertical, carried end to end.
//!
//! Run it from this directory; `README.md` records the exact invocation, what
//! the run proves, and where its evidence stops.
//!
//! The binary's only product is a verdict, so a failure at any stage is a
//! non-zero exit with the stage named. There is no partial success: a run that
//! reported success after skipping the comparison would be reporting a proof
//! nobody performed.

mod host;
mod image;
mod interpret;
mod profile;
mod vertical;

use std::process::ExitCode;

fn main() -> ExitCode {
    match vertical::run() {
        Ok(report) => {
            let out = std::env::args().nth(1);
            if let Some(path) = out {
                match std::fs::write(&path, report.to_json()) {
                    Ok(()) => println!("recorded {path}"),
                    Err(cause) => {
                        eprintln!("the result fixture could not be written to {path}: {cause}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Err(failure) => {
            eprintln!("scalar CPU vertical failed: {failure}");
            ExitCode::FAILURE
        }
    }
}
