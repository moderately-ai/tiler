---
id: compose-the-two-level-subgroup-and-workgroup-reduction
title: Represent the two-level subgroup-then-workgroup reduction
status: todo
priority: p2
dependencies: [accept-the-subgroup-execution-tier-adr]
related: [design-the-subgroup-execution-tier, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, scheduling, subgroup, execution-hierarchy]
---
## User-visible outcome

A reduction whose contributor sequence is longer than one subgroup has a representation: several subgroups reduce internally through shuffles, their result lanes stage partials through workgroup memory, and one final combine produces the output — as one schedule the verifier checks whole, rather than as two schedules nothing relates.

## Why this is its own ticket

**Fact.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md) answers its ticket's five questions for a reduction owned by *one* subgroup and states in §1 that the two-level composition is out of scope, is neither a `SubgroupTree` nor a `CooperativeWorkgroup`, and needs a hierarchical partition the vocabulary does not have. **Fact.** The adopted [scheduled-region model](../docs/research/scheduling/scheduled-region-model.md) states that each reduction domain has exactly one topology, so the composition cannot be spelled by naming both.

**Fact.** This is the shape the Metal Shading Language Specification's own example reduction kernel uses (§6.10.2.1): a per-SIMD shuffle fold, `if (simd_lane_id == 0) ldata[simd_group_id] = val;`, a `threadgroup_barrier`, then a second shuffle fold over the staged partials.

**Inference.** It is also the first shape a realistic softmax needs. [The first attention program vertical](../docs/research/program-planning/first-attention-program-vertical.md) records that under its zero-synchronization schedule profile "a SIMD-group-cooperative row reduction survives; anything wider does not", so the first attention row longer than one subgroup fires this.

## Questions this must decide

- Whether the composition is a third `ReductionTopology` variant, a nesting field on the cooperative one, or a hierarchical `ContributorPartition` that both existing variants read — and what each does to `CanonicalScheduledRegionIdentity`.
- Whether the outer level's participants are the subgroup result lanes (a strided subset of the workgroup, which `ParticipantRange` cannot express, since it is a contiguous run) or every invocation with a predicate — and what that does to the tile's uniform-participation rule, which is what makes its synchronization point convergent.
- Whether the staged handoff between simdgroups needs `MemoryScope::Subgroup`, which [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) reserves. The subgroup tier established that a shuffle tree does not; a staged handoff *between* simdgroups is the construct that might.
- Which permissions the composition consumes, given that both levels are reassociations and the subgroup tier established that the leaf order decides permutation. A two-level tree's leaf order is the composition of the two, and it is not obviously ascending.
- Whether the identity obligation applies at one level or both, given that the inner width is imposed and the outer participant count is chosen.

## Non-goals

Implementation. Any Metal, CUDA, or WebGPU backend claim. Re-deciding anything the subgroup tier ADR settles.

## Closes when

Each question is answered with its elimination or explicitly deferred with a trigger, the surviving representation is written with a worked example beside the two the subgroup tier record carries, and the outcome is an accepted design, a recorded deferral, or a bounded experiment.
