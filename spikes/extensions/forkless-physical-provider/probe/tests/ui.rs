//! Compile-fail evidence for the private surfaces that gate a forkless provider.
//!
//! An absent API is only evidence when the compiler says it is absent. The
//! `.stderr` file beside each `fail/` case is the measurement: it is compared
//! byte for byte, so a later commit that makes `frontier` public, or that adds
//! a physical-provider installation method, fails this test instead of quietly
//! invalidating the recorded conclusion. That is the point — these fixtures are
//! the trigger that reopens the question.
//!
//! Each blocked case is paired with the compiling contrast it is evidence
//! against, under `pass/`. A diagnostic alone says what the compiler rejects
//! and not what it accepts, and the whole finding here is an *asymmetry*
//! between two seams, so the accepted half has to be shown.
//!
//! Refresh a diagnostic with `TRYBUILD=overwrite` only after deciding the claim
//! still holds, and re-record the toolchain in `results/` in the same commit.

#[test]
fn forkless_provider_surface_diagnostics() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
