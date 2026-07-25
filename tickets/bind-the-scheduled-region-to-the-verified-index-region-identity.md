---
id: bind-the-scheduled-region-to-the-verified-index-region-identity
title: Bind the scheduled region to the verified index region's identity
status: todo
priority: p1
dependencies: []
related: [update-adr-0071-schedule-builder-boundary, prototype-scheduled-region-ir, prototype-canonical-index-region-slice]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, ir, identity, scheduling]
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
