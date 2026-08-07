//! Cross-layer executed conformance evidence for Tiler.
//!
//! A target profile is a set of claims, and conformance is what refutes them.
//! Tiler has the declaring half built — typed profiles, measured rows,
//! numerical realizations, feasibility predicates — and this crate is the
//! refuting half. A run here builds a program in the shared IR, lowers and
//! compiles it for a real target, executes it on a device, and compares the
//! result against the independent oracle. Every layer it crosses already tests
//! itself; what nothing tested is the composition, so a regression anywhere in
//! the vertical becomes a red test in `make full` rather than a spike someone
//! remembers to run.
//!
//! **Which layers one run crosses is the run's own claim, not this crate's.**
//! This paragraph read "plans it through the compiler … packages and validates
//! the artifact" when the crate was admitted, ahead of any content, and the
//! first run does neither: `tiler_compiler`'s recognizer refuses every
//! non-`f32` program under the rule `dtype-f32` before a subject is normalized,
//! so nothing can produce the plan alternative the optimizer, the artifact
//! envelope, and the runtime routing commit all consume. `bf16_vertical`
//! records that boundary at the function a reader would expect a `compile()`
//! call in, and states it again in its module header. A later run over an
//! `f32` program crosses those layers; a claim that *this* crate always does
//! would have been a claim about a member rather than about a run.
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
//! consumer and its own acceptance. Every module below is private and every
//! item in them is `pub(crate)`, so this crate exports nothing at all — the
//! runs are `#[cfg(test)]` entry points into private machinery.
//!
//! # `unsafe`, and where the whole of it lives
//!
//! This crate carries `unsafe_code = "deny"` rather than the workspace's
//! `forbid`, and the level moved here with the first site that needed it.
//! Tom decided the rule on 2026-08-07
//! (`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`):
//! named `#[allow(unsafe_code)]` at **individual sites, never at the crate**,
//! with FFI memory management against Metal as the only admitted justification,
//! and isolation as a design constraint rather than a preference.
//!
//! The complete population is **two sites**, both in `device_buffer`, both for
//! the raw pointer `metal::Buffer::contents` returns. Named rather than linked
//! throughout this header: every module below is `#[cfg(test)]`, so an
//! intra-doc link to one does not resolve when the crate is documented, which
//! is a rustdoc error rather than a dead link. The conformance logic — corpus,
//! oracle comparison, evidence attribution — contains none and does not need
//! any: `device_buffer` exposes a byte interface, so every width,
//! stride, and element count stays in safe code where it can be perturbed.
//! `bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`
//! walks `src/` and fails when a third appears.
//!
//! # Modules
//!
//! | module | what it owns |
//! | --- | --- |
//! | `bf16_vertical` | the BF16 corpus, its semantic program, scheduled region, emission, and comparison |
//! | `serial_sum` | the `f32` reduction vertical: the direct path, the retained portfolio, and the declared-grouping oracle |
//! | `envelope` | the artifact-delivered route: interface, placement, fail-closed probes, and the retained-digest comparison |
//! | `applicability` | whether this host may *offer* the profile it routes under, and the observation that is asked from |
//! | `device_preflight` | every obligation a host discharges before a routing commit, and how each refusal is classified |
//! | `measurement` | whether this host could measure, and the exact row a measured result is bounded to |
//! | `dispatch` | preparing, encoding, submitting, and classifying device dispatches (macOS only) |
//! | `device_buffer` | the two unsafe sites, and nothing else (macOS only) |
//!
//! Every module is `#[cfg(test)]`, which is the honest shape of what this crate
//! is: a conformance run is a *test*, and the machinery it needs has no
//! non-test caller and must not acquire one. The normal dependency edges in the
//! manifest still state what the crate is for — see the note there — and
//! `cargo check --workspace --all-targets`, `cargo nextest run --workspace`,
//! and therefore `make full` all build and run this content.
//!
//! The Metal binding and the two unsafe sites carry a second gate,
//! `cfg(target_os = "macos")`, which is what lets a non-Apple host build and
//! run the deterministic half and report the measured half as unavailable
//! rather than skip. **Only two modules carry that gate**, and the split is
//! deliberate: every comparison, classification, and refusal a device merely
//! supplies numbers to lives in a module compiled on every host, so the
//! device-free half of each claim runs in the gate wherever the workspace's
//! tests do rather than only on hardware.

#[cfg(test)]
mod applicability;
#[cfg(test)]
mod bf16_vertical;
#[cfg(all(test, target_os = "macos"))]
mod device_buffer;
#[cfg(test)]
mod device_preflight;
#[cfg(all(test, target_os = "macos"))]
mod dispatch;
#[cfg(test)]
mod envelope;
#[cfg(test)]
mod measurement;
#[cfg(test)]
mod serial_sum;
