---
id: route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge
title: Route the BF16 vertical's declared conformance through the checked bridge
status: in-progress
priority: p2
dependencies: [give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject]
related: [give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject, accept-the-bf16-subnormal-resolution-carrier, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/conformance, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, bf16, fail-closed]
claimed_from: todo
assignee: w-route-bf16
lease_expires_at: 1786143052
---
## User-visible outcome

The one site in the workspace where a region's declared realization and a reference evaluation meet derives its conformance through `ReferenceNumericalConformance::from_realization` instead of transcribing the same contract twice, so that route's six transform refusals and its subject-agreement check both become reachable in production rather than only in `tiler-reference`'s own tests.

## Why this node exists

**Fact — the parent could not reach this site.** [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md) is scoped to `crates/tiler-reference/**`. It gave the bridge its subject, made `ReferenceEvaluationRequest::conformance_for` refuse a conformance another format resolved, and exercised the whole path end to end in-crate — but `tiler-reference` names no region type at all: it imports nothing from `tiler_ir::schedule` except the numerics vocabulary, and neither `ReferenceEvaluator` nor `IndexRegionEvaluator` takes a scheduled region. The caller has to live where the region does.

**Fact — that place is the BF16 vertical, and it currently transcribes rather than bridges.** `crates/tiler-conformance/src/bf16_vertical.rs` holds both halves within thirty lines of each other:

- `declared_realization()` (`:433`) builds the region's `NumericalRealization` from `declared_contract()`'s accessors;
- `declared_conformance()` (`:463-466`) builds the oracle's `ReferenceNumericalConformance::new(contract.input_subnormals(), contract.result_subnormals())` from the same accessors.

One contract, two transcriptions. `the_oracle_and_the_region_are_told_one_contract` holds them equal, which is why nothing is wrong today — but the bridge's refusals are the thing being routed around, and a contract resolving `reassociation: Permitted` would reach this oracle as two silently-ignored subnormal modes instead of a `ReassociationPermitted` refusal.

**Fact — the parent's own header cites this ticket by name at that site.** `bf16_vertical.rs:148-157` says the conformance is stated at the call site because `from_realization` "discards the format its realization was stated about and has no caller". Both clauses are now false, so that paragraph is stale on top of the routing gap.

## What this owes

- **`declared_conformance()` derives from the region.** `RealizationWitness::of(&region)` (`crates/tiler-ir/src/schedule/witness.rs:93`) hands back `realization()` and `accumulation()` — the region's own arithmetic type — from one object, which is the pairing `from_realization` is documented against. Reading the realization from the witness and the subject from somewhere else is what would let the two drift.
- **The transcription is removed, not kept beside the derivation.** Two sources that a test holds equal is the shape this ticket exists to collapse; `the_oracle_and_the_region_are_told_one_contract` should become vacuous or be restated as something the single source cannot satisfy trivially.
- **At least one refusal watched firing on this route.** `realization_of(NumericalContract::…)` is already parameterized so the vertical can build its region under a *different* contract — that is how it watches emission refuse the strict BF16 contract. A contract permitting reassociation, run through the bridge, must produce `UnsupportedReferenceContract::ReassociationPermitted` rather than a quietly preserving oracle.
- **The two stale documents corrected**, both of which assert the pre-parent state as current fact:
  - `crates/tiler-conformance/src/bf16_vertical.rs:148-157` — "discards the format … and has no caller".
  - `docs/correctness-and-testing.md:55` — "The window is real and currently **unreachable**: `from_realization` has no caller anywhere in `crates/` or `prototypes/`", and "what no capability yet checks is that the conformance it was handed was stated about its own format". The capability check landed with the parent (`ReferenceEvaluationRequest::conformance_for`, and the BF16 family names `ArithmeticType::Bf16`); the sentence that survives is the `Unstated` boundary, which is narrower and should be restated as such rather than deleted.

## Closes when

The BF16 vertical's oracle conformance is derived from the region's own witness through `from_realization`, no second transcription of the contract remains, one bridge refusal is observed firing on that route before restoration, and both stale documents state the boundary that actually holds — that an *unsubjected* conformance still reaches every capability, and a subjected one is checked.

## Graph maintenance

Filed 2026-08-07 by the worker on the parent, on discovering that the parent's own "every construction site is `strict()` or a test's `new()`" Fact was false and that the false half named this exact site. Split rather than absorbed because `crates/tiler-conformance/**` and `docs/correctness-and-testing.md` are outside the parent's `implementation/reference` scope.
