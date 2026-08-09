---
id: size-the-four-hand-written-metal-all-arrays-from-their-types
title: Size the four hand-written metal ALL arrays from their types
status: done
priority: p1
dependencies: []
related: [cover-the-fifth-envelope-digest-domain-in-the-union-no-prefix-check]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [enumeration, tests, identity]
---

Four `ALL` arrays over enums carry hand-written lengths. Adding a variant without adding it to `ALL` **compiles**, so the population silently stops covering its domain while every check over it stays green.

## Facts, coordinator-verified at `4361a658`

**Fact — the correct pattern already exists in one of these very files.** `crates/tiler-metal/src/applicability.rs` declares `pub const ALL: [Self; core::mem::variant_count::<Self>()]` for `MetalGpuFamily`, and states the reason in prose anchored at *"declared length is `variant_count`, so the omission is an array-length"*. A `const` block at the same site asserts `MetalGpuFamily::ALL.len() == core::mem::variant_count::<MetalGpuFamily>()`.

**Worker correction (verified at `c0829b41`) — that `const` assertion was a tautology and could never fail.** `ALL` is declared `[Self; variant_count::<Self>()]`, so `ALL.len()` *is* `variant_count::<Self>()` and the comparison comes out true for every possible content of the list. Perturbing the subject confirms it: adding an `Apple10` variant without extending `ALL` fails with `error[E0308] ... expected an array with a size of 6, found one with a size of 5` at the declaration, and the assertion's message `MetalGpuFamily::ALL must name every family this vocabulary declares` never appears, because compilation stops at the declaration it restates. The completeness guard is the *declared length*, never the assertion. This ticket therefore did **not** replicate the assertion at the four new sites — four more checks that cannot say no would be exactly the artifact `AGENTS.md` warns about under "Verify that a check reaches its subject at all" — and removed the existing one, retargeting its prose onto the declaration. The ordering half of the same `const` block is a real check and is retained; perturbing it (swapping `Apple8` and `Apple9` in `ALL`) still fails with `error[E0080]: evaluation panicked: MetalGpuFamily::ALL must ascend, lowest Apple family first`.

**Fact — four siblings do not follow it.** `MslLanguageVersion::ALL` declares `[Self; 12]`, `MetalPlatform::ALL` declares `[Self; 10]`, and `MetalFloatArithmeticType::ALL` declares `[Self; 3]`, all in `crates/tiler-metal/src/target.rs`; `MetalHostPredicate::ALL` declares `[Self; 7]` in `applicability.rs`. Each is preceded by its own `pub enum` declaration and none is length-checked against `variant_count`.

**Fact — the damage propagates.** Each of the four is immediately followed by `pub const COUNT: usize = Self::ALL.len()`. `COUNT` is therefore derived from the array rather than from the type, so an `ALL` that stops covering shrinks `COUNT` with it, and any population sized by `COUNT` shrinks silently while still reporting success.

**Inference — why p1 despite no live defect.** All four arrays are complete today; this is latent. It is p1 because the failure is invisible by construction: a widened vocabulary produces no error, no warning, and no red check, and `AGENTS.md` names exactly this — "a hand-written length, a successor chain, and a wildcard-free match can all be satisfied by an enumeration that has stopped covering its domain." These four are the hand-written-length case, sitting beside a correct example.

**Worker correction (verified at `c0829b41`) — "no red check" is exactly true for one of the four, not all four.** Each of the four enums is matched exhaustively somewhere in non-test code, so *adding a variant* is always an `E0004` first; what the hand-written length permits is the developer fixing those arms and leaving `ALL` short. Measured from that state, the four differ:

- `MetalHostPredicate` is the fully silent one. Nothing reads `ALL` — it is cited in prose by `evaluate_metal_host_applicability` and by `crate::applicability_tests`, and `COUNT` is only ever compared against the number of cases a test wrote by hand. With an eighth predicate added, `as_str` arm supplied, and `ALL` left at seven, `cargo check -p tiler-metal --all-targets` exits 0 and `cargo nextest run -p tiler-metal` reports `122 tests run: 122 passed, 0 skipped`.
- `MslLanguageVersion` and `MetalPlatform` would eventually redden `crate::target_correspondence`, and `MetalFloatArithmeticType` would redden `every_arithmetic_type_indexes_to_its_own_slot` — but as an out-of-bounds panic in a `#[cfg(test)]` module far from the declaration, not as a statement about the list.

So the defect is real for all four and the fix is unchanged; the *severity argument* is "the guard is distant and test-only" for three of them and "there is no guard" for the fourth. Repairing this does not change what the ticket is for.

## What closes this

The four declarations sized from `core::mem::variant_count::<Self>()`, matching the sibling that already does it. Prefer the existing spelling over inventing a second one — two patterns for the same property is how the asymmetry arose.

**Perturb each of the four separately and quote the failure text.** Add a variant to each enum in turn without extending its `ALL`, and show the array-length error. Perturbing one and asserting the others behave the same is not evidence; `AGENTS.md` requires that where a check guards several independent properties, each is perturbed on its own. Revert each and confirm.

**Check whether any of the four is `pub` in a way that makes its length observable.** `MslLanguageVersion`, `MetalPlatform`, and `MetalFloatArithmeticType` are `pub` enums, so if `COUNT` is reachable by a consumer its value is contract, and any change to how it is derived is a **labelled draft** under ADR 0075 until Tom accepts the surface. Report the included and excluded sets; do not decide the boundary.

**Then check the rest of the workspace for the same shape.** This audit swept `crates/` for `[Self; N]`, `[&str; N]`, and `[&[u8]; N]` and found further hand-sized arrays outside `tiler-metal` — in `tiler-cache`, `tiler-conformance`, and `tiler-runtime` tests. Those are other scopes; **report them with a count** rather than editing, so the sweep's extent is on record either way.

## Sweep result (worker, at `c0829b41`; reported not edited)

The shape is *an array that enumerates a fieldless enum's variants one-to-one, sized by a literal*. Two neighbouring shapes are deliberately **not** it and must not be "fixed": an array enumerating variant×payload combinations, where the literal is correctly larger than `variant_count` (`SUBNORMAL_MODES` and `EXCEPTIONAL_ASSUMPTIONS` in `crates/tiler-ir/src/exhaustive_injectivity.rs`, `SHAPES` in `crates/tiler-ir/src/semantic/types.rs`); and a deliberate subset or ordered route (`REALIZED_DIMENSIONS` is 8 of `NumericalDimension`'s 11; `MATERIALIZED_ROUTE` is a 10-step sequence over an 8-variant `Stage`). `exhaustive_injectivity.rs` already argues its own exclusions in prose and is consistent.

**Correction 2026-08-08.** The exclusion from this ticket's *fieldless-enum* shape was right, but `exhaustive_injectivity.rs` was not consistent. The two lists contain every current inhabitant, yet their literal lengths are not derived from `FlushedZeroSign` and `ValueDomainProvenance`; after the exhaustive encoder matches are repaired, either payload vocabulary can widen while its list stays short. [`derive-the-payload-carrying-enum-populations-in-the-injectivity-module`](derive-the-payload-carrying-enum-populations-in-the-injectivity-module.md) owns the IR repair, and [`derive-the-artifact-numerical-and-fenced-space-populations`](derive-the-artifact-numerical-and-fenced-space-populations.md) owns the artifact copies. This correction preserves the sweep's boundary and retracts only the false consistency conclusion.

**18 same-shape sites remain, in seven crates, none in scope here.** `tiler-metal-aot` 4 — `AppleSdk::ALL` `[Self; 9]`, `ApplePlatform::ALL` `[Self; 10]`, `MslVersion::ALL` `[Self; 12]` (`src/input.rs`), `CompileStage::ALL` `[Self; 2]` (`src/diagnostic.rs`); `tiler-compiler` 4 — `CANONICAL_PROPERTIES`, `CANONICAL_COMPONENTS`, `CallFailureStage::ALL`, `CANONICAL_AXES`; `tiler-ir` 3 — `ArithmeticType::ALL`, `ConformanceEvidenceClass::ALL`, `CompositionStep::ORDER`; `tiler-cache` 2 — `SubjectFacet::ORDER`, `BundleSection::ORDER`; `tiler-macros` 2 — `DeliveredFamily::ALL`, `NamedProfile::ALL`; `tiler-artifact` 1 — `RouteResourceDimension::ALL`; `tiler-build` 1 — `Variant::ALL` in `examples/identity_join_producer.rs`; `tiler-runtime` 1 — `COMPLETE_ROUTE` in `tests/adapter_route/main.rs`. No `tiler-conformance` site has the shape: `grep -rn --include='*.rs' -E ':[[:space:]]*\[Self;' crates/tiler-conformance` returns nothing, and its literal-sized arrays are operand corpora and a foreign `MTLGPUFamily` binding table.

**The `tiler-metal-aot` three are the highest-value follow-up**, because `ApplePlatform` and `MslVersion` are the paired counterparts of the `MetalPlatform` and `MslLanguageVersion` fixed here, and `crate::target_correspondence` couples them by `const _: [(); FAMILY_COUNT] = [(); ApplePlatform::COUNT];`. That coupling does **not** rescue the driver side: a family added to `ApplePlatform` and omitted from its `ALL` leaves `ApplePlatform::COUNT` at 10 and the equality still holds. Fixing it needs `#![feature(variant_count)]` at that crate's root, which it does not currently carry.

## Current sweep correction — 2026-08-09

The 18-site remainder above is the worker's census at `c0829b41`, not the
current population. [`size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them`](size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them.md)
subsequently repaired all four named `tiler-metal-aot` declarations and added
that crate's `variant_count` feature gate. Their live declarations are sized
from `core::mem::variant_count::<Self>()`; the count coupling remains only a
cross-crate correspondence check. The other fourteen reported sites remain
outside this ticket and are not reclassified by that follow-up. This correction
updates the sweep count without changing the four-site `tiler-metal` outcome.
