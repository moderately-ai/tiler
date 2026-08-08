//! Compile-fail evidence for the subjects the host reserves from a provider.
//!
//! An absent API is only evidence when the compiler says it is absent. The
//! `.stderr` file beside each `fail/` case is the measurement: it is compared
//! byte for byte, so a commit that publishes one of the five reserved subjects
//! fails this test instead of quietly invalidating the recorded conclusion.
//!
//! **What these fixtures pin changed on 2026-08-08, and the difference is the
//! point.** Until then they pinned the *seam's absence*: no installation
//! method, and a provider vocabulary behind a private module. Both landed, so
//! both goldens went red — which is the trigger firing, not a golden to bless —
//! and re-pointing them at whatever the compiler now says would have produced a
//! check that nobody chose and that could never fail meaningfully again. They
//! pin the seam's *boundary* instead: the five subjects an installed provider
//! may not reach, four of which are also `compile_fail` doctests on
//! `crates/tiler-compiler/src/physical_provider.rs` and are therefore stated
//! from two independent places, and one of which — the proposal-body
//! restriction — nothing else checks.
//!
//! Each blocked case is paired with the compiling contrast it is evidence
//! against, under `pass/`. A diagnostic alone says what the compiler rejects and
//! not what it accepts, and the whole finding here is that a provider is
//! *installable and bounded* rather than blocked, so the accepted half has to be
//! shown.
//!
//! Refresh a diagnostic with `TRYBUILD=overwrite` only after deciding the claim
//! still holds, and re-record the toolchain in `results/` in the same commit.

#[test]
fn reserved_provider_subject_diagnostics() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
