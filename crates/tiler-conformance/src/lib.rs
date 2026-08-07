//! Cross-layer executed conformance evidence for Tiler.
//!
//! A target profile is a set of claims, and conformance is what refutes them.
//! Tiler has the declaring half built — typed profiles, measured rows,
//! numerical realizations, feasibility predicates — and this crate is the
//! refuting half. A run here builds a program in the shared IR, plans it
//! through the compiler, lowers and compiles it for a real target, packages
//! and validates the artifact, executes it on a device, and compares the
//! result against the independent oracle. Every layer it crosses already
//! tests itself; what nothing tested is the composition, so a regression
//! anywhere in the vertical becomes a red test in `make full` rather than a
//! spike someone remembers to run.
//!
//! The crate is admitted ahead of its first run deliberately. Admitting a
//! workspace member fixes a dependency edge and a verifier-ownership boundary,
//! which is reviewable on its own; a member admitted with a migration attached
//! would have decided what migrates before the survey that decides it ran.
//! `conform-the-bf16-vertical-end-to-end` is the first content and
//! `survey-what-belongs-in-the-conformance-crate` decides what follows.
//!
//! # The frontend is not reachable from here
//!
//! Nothing in this crate may depend on `tiler` or `tiler-macros`.
//! `crates/tiler/tests/dependency_direction.rs` holds the frontend at the top
//! of the workspace graph, and it reads `Cargo.lock` precisely because the
//! lockfile merges normal, build, and development edges into one list — so a
//! development edge trips it exactly as a normal one does, and there is no
//! spelling of the dependency that this crate could take.
//!
//! That is a real limitation rather than a formality, and stating it is the
//! point: a program under conformance is built through `tiler_ir` and handed
//! to the compiler directly, never written as `tiler::tensor!` and expanded.
//! The inline macro path — expansion, symbol binding, the ahead-of-time
//! embedding workflow — is therefore *not* covered by anything here, and stays
//! covered where it already is, in `crates/tiler/tests/facade/`. A future run
//! that wanted to execute a macro-expanded region would need a home outside
//! this crate, not a relaxation of that test.
//!
//! # What this crate is not
//!
//! **Not a second semantic authority.** `tiler-reference` is the oracle and
//! this crate uses it; it never states what a program should compute. The
//! moment a run here computes an expected value of its own, the comparison
//! stops being evidence — it is the shared-implementation failure
//! [Correctness and testing](../../../docs/correctness-and-testing.md)
//! already names, one layer up, and weakening what an operation means to fit
//! what a target delivers is the authority substitution
//! [ADR 0076](../../../docs/decisions/0076-declare-target-honourable-numerical-realizations.md)
//! forbids. Where a contract permits reassociation the oracle is still the
//! reference's, against the grouping the physical plan declared, and not a
//! tolerance.
//!
//! **Not a benchmark harness.** Timing has its own discipline — an idle host,
//! warm-up, repetitions, stated noise controls, a named baseline — and none of
//! it survives contact with a correctness gate that runs beside every other
//! test in the workspace. Mixing them makes the gate flaky and the numbers
//! untrustworthy in one step, so a measurement belongs in the performance work
//! that can state its conditions, not here.
//!
//! **Not a home for layer-local tests.** What this crate owns is *cross-layer
//! executed* evidence. A test of one layer's own behaviour stays in that
//! layer's crate, where its failure names the layer that broke. Without this
//! line the crate becomes the place a test goes when nobody wants to decide
//! where it belongs, and its failures stop attributing anything.
//!
//! # A host that cannot measure says so
//!
//! A conformance run has two halves that fail for unrelated reasons: a
//! deterministic half — building the program, planning it, the structural and
//! reference-comparable facts — and a measured half that needs a device, a
//! toolchain, and the environment row the claim is bounded to. A host that
//! offers the second runs both. A host that does not runs the first and
//! **reports the measured half as unavailable, naming what was missing.**
//!
//! It never skips silently, and it never reports a pass it did not observe. A
//! silent skip makes an unmeasured host indistinguishable from a green one,
//! so the gate's verdict comes to depend on which machine ran it and nothing
//! says which; a claimed pass is worse, because it manufactures evidence for a
//! device that was never reached. The unavailability is an outcome the run
//! states, so a reader can always tell which of the two halves a green result
//! covers.
//!
//! # Public surface
//!
//! There is none, and admitting this member accepted none. A crate's public
//! namespace is a boundary under
//! [ADR 0075](../../../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md);
//! anything here stays `pub(crate)` or test-only until an item has a stated
//! consumer and its own acceptance.
