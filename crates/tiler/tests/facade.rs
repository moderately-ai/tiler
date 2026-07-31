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

//! The `bind_*` cases carry a second claim beyond compiling: each defines its
//! own adapter over its own value type, in a crate that depends on `tiler`
//! alone. That is what "an arbitrary external consumer supplies the adapter
//! without a facade change or a global registration" means, checked rather than
//! asserted. Their `FACTS` constants are byte-identical to what
//! `tiler_macros::binding` emits, and the macro crate's tests read these files
//! to keep the two ends from drifting apart.

#[test]
fn facade_reexport_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/facade/pass/*.rs");
    cases.compile_fail("tests/facade/fail/*.rs");
}
