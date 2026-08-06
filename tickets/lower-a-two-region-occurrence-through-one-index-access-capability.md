---
id: lower-a-two-region-occurrence-through-one-index-access-capability
title: Lower a two-region occurrence through one index-access capability
status: in-progress
priority: p1
dependencies: [admit-a-multi-region-index-realization-law]
related: [admit-the-rms-normalization-family, admit-the-softmax-family, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, lowering, capability, normalization]
claimed_from: todo
assignee: agent-two-region-2
lease_expires_at: 1786000442
---
## User-visible outcome

An operation whose realization needs more than one index region — a reduction that produces a shared intermediate, then an elementwise pass that consumes it — can resolve an index-access lowering capability, so that a normalization or a softmax is not held below R6 by the shape of the lowering vocabulary rather than by anything about the family.

## Why this is filed

**Fact.** `admit-the-rms-normalization-family` registered `tiler::rms-norm-f32@1` with a fusion role, a numerical capability row, structured-kernel constructs, and a Metal emission, and deliberately registered **no** `GovernedIndexAccess` row. The reason is structural rather than a deferral of effort.

**Inference — one region cannot express the occurrence.** `IndexAccessLoweringProvider::lower` emits one index region per occurrence, and a region evaluates one scalar expression per point of one iteration domain. RMS normalization is shape-preserving, so its output domain is the whole tensor, while its reduction's result is shared by every point of a normalized row. Emitting it as one region would re-evaluate the whole fold at every point: at the workload's extent of 1024 that is an unrolled expression of about 1024 nodes per output point and about 10⁶ nodes per row, which the index region's structural bounds refuse long before it becomes merely slow.

> **Correction (2026-08-05, from the dispatch that hit the discovery stop).** The paragraph above is true about the trait and false about the wall. It is retained rather than deleted because the *reasoning* is what the blocking ticket inherits — the occurrence genuinely needs two regions — but the attribution was wrong, and a reader acting on it would widen a surface that changes nothing. See the Outcome section.

**Fact — the two-region shape already exists elsewhere.** The physical planner's materialized serial-sum path plans exactly this: a reduction region writing an intermediate, then a pointwise region reading it. What is missing is a way for a *capability* to describe it, so an occurrence that needs two regions currently resolves nothing and fails closed.

> **Correction (2026-08-05).** The first sentence is true but names a different IR. That path is `KernelSubprogram` / `SubprogramStage` over `VerifiedScheduledRegion` (the physical/schedule IR, `crates/tiler-compiler/src/frontier.rs`); capabilities emit `tiler_ir::index::VerifiedIndexRegion` (the index-refinement IR). The second sentence's "what is missing" is therefore wrong: what is missing is upstream of the capability, in the realization law and receipt. Conflating the two IRs is precisely what made this ticket look reachable from `implementation/compiler`.

## Non-goals

Widening `select_supported_strategy`, which [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md) owns; choosing between a fused and a materialized plan, which is selection's; and any new scalar key beyond what the two regions already emit.

## Closes when

1. A lowering capability can declare that it emits an ordered sequence of regions with a named intermediate between them, and the intermediate's shape, ownership, and lifetime are explicit physical contracts rather than implied by the order.
2. The capability's declared emitted-scalar set covers every region it emits, so the refinement containment check still sees the whole realization.
3. `tiler::rms-norm-f32@1` resolves a capability, and a deliberate perturbation — a capability declaring one region for a two-region occurrence — refuses with a typed reason rather than emitting a truncated realization.
4. The explain output names both regions and the intermediate, because a reader asking why an occurrence produced two dispatches must not have to infer it from the dispatch count.

## Outcome

**Why this is parked rather than delivered.** The ticket's premise is falsified by the tree it was filed against: nothing in `implementation/compiler` can deliver its stated outcome, because the refusal that holds `tiler::rms-norm-f32@1` fires in `implementation/ir` before any compiler-side lowering surface is reached. Widening `IndexAccessLoweringProvider` to emit a region sequence would have been a surface with no consumer and no verifier — a type-system reservation shaped like implemented support. Parked at `blocked` behind [`admit-a-multi-region-index-realization-law`](admit-a-multi-region-index-realization-law.md), which owns the authority this ticket needs.

**Fact.** `crates/tiler-ir/src/semantic/registry.rs` registers an `IndexRealizationLaw` for exactly nine operations; the normalization and the softmax are deliberately absent, and the comment above the list says absence "fails closed later".

**Fact.** `refine_index_region` (`crates/tiler-compiler/src/legality.rs`) calls `realizations.resolve(subject)` *before* `emit_region`. For a lawless family `resolve` returns `IndexRefinementVerificationError::MissingRealizationLaw`, so the resolved provider is never driven.

**Measurement.** `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs`, four tests, run on this branch with `cargo nextest run -p tiler-compiler -E 'binary(two_region_occurrence_lowering_wall)'` → `4 tests run: 4 passed`. `refining_the_normalization_refuses_before_the_provider_is_driven` registers a normalization index-access capability whose provider increments an `AtomicUsize`, refines, and observes the error `IrVerifier(MissingRealizationLaw)` with a driven count of exactly `0`. The control `a_family_carrying_a_law_refines_through_the_identical_harness` refines `tiler::multiply-f32@1` through the same harness, so the refusal is attributable to the absent law rather than to the fixture.

**Measurement — the checks can say no.** Each assertion was perturbed and observed failing: resolving `multiply_subject()` instead of the normalization (the `expect_err` panics on `Ok`); pointing the recording provider at `tiler::multiply-f32@1` (observed `Emit(Occurrence { rule: "fixture-never-reached" })`, which also demonstrates that a family *with* a law does drive its provider); both driven-counter expectations flipped to `1` (observed `left: 0, right: 1`, proving the counters read a real zero rather than an absent value); the control pointed at the normalization (observed `IrVerifier(MissingRealizationLaw)`); and the resolved-operation assertion pointed at the multiply key.

**Inference — why the sequence vocabulary cannot start at the capability.** `IndexRealizationLaw::realize` returns one `VerifiedIndexRegion` and `ResolvedIndexRealization::verify` requires the candidate's canonical identity to equal the law's own reconstruction. A region sequence has no canonical identity that comparison can consume, and `IndexRefinementReceipt` binds one region's operands and results. So closes-when 1–4 above are all downstream of the law and receipt layer.

**Fact — what was deliberately not built.** No public boundary was changed. `crates/tiler-compiler/src/capability.rs` and `src/legality.rs` are untouched; the branch adds one integration test and ticket-graph edits. Identity-pin survey (`grep -rnoE '\b[0-9a-f]{16}\b'` and the `{64}` form over `crates/tiler-compiler`): 21 distinct 16-hex pins — 1 live at `explain.rs:4090` (`request=8e06e11fdc3a2889`) plus 20 historical values in its ledger comment — and 6 distinct 64-hex pins. None moved: the live pin hashes the request subject, and this branch adds no program shape to a request and no registry entry.

**Remaining reachable work in this scope.** None until the dependency lands. When it does, closes-when 1–4 are reachable from `implementation/compiler` as written, and the explain slot for closes-when 4 is already identified: the `capability.index-access-resolution.v1` record in `crates/tiler-compiler/src/pipeline/trace.rs` currently carries zero `ExplainFact`s, and `crates/tiler-compiler/src/pipeline/tests.rs` holds a rule census that a new record or changed count fails first.

## Unparked — 2026-08-06

The blocking dependency (`admit-a-multi-region-index-realization-law`) is done: `StagedStrictSerialSumThenPointwiseF32` and `VerifiedIndexRegionSequence` exist with domain-separated identity, and the wall tests in `crates/tiler-compiler/tests/two_region_occurrence_lowering_wall.rs` pin exactly what this ticket now flips. Two items travel with the unparking: the acceptance node `accept-the-multi-region-index-realization-surface` is parked for Tom (this ticket's work is a draft consumer until he rules), and the law worker put the open accessor question — whether a staged receipt should expose a single-region accessor at all — to this ticket to answer at its own boundary.
