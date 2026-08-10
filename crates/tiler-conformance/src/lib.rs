#![doc(test(attr(forbid(unsafe_code))))]
#![cfg_attr(test, feature(variant_count))]
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
//! BF16 run does neither.
//!
//! **Corrected 2026-08-07 as to *why* it does neither.** The reason given here
//! was `tiler_compiler`'s recognizer, which "refuses every non-`f32` program
//! under the rule `dtype-f32` before a subject is normalized".
//! `widen-the-strategy-recognizer-past-the-f32-wall` retired that rule: the
//! recognizer now derives a program's one arithmetic type and admits the widths
//! this build can spell a per-point body in, BF16 among them. The boundary that
//! survives is the **target profile's**. The authoritative macOS Apple9 ledger
//! declares BF16 dispatchability and the two subnormal tables and nothing else,
//! so a BF16 request clears the dimensions that were measured and is refused at
//! numerical resolution on an undeclared one — and nothing can then produce the
//! plan alternative the optimizer, the artifact envelope, and the runtime
//! routing commit all consume. `bf16_vertical` records that boundary at the
//! function a reader would expect a `compile()` call in, states it again in its
//! module header, and — unlike every previous statement of it —
//! `bf16_vertical::tests::the_request_boundary_stops_at_the_ledgers_undeclared_bf16_contraction_row`
//! observes it, so the next time the reason moves a test says so. `serial_sum`'s
//! `f32` runs do cross those layers, which is exactly why a claim that *this*
//! crate always does would have been a claim about a member rather than about a
//! run.
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
//! `crates/tiler/tests/workspace_unsafe_sites.rs` cross-checks Cargo's actual
//! package/target population against the explicit members, follows supported
//! local source loads, and pins all four workspace permissions by path,
//! complete item signature, and exact reason, including this crate's pair.
//!
//! **Not inheriting has a second cost, and it is also checked.** A member that
//! cannot inherit one entry of the workspace lint table has to restate all of
//! it, and a restatement drifts: a lint added, tightened, or removed
//! workspace-wide reaches the fourteen inheriting members and neither checked
//! exception. `lints` reads both manifests and fails unless they differ by
//! exactly that one level; the prototype runs the same reader. The
//! workspace-wide inheritance test holds the exception set to these two and
//! requires both checks, so a third uninherited member or either table drifting
//! is a red test.
//!
//! # Modules
//!
//! | module | what it owns |
//! | --- | --- |
//! | `bf16_vertical` | the BF16 corpus, its semantic program, scheduled region, emission, and comparison |
//! | `serial_sum` | the `f32` reduction vertical: the direct path, the retained portfolio, and the declared-grouping oracle |
//! | `envelope` | the artifact-delivered route: interface, placement, fail-closed probes, and the retained-digest comparison |
//! | `publication` | the envelopes and proof records that route reads, published in the same run |
//! | `applicability` | whether this host may *offer* the profile it routes under, and the observation that is asked from |
//! | `device_preflight` | every obligation a host discharges before a routing commit, and how each refusal is classified |
//! | `measurement` | whether this host could measure, and the exact row a measured result is bounded to |
//! | `retained_record` | the realization probe's retained record, its `direct` digests, and how this host's row compares against it |
//! | `ledger` | the private typed declarations compared with the ledger's manually owned conformance-evidence prose |
//! | `portability` | the census that holds the non-Apple claim below to a population rather than to this paragraph |
//! | `lints` | this crate's uninherited lint table, held against the workspace's |
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
//! rather than skip. **Only two of the modules above carry that gate** —
//! `dispatch` and `device_buffer` — and the split is deliberate: every
//! comparison, classification, and refusal a device merely supplies numbers to
//! lives in a module compiled on every host, so the device-free half of each
//! claim runs in the gate wherever the workspace's tests do rather than only on
//! hardware. Four *nested* modules carry it too — `envelope::apple`,
//! `measurement::host`, `measurement::apple`, and `serial_sum::apple` — and
//! each is paired with a `cfg(not(target_os = "macos"))` sibling of the same
//! name, so what a non-Apple host loses there is the body of a device call and
//! never the call itself.
//!
//! # That claim is checked rather than asserted
//!
//! It has failed twice without anything noticing, so it is worth being exact
//! about which instrument covers which half.
//!
//! **That the crate compiles off Apple** is not observable from inside a
//! running test, and the instrument is a cross-target build:
//!
//! ```sh
//! cargo check   -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu
//! cargo clippy  -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
//! ```
//!
//! **It is deliberately not in `make full`**, and that is Tom's standing
//! decision rather than an omission: the target is a 156 MB standard library
//! that no host bootstrapped from `deps.sh` installs, and
//! `declare-the-cross-compilation-targets-in-the-toolchain-manifest` parked
//! taking it as a gate dependency. It is a manual check owned by whoever
//! changes this crate, and it has to be re-run when a module is added, when an
//! item's only caller moves behind the macOS predicate, or when the routed half
//! grows.
//!
//! **That the deterministic tests still exist there** is the failure the manual
//! check would not catch even when it is run, because a collapsed population
//! compiles and lints perfectly. `portability` is the in-gate instrument for
//! it: it counts the test population the macOS predicate does *not* remove and
//! refuses a floor, on both hosts, so gating one more module fails on the
//! machine that did it. It reads that population out of the source, which is
//! why this header names the test attribute in prose rather than spelling it —
//! a literal here would be counted as a test in the one file that declares the
//! gates, and that file is the one the census cannot classify.
//!
//! The items an `apple` module is the only caller of are dead on every other
//! host, and `envelope` and `publication` say so at the module rather than at
//! each of their forty-odd items — under `cfg(not(target_os = "macos"))`, so the
//! host that does use them is still held to the ordinary lint. Their headers
//! name the population.

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
mod ledger;
#[cfg(test)]
mod lints;
#[cfg(test)]
mod measurement;
#[cfg(test)]
mod portability;
#[cfg(test)]
mod publication;
#[cfg(test)]
mod retained_record;
#[cfg(test)]
mod serial_sum;
