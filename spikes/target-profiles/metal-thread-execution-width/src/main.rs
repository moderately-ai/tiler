//! Measures `MTLComputePipelineState.threadExecutionWidth` across a frozen pipeline population.
//!
//! Width observations are collected only on the authorized Apple M3 Pro host.
//! `cargo test` does not read the metric and may run on the coordination host.

#![feature(variant_count)]

mod measure;
mod population;
mod record;
mod validate;

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use record::{WidthRecord, spike_root};
use validate::validate;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("measure") => measure(),
        Some("validate") => {
            let Some(path) = args.next() else {
                eprintln!("usage: metal-thread-execution-width validate <widths.json>");
                return ExitCode::from(2);
            };
            validate_path(path.as_ref())
        }
        _ => {
            eprintln!("usage: metal-thread-execution-width <measure|validate> [path]");
            ExitCode::from(2)
        }
    }
}

fn measure() -> ExitCode {
    let record = measure::measure();
    let encoded = serde_json::to_string_pretty(&record).expect("the record serializes");
    println!("{encoded}");
    let failures = validate(&record, &spike_root());
    if failures.is_empty() {
        return ExitCode::SUCCESS;
    }
    eprintln!("measured record failed its own validation:");
    for failure in failures {
        eprintln!("  {}", failure.message);
    }
    ExitCode::from(1)
}

fn validate_path(path: &Path) -> ExitCode {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("{} is readable: {error}", path.display());
    });
    let record: WidthRecord = serde_json::from_str(&text).unwrap_or_else(|error| {
        panic!("{} is a width record: {error}", path.display());
    });
    let failures = validate(&record, &spike_root());
    if failures.is_empty() {
        eprintln!("validation passed");
        return ExitCode::SUCCESS;
    }
    eprintln!("validation failed:");
    for failure in failures {
        eprintln!("  {}", failure.message);
    }
    ExitCode::from(1)
}
