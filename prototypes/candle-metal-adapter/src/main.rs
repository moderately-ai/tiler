//! Tiler's first external consumer: a Candle custom op backed by a Tiler artifact.
//!
//! Everything of substance is in the modules below and compiled on macOS only,
//! for the same reason `prototypes/serial-sum-run` splits the same way: Candle's
//! Metal backend and the `objc2-metal` binding link Apple frameworks and cannot
//! link anywhere else, so an unconditional body would make
//! `cargo check --workspace` structurally impossible off Apple hardware.

#[cfg(target_os = "macos")]
mod adapter;
#[cfg(target_os = "macos")]
mod cache;
#[cfg(target_os = "macos")]
mod proof;
#[cfg(target_os = "macos")]
mod refusal;
#[cfg(target_os = "macos")]
mod wrapper;

/// Runs the Candle adapter proof.
#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    proof::main()
}

/// Reports that the proof needs Apple hardware rather than pretending to run.
#[cfg(not(target_os = "macos"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "the Candle Metal adapter prototype runs on macOS only; this build carries no Candle \
         Metal backend"
    );
    std::process::ExitCode::FAILURE
}
