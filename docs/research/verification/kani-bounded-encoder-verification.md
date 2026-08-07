---
schema: "tiler-doc/v1"
id: "tiler.research.verification.kani-bounded-encoder-verification"
kind: "research"
title: "Kani bounded verification of inexhaustible identity encoders"
topics: ["verification", "kani", "identity", "injectivity", "toolchain", "model-checking"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement", "primary-source-synthesis"]
informs: ["tiler.contract.correctness-and-testing"]
ticket: "spike-kani-bounded-verification-on-one-inexhaustible-encoder"
---

# Kani bounded verification of inexhaustible identity encoders

**Status:** complete, with a blocked primary path and a positive secondary result

**Reviewed:** 2026-08-07

## Traceability

- **Work record:** [spike-kani-bounded-verification-on-one-inexhaustible-encoder](../../../tickets/spike-kani-bounded-verification-on-one-inexhaustible-encoder.md).
- **Predecessor:** [prove-the-exhaustible-encoder-injectivity-claims-natively](../../../tickets/prove-the-exhaustible-encoder-injectivity-claims-natively.md) supplied the inexhaustible-encoder menu this spike selects from.
- **Reproduction:** [Kani bounded verification of inexhaustible identity encoders](../../../spikes/verification/kani-encoder-injectivity/README.md).
- **Host:** Apple M3 Pro, macOS 27.0 (build 26A5388g), `aarch64-apple-darwin`. Every timing below is from this host and bounded to it.

## Summary

**Fact.** The ticket's stop condition fired: `crates/tiler-ir` does not compile under Kani 0.67.0's bundled rustc. Nine errors from three independent causes, listed below. The primary path — a `#[cfg(kani)]` harness against the real crate — is **blocked on toolchain convergence**.

**Fact.** Against verbatim copies of the encoders, Kani proves injectivity, and for three of the four target encoders it proves it over the **entire** input domain with no residual bound — including the full 2^32 ordinal ranges that defeated the exhaustive-finite work. `push_resources` injectivity over all ~2^161 ordered input pairs discharges in 72 s.

**Fact.** The cost driver is not the input domain. CBMC quantifies over a `u32` symbolically at no measurable cost. The driver is the `Vec<u8>` **output**: comparing two vectors lowers to `memcmp` over a length CBMC knows only symbolically, and without an explicit unwind bound the loop unwinds forever.

**Inference.** For this class of encoder — finite-width input, no data-dependent loop, bounded output length — Kani is not merely feasible but cheap, and the bound it needs is a property of the encoder's own maximum output length rather than a guess. That makes the bound *provably sufficient* rather than a stated limitation.

**Proposal.** The blocked primary path is worth re-probing on each Kani release with a single command, and is not worth working around today.

## Per-Fact audit of the ticket, before building on it

`AGENTS.md` ranks ticket claims as unverified. Each was re-checked at base `411e09bf`.

| ticket claim | verdict | evidence |
| --- | --- | --- |
| `tiler-ir` carries `generic_const_parameter_types` + `min_adt_const_params` incomplete features | **true** | Both `#![feature(...)]` attributes and `#![allow(incomplete_features)]` are crate attributes in `crates/tiler-ir/src/lib.rs`. |
| ~8-month gap between Kani's bundled nightly and the repository pin | **true** | `rust-toolchain.toml` pins `nightly-2026-07-19`; Kani 0.67.0 installed `nightly-2025-11-21`. Seven months, four weeks. |
| Kani installs its own toolchain bundle and ignores `rust-toolchain.toml` | **true, and now measured rather than inferred** | `cargo kani setup` reported `[3/5] Installing rust toolchain version: nightly-2025-11-21-aarch64-apple-darwin`, and the failure diagnostic from inside this repository reads `this compiler was built on 2025-11-20`. The pin was in scope by ancestry and was not used. |
| latest Kani release bundles ~`nightly-2025-11-21` | **true** | Kani 0.67.0, published 2026-01-16, release notes "Upgrade Rust toolchain to 2025-11-21". |
| Kani ships "monthly releases each pinning their own nightly" | **stale, and it matters** | Monthly held through mid-2025 (0.60 2025-03-06 … 0.64 2025-07-03). It has since slowed: 0.65.0 2025-08-07, 0.66.0 2025-11-06, 0.67.0 2026-01-16, and **no release in the ~7 months since**. The ticket's re-probe condition was written assuming a cadence that would close the gap soon; it will not necessarily. |
| `push_resources` has a finite tail of 1 495 296 values over an unbounded head | **true** | `fn push_resources` in `crates/tiler-artifact/src/program/model.rs`. Head is `u32`, `u32`, `u64`, `bool`; tail is 649 × 3² × 2⁴ × 4² = 1 495 296. |
| `push_numerical` has a finite tail of 2 304 values | **true** | `fn push_numerical`, same file. 3² × 2⁴ × 4² = 2 304, behind a length-framed `String` and a `u32`. |
| `push_tensor_role` and `push_component_role` are "a single `u32` each, no recursion" | **true, with one imprecision** | `pub struct InputOrdinal(u32)` in `crates/tiler-ir/src/schedule/handles.rs`, `pub struct EncodedComponentRole(u32)` in `crates/tiler-ir/src/semantic/types.rs`. Both `new` admit every `u32`, so the domains really are 2^32 + 2 and 2^32 + 1. The predecessor's Outcome describes both as "three shapes"; `push_component_role` has two. Immaterial to the selection. |
| Tom authorized the Kani toolchain installation | **recorded, relayed** | Ticket trigger log, 2026-08-06 "later — fired", relayed by the coordinator. Not independently verifiable from the repository; the install proceeded on that record. |

No claim was materially false. The release-cadence claim is the one that changes a decision, and it is corrected above.

## The stop condition, and exactly what fails

```sh
cargo kani -p tiler-ir --only-codegen
```

Exit 1, `9 compilation errors`, three independent causes. The locations below are quoted from the compiler's own diagnostics at base `411e09bf`; they record where the errors were reported, and are not maintained pins into those files.

1. **`error[E0635]: unknown feature min_adt_const_params`** — `crates/tiler-ir/src/lib.rs:2`. The feature *name* does not exist at `nightly-2025-11-21`. This is not a case of an unstable feature needing a flag; there is nothing to enable.
2. **`` `[u64; RANK]` is forbidden as the type of a const generic parameter ``** — four sites: `semantic/shape_evidence.rs:32`, `shape/evidence.rs:63`, `:65`, `:67`. Downstream of (1). The nightly's own suggestion is the older, broader `adt_const_params`, which is a different feature with different semantics, so taking the suggestion would not be a faithful build of this crate.
3. **`` error[E0658]: use of unstable library feature `atomic_try_update` ``** — four sites: `index/handles.rs:13`, `kernel/handles.rs:24`, `program/handles.rs:24`, `semantic/handles.rs:14`. `Atomic*::try_update` is stable at the repository's pin and unstable at Kani's. Upstream tracking issue [rust-lang/rust#135894](https://github.com/rust-lang/rust/issues/135894).

Causes (1) and (3) are independent: fixing either leaves the other. Neither is a Tiler defect — both are the age gap.

### Would a toolchain change fix it, and is one required?

**No change is available that would help, so none is requested.** Kani does not accept a caller-supplied toolchain; it uses the nightly its release bundles, which is why (1) and (3) are not configuration problems. The only lever is a newer Kani release, and none exists.

For the record, the gap is narrower than "8 months of Rust changes":

| toolchain | rustc | `cargo check -p tiler-ir` |
| --- | --- | --- |
| `nightly-2025-11-21` (Kani 0.67.0's bundle) | 1.93.0-nightly | **fails**, the 9 errors above |
| `nightly-2026-05-03` | 1.97.0-nightly | **exits 0**, clean |
| `nightly-2026-07-19` (repository pin) | 1.99.0-nightly | exits 0, clean |

So a Kani release bundling anything from roughly 2026-05 onward would unblock the primary path. Narrowing the threshold further means installing intermediate nightlies — a host-environment change, and Tom's call — and would not change any decision, because no released Kani bundles a nightly in that window.

**Re-probe condition, in one command:** run `cargo kani -p tiler-ir --only-codegen` after any new Kani release. No release-note interpretation required.

## What was built instead, and what it is worth

The ticket permits a shim fallback only if the record states that a shim proof proves a copy and names the guard that would tie them. Both are done.

`spikes/verification/kani-encoder-injectivity/src/lib.rs` holds verbatim copies of 13 encoder functions and 15 type definitions. `guard.sh` re-extracts each named item from its source file and compares token content — comments, formatting, and visibility normalized away; renamed fields, added variants, changed tag literals, dropped writes, and reordered writes not. It asserts its own population (28) so a marker syntax that stopped matching fails loudly rather than reporting a clean zero.

**The guard was watched failing before being trusted**, on four planted drifts:

| planted drift | guard result |
| --- | --- |
| `SynchronizationKind::Collective => 0x04` changed to `0x09` | exit 1, `DRIFT: synchronization_kind_tag`, both sides printed |
| variant `Invented` added to copied `MemoryOrdering` | exit 1, `DRIFT: MemoryOrdering` |
| `bytes.push(permission_tag(signed_zero));` deleted from copied `push_resources` | exit 1, `DRIFT: push_resources` |
| one `@source:` marker deleted | exit 1, `GUARD POPULATION CHANGED: compared 27 items, expected 28` |

**What the tie does not cover, stated plainly.** It is a text tie. It catches a source edit not mirrored into the copy, which is the drift that would leave a stale proof standing. It does not tie the *callers* — fixed width and prefix-freeness matter relative to what a record writes next, and no caller is copied. And nothing forces it to run: no `make` target reaches the directory, by the standing spikes discipline. **A proof in this spike is evidence about Tiler's encoders only as strong as someone's willingness to run `guard.sh`.**

## The experiment

**Inputs.** Two independent symbolic values of the encoder's input type per harness, via `kani::any()`, plus for the prefix-freeness harness two symbolic 4-byte trailing runs.

**Outputs.** For each harness: a Kani verdict, the count of discharged checks, and the status of the CBMC unwinding assertion.

**Metrics.** Wall-clock per harness, CBMC's own reported verification time, and SAT instance size (variables, clauses).

**Unsupported cases, established rather than assumed.** Kani's codegen reported `caller_location (1)` and `foreign function (2)` as unsupported constructs, reachable only on panic paths; none was reachable in a violating way in any harness (each run reports its unreachable count). Structurally outside the technique as applied here: unbounded slices and vectors, unbounded strings, structural recursion, and trait objects — which is most of the predecessor's inexhaustible list, not a corner of it.

**Stop conditions.** (a) `tiler-ir` failing to codegen stops the primary path — fired. (b) A harness exceeding 1800 s stops that harness — did not fire. (c) A failing unwinding assertion demotes a harness's claim from complete to bounded-at-N — did not fire on any harness where completeness was claimed.

## Results

All eight harnesses verify. Wall-clock is the whole `cargo kani --harness` invocation; verification time is CBMC's.

| harness | domain | unwind | wall | CBMC | checks | unwinding assertion |
| --- | --- | --- | --- | --- | --- | --- |
| `push_tensor_role_injective` | 2^32 + 2 values, all pairs | 6 | 3 s | 1.44 s | 0 of 427 failed | SUCCESS |
| `push_component_role_injective` | 2^32 + 1 values, all pairs | 6 | 3 s | 1.00 s | 0 of 410 failed | SUCCESS |
| `push_resources_injective` | ~2^80.5 values, ~2^161 pairs | 33 | 72 s | 71.63 s | 0 of 628 failed | SUCCESS |

### The bound is on the output, not the input

This is the result most likely to be mis-cited, so it is stated separately.

Before any unwind bound was set, `push_tensor_role_injective` did not terminate. It was killed after ten minutes, still emitting `Unwinding loop memcmp.0 iteration 7370`. The input type is finite-width and the encoder has no loop; nothing about the 2^32 domain was the problem. `Vec<u8> == Vec<u8>` lowers to `memcmp`, and because the vector's length depends on which enum variant was taken, CBMC treats it as symbolic and unwinds without limit.

Setting `#[kani::unwind(6)]` took the same harness to **1.44 s**. The bound is not a compromise: `push_tensor_role` writes at most five bytes, so no execution can reach a sixth iteration, and CBMC's unwinding assertion — reported as `memcmp.unwind.0: SUCCESS` — is what proves the bound sufficient rather than merely stated. **Nothing lies outside these three proofs.** They cover their entire domains.

That distinction is the difference between a bounded model check and a proof, and here it is checkable per harness by reading one line of output.

## Harness ergonomics, which the ticket asks to be recorded

**`Arbitrary` derivation was frictionless for every type in the identity vocabulary.** `#[cfg_attr(kani, derive(kani::Arbitrary))]` worked unmodified on plain enums, on enums with struct variants carrying another enum (`SubnormalMode`, `ExceptionalValueAssumption`), on multi-field structs (`SynchronizationSubject`, `ResourceRequirements`), on `u32` newtypes, and on `Option<T>` of a derived type. Nothing needed a hand-written implementation and nothing needed a bound narrowed to make the derive work.

**`String` is the one type that has no `Arbitrary`, and that is the whole reason the `push_numerical` harnesses carry a bound.** A symbolic key has to be built by hand from a fixed-size symbolic byte array plus a symbolic length, which puts a `const N` in the harness and makes `N` the proof's domain boundary. Any encoder in the predecessor's string list inherits exactly this.

**The workspace lint set is a projected friction, not an observed one.** This spike is its own workspace and inherits none of it. `[workspace.lints]` sets `missing_docs = "warn"`, `unsafe_code = "forbid"`, and clippy `all` + `pedantic` at warn, and `crates/tiler-ir` takes them via `[lints] workspace = true`. A `#[cfg(kani)]` harness living inside the crate would face all of it — `missing_docs` on any `pub` harness helper, and pedantic on the harness bodies — under `-D warnings` in `make lint`. Whether harness code should be `cfg`-excluded from those lints or written to satisfy them is a question for whoever lands the in-crate version, and it is not answered here because nothing in-crate compiles yet.

## What this would be worth if the primary path unblocked

**Inference.** Three of the predecessor's named inexhaustible encoders move from "unverified framing argument" to complete proof, at a cost between 1 s and 72 s each. The 2^32 ordinals are the interesting part: they are not reachable by enumeration at any test budget, and they are reachable by CBMC essentially for free, because a SAT solver does not walk a domain.

**Inference, and the limit.** This does not generalize to most of the list. The predecessor named roughly fourteen slice/vector encoders, nine string encoders, and six structurally recursive ones. Each of those needs a bound that is a *guess about workloads* rather than a fact about the encoder, and a guessed bound cannot carry an unwinding-assertion completeness argument. For those, a Kani result would be genuinely bounded evidence and would need its own evidence class or an explicit bound field — which is the classification question the ticket routes back to the claims-ledger discussion with Tom, and which this spike does not decide.

**Proposal for the taxonomy, not a decision.** The three complete harnesses above are not "`SoundProof`-with-bound". They are unbounded over their stated domain, and their weakness is elsewhere: the subject is a copy, and the tie is a text guard someone must run. That is a *provenance* limitation, not a domain limitation, and the existing vocabulary does not have a slot for it. Whatever class is chosen should distinguish "proved over the whole domain of a copy" from "proved over part of the domain of the real thing", because they fail in different ways.

## Deferred, with reasons

- **Catalog rows.** `docs/research/README.md` and `spikes/README.md` are in the `contracts/navigation` scope, not `research/verification`, so this record and its spike are not yet reachable from the catalogs. Verbatim-landable rows and a carrier ticket: [`catalog-the-kani-verification-research-and-spike`](../../../tickets/catalog-the-kani-verification-research-and-spike.md).
- **`tiler-artifact` under Kani.** Not probed. It depends on `tiler-ir`, so it fails for the same reasons; a separate probe would add no information.
- **The exact nightly threshold.** Requires installing intermediate nightlies, which changes the evidence environment.
