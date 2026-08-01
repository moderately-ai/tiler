---
id: compose-the-two-level-subgroup-and-workgroup-reduction
title: Represent the two-level subgroup-then-workgroup reduction
status: review
priority: p2
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [design-the-subgroup-execution-tier, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, scheduling, subgroup, execution-hierarchy]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785611639
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

## Outcome (2026-08-01)

[The two-level subgroup-then-workgroup reduction](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md) is the record. All five questions are answered with their eliminations; none left a second survivor, so no fork is recorded and nothing here is Tom's to choose beyond the seven public-boundary items the record enumerates and does not self-accept. Nothing was executed, emitted, compiled, or timed; every claim is inspected source at `2aa0824` or a primary vendor specification cited by section and page.

**Two of this ticket's own premises were refuted, and both are corrected in the record rather than dropped.**

1. **The composition needs no hierarchical partition.** The question above asserts one, following the subgroup tier's §1. A threadgroup of `T = G·W` invocations each folding `k` contributors is `ContributorPartition { partitions: T, contributors_per_partition: k }` — the type the vocabulary already has, satisfying the equality `verify_cooperative_semantics` already checks against the tile's participant count. **The split is flat; only the combine is two-level**, and correcting that is what made the remaining four questions tractable.
2. **A handoff between subgroups needs *workgroup* visibility, not subgroup visibility.** The third question, ADR 0094's deferral, and the 2026-08-01 addendum on [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) all name this composition as the construct that would fire `MemoryScope::Subgroup`. It does not: the reader of a staged partial lies outside the writer's subgroup, so publication must reach across the boundary rather than stop at it. A second addendum on that ticket carries the correction with its MSL §6.16.2 and §6.10.1 evidence; the ticket stays `deferred` and its trigger line is left for a graph decision rather than rewritten here.

**The five answers.**

- **Representation:** a third `ReductionTopology` variant. The decisive ground is that the staging coverage rule inverts — a cooperative tile's staged writes are a bijection from *every* participant onto the slots, and the composition's are a bijection from `G` selected participants onto `G` slots, so one variant would carry two mutually exclusive rules keyed by an option. Two independent grounds follow: the composition derives a subgroup width equality and a prepared-pipeline preflight that `CooperativeWorkgroup` derives neither of, and a new topology tag is appends-only injective where extending the `0x35` arm needs a presence byte and a second offset argument.
- **Outer participants:** neither candidate the question names. A strided `ParticipantRange` is *insufficient* — `addressed_slots` composes the participant coordinate with an affine span, so a strided writer set would need a fractional slot stride — and a declared per-access subset is the construct the cooperative module says is "absent rather than reserved". What survives is a two-component local coordinate from which the writer set is *derived*. **The uniform-participation rule is untouched**: arrival stays every invocation, so the point stays convergent, and what narrows is the write rather than the arrival.
- **`MemoryScope::Subgroup`:** no, per the refutation above.
- **Permissions:** reassociation alone, under three stated conditions — ascending inner masks, `AscendingParticipant` outer arrival, and a contributor-block index that is the `(subgroup index, lane index)` pair. **The third is the new result.** Metal states that "threads are divided into SIMD-groups in an implementation-defined fashion" and WGSL that there is "no defined relationship" with `local_invocation_index`, so partitioning by the linear index while combining by subgroup structure consumes a permutation whose shape the schedule cannot state. The record works the composed leaf order both ways at `W = 32`, `G = 4`.
- **Identity:** inner level always, outer level never, for the admitted outer form. The inner width is imposed so a contributor-free lane is the general case; the outer participant count is chosen and equals the staged slot count, so outer coverage is exact by construction. A second width-`W` shuffle tree at the outer level would reintroduce the obligation and is deferred with a trigger.

**Worked example C** is the subgroup and CPU tiers' own program at `W = 32`, `T = 128`, `G = 4`, priced beside examples A and B — 16 bytes of workgroup memory against B's 404, one barrier, 27 padding positions confined to one simdgroup — plus the `S = 8,192` attention row as derived operation counts, labelled as counts rather than timings.

**A finding worth carrying out of the record.** The Metal specification's own example kernel, quoted verbatim at §6.10.2.1 pages 224–225, does not execute its staging loop for any threadgroup smaller than `W·(W+1)` threads (1,056 at `W = 32`), and the printed program then delivers at most one SIMD-group's partial to the atomic; in its multi-round form it also carries a write-after-read hazard on the staged allocation with only one barrier per round. Both are derivations over printed source rather than executions, and each maps onto a rule the implemented vocabulary already names — which is the concrete argument for this ticket's stated outcome that the composition be one schedule a verifier checks whole.

**Filed:** [`land-the-two-level-reduction-adr`](land-the-two-level-reduction-adr.md), which carries the byte-identical ADR body drafted inside the record, the ADR's catalog row, and the research record's catalog row — three files this ticket's `research/scheduling` scope cannot reach. The drafted body says `0096` because `0095` was highest at `2aa0824`; the carrier re-reads the directory.
