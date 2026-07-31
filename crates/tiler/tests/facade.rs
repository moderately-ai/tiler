//! Downstream compile contract for the facade's macro re-export.
//!
//! These cases compile as a separate out-of-tree crate, which is what makes
//! them evidence: an in-crate test resolves `crate::__private` and so cannot
//! tell a working expansion from one whose absolute path is wrong.
//!
//! What that separate crate does *not* isolate is the manifest. `trybuild`
//! copies the crate under test's `[dependencies]` into the generated project,
//! so `tiler-macros` is declared there too (inspect
//! `target/tests/trybuild/tiler/Cargo.toml` after a run). No `trybuild` case
//! can remove it, because the facade genuinely depends on it. The fixtures
//! therefore prove what they can — that nothing a consumer *writes* or a macro
//! *emits* names anything but `tiler` — while the resolved-graph invariant is
//! `dependency_direction`'s job.

#[test]
fn facade_reexport_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/facade/pass/*.rs");
    cases.compile_fail("tests/facade/fail/*.rs");
}
