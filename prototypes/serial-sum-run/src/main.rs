//! Host entry point for the serial-Sum value proof.
//!
//! This binary dispatches a compiled kernel on a real Metal device, so it is
//! meaningful on Apple platforms alone. The whole proof therefore sits behind
//! one `cfg` boundary here rather than being gated item by item, and the
//! `metal` dependency is declared only for that target — the crate is compiled,
//! linted, and formatted on every supported host, while the Apple-only body is
//! selected by the target rather than by whether the workspace was checked at
//! all. [`proof`] documents what the proof does and why it takes two paths.
//!
//! A host without Metal fails loudly. Returning success here would report a
//! passing value proof on a machine that never ran one, and this binary's only
//! output is that verdict.

#[cfg(target_os = "macos")]
mod buffer;
#[cfg(target_os = "macos")]
mod proof;

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    proof::main()
}

#[cfg(not(target_os = "macos"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "serial-sum runtime proof requires a Metal device and is supported on macOS only; \
         this host is {}",
        std::env::consts::OS,
    );
    std::process::ExitCode::FAILURE
}
