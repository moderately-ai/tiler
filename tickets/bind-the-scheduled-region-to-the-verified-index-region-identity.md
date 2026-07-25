---
id: bind-the-scheduled-region-to-the-verified-index-region-identity
title: Bind the scheduled region to the verified index region's identity
status: review
priority: p1
dependencies: []
related: [update-adr-0071-schedule-builder-boundary, prototype-scheduled-region-ir, prototype-canonical-index-region-slice]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, ir, identity, scheduling]
claimed_from: todo
assignee: agent-shapes2
lease_expires_at: 1785004839
---
ADR 0071's Decision requires that "each verified structural layer retains the exact identity of the lower structural layer it refines: schedule to index region and kernel to schedule". One of those two edges exists and one does not.

**Fact — kernel to schedule is realized.** `crates/tiler-ir/src/kernel/model.rs` stores `schedule_identity: CanonicalScheduledRegionIdentity` on the verified kernel, exposes it through an accessor documented as "the canonical identity of the scheduled region this refines", and folds its bytes into `CanonicalKernelIdentity` in the identity encoder. A kernel therefore names the exact schedule it refines, and two kernels over different schedules cannot collide.

**Fact — schedule to index region is not.** `crates/tiler-ir/src/schedule/model.rs` declares its own `IndexRegion`: a public-field struct holding `id`, `iteration_shape`, `accesses`, `bounds_proofs`, `ownership_proof`, `scalar_program`, and `numerical`. That is a different type from `tiler_ir::index::VerifiedIndexRegion`, which is an `Arc`-backed compacted arena of dimensions, tensors, index expressions, accesses, scalar operations, values, and outputs behind a separate `CanonicalIndexRegionIdentity`. `encode_identity` in the schedule module folds the schedule-local struct's *content* into `CanonicalScheduledRegionIdentity` and never references `CanonicalIndexRegionIdentity`.

**Fact — the two modules do not meet.** The exact check is `grep -rn 'crate::index' crates/tiler-ir/src/schedule/`, which returns exactly one line: a doc-comment cross-reference in `error.rs` saying the error boundaries "mirror the `crate::index` discipline". There is no code path from the schedule module into the index module. `ScheduledRegionBuilder::from_region` takes a `ScheduledRegion` and destructures its schedule-local `index` field; nothing in the crate converts a `VerifiedIndexRegion` into one.

**What this costs.** Two representations of one layer with two canonical identities. A schedule's identity is a function of index-region content it re-encodes itself, so the index layer's own verifier, its compaction, and its identity derivation contribute nothing to it — and a schedule can be verified over index content that the index verifier would have rejected, because the schedule verifier is a separate authority checking its own struct. ADR 0071 exists precisely to stop a second verifier authority from forming.

## What this ticket must decide before it codes

Whether the duplication is a defect or a deliberate asymmetry. Read both modules in full before assuming the former. The plausible defence is that the schedule layer's `IndexRegion` is the *scheduled* view — an iteration domain plus proofs, sized for what a launch needs — while `tiler_ir::index` is the canonical scalar-region slice sized for what a scalar program needs, and that collapsing them would force one representation to carry both. If that defence holds, ADR 0071's "schedule to index region" clause is what needs the correction, not the code, and the correct outcome is a `contracts/decisions` follow-up rather than an IR change.

If it does not hold, the shape to reach for is the one the kernel layer already uses: `VerifiedScheduledRegion` retains a `CanonicalIndexRegionIdentity`, the schedule builder is seeded from a `VerifiedIndexRegion` rather than from a freely constructible struct, and `encode_identity` folds the retained identity instead of re-encoding content. Note that this makes the schedule builder's public entry point stricter, which is an ADR 0075 always-ask category if it changes an existing public signature.

## Closes when

The question above is answered with evidence from both modules; either the identity edge exists with a test proving two schedules over different index regions cannot share identity, or a `contracts/decisions` ticket carries the ADR 0071 correction with the asymmetry argument. ADR 0071's "Partially realized clause — retained lower-layer identity" paragraph is updated either way, which needs `contracts/decisions` added here or split.

## Outcome

**Answered, and the answer is neither of the two the ticket offered. No code changed, deliberately.** The duplication is a real defect, but the edge this ticket proposes cannot be built, because the layering ADR 0071 asserts is not the layering the compile path has. Both branches of "Closes when" therefore resolve to the `contracts/decisions` correction, split below, plus a compile-path ticket that is the actual work.

### The decisive fact: the cardinality is wrong

**Fact — the schedule does not refine *one* verified index region; it covers *several*, and on some paths none.** `crates/tiler-compiler/src/physical.rs` builds every `ScheduledRegion` by struct literal from a `VerifiedTargetRequest` — `pointwise_region`, `reduction_region`, and `fused_region` each return `(ScheduledRegion, Vec<SemanticMemberId>)`. A fused region covers several members. Separately, `crates/tiler-compiler/src/legality.rs::emit_region` drives `tiler_ir::index::IndexRegionBuilder` and returns a `VerifiedIndexRegion` **per semantic occurrence**. So the compile path produces both kinds of region, on different axes: one scheduled region per region candidate, one verified index region per occurrence.

A `schedule_identity`-shaped field cannot express that. The kernel-to-schedule edge works because a kernel refines exactly one schedule. A single `CanonicalIndexRegionIdentity` on a scheduled region would have to name one of N refinements or none, and either would be a false statement about what the schedule rests on.

Reproducible checks: `grep -rn 'VerifiedIndexRegion' crates/tiler-compiler/src/` returns matches only in `legality.rs` and `capability.rs`, never in `physical.rs`; `grep -rn 'crate::index' crates/tiler-ir/src/schedule/` returns three doc comments and no code path.

### The real cost, restated correctly

**Fact — `CanonicalIndexRegionIdentity` reaches no verified product's identity anywhere.** It is derived in `emit_region`, carried on `IndexRefinement`, and consumed by `crates/tiler-compiler/src/pipeline.rs::refinement_label`, which slices `identity().as_bytes()` to format an `EXPLAIN` string. The kernel program's stage coverage is `Vec<SemanticOccurrence>` (`crates/tiler-ir/src/program/model.rs:231`, folded at `:882`) — semantic occurrences, not refinement identities.

So the index layer's verifier, compaction, and identity derivation contribute to explain output and to nothing else. That *is* the harm ADR 0071's clause exists to prevent, and this ticket's framing located it one layer too low. The place a refinement identity belongs is the program stage's coverage, where the cardinality is already 1:N and where a stage already claims which occurrences it implements. Binding coverage to refinement identity would make a stage name the exact verified index regions it rests on; the schedule layer would remain what it is.

### Why the schedule's own struct still should not absorb `VerifiedIndexRegion`

Verified independently of the above and agreeing with `unify-schedule-index-region-with-verified-index-region`'s recommendation. `schedule::ScalarProgram` is a closed three-variant enum of `f32` bit-pattern records and `schedule::LogicalAccess` is a closed pair; `index`'s scalar program is an open registry-governed SSA graph over `ScalarOpKey` with symbolic coordinate expressions. Neither type is a subset of the other in *both* directions either: `schedule::TensorRole` has an `Intermediate` variant that `index::TensorRole` (`Input | Output` only) cannot express, and the schedule carries a `NumericalRealization` the index layer has no field for. Merging them would push an open vocabulary into the physical layer, which AGENTS.md separates explicitly — "keep semantic/logical IR, symbolic access relations, fusion alternatives, physical schedules … distinct".

### Retraction

The ticket's own framing — "one of those two edges exists and one does not", implying the missing one is buildable in the same shape — is what I set out to implement, and it does not survive reading `physical.rs`. Recording that rather than adding a field that would have compiled, passed a same-shaped test, and asserted something false.

### Split

- `correct-adr-0071-retained-lower-layer-identity-cardinality` — `contracts/decisions`, which this ticket does not hold. ADR 0071's Decision clause and its "Partially realized clause" boundary entry both need the correction.
- `bind-stage-coverage-to-index-refinement-identity` — the implementation, in `implementation/ir` and `implementation/compiler`.
