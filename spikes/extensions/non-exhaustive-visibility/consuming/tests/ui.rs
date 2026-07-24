//! Compile-pass and compile-fail evidence for `#[non_exhaustive]` visibility.
//!
//! The `.stderr` file beside each `fail/` case is the retained measurement: it
//! is compared byte for byte, so a future compiler that changes the diagnostic
//! code, its note, or which patterns it reports fails this test instead of
//! being absorbed silently. Refresh one only after deciding that ADR 0074's
//! claim still holds under the new compiler; `TRYBUILD=overwrite` rewrites the
//! evidence and must not be used to make a red run green.

#[test]
fn non_exhaustive_visibility_diagnostics() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
