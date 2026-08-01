---
id: admit-loop-carried-cooperative-staging
title: Admit loop-carried cooperative staging so a reused tile is expressible
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, represent-cooperative-workgroup-reduction-dataflow, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir]
shared_scopes: []
paths: []
tags: [research, physical-planning, contraction]
claimed_from: todo
assignee: worker-loop-staging
lease_expires_at: 1785610106
---
## User-visible outcome

A cooperative tile can stage into one fixed allocation, hand it off behind a barrier, and then *reuse the same slots* for a later round — the shape every blocked-tile GPU kernel has and the one `CooperativeTile` deliberately does not model. Until it exists, the L3-selected `tiled` contraction is unstatable at any extent the pinned workload uses.

## Why this is filed as its own node

[`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) stopped here a second time, on a blocker strictly narrower than its first. The synchronization authority it originally waited on has landed in full: `CooperativeTile`, `SynchronizationPoint`, the KIR `Barrier`/`StagedStore`/`StagedLoad` constructs, the single-workgroup tree strategy, and — at `c81e5c2`'s successor — Metal emission of a staged, fenced kernel that compiles and links on the Apple toolchain. What remains is one modelling gap, and it is already named in the source that has it.

**Fact — the module states the gap and calls it unmodelled.** `crates/tiler-ir/src/schedule/cooperative.rs:41-47`: "A tile that rewrote one slot across several rounds — a logarithmic tree — is statable in this vocabulary but is refused: [`CooperativeTile`] admits one writer per slot, because a second write to a live slot needs a per-round lifetime and a per-round visibility edge that this profile does not yet model." The same paragraph records that this is what caps `workgroup_tree_tile` at depth two rather than `log2(participants)`.

So one missing capability now blocks two independent consumers — the log-depth tree and the tiled contraction — which is what makes it a node rather than a line item inside either.

## The three things missing, each with its exact check

**1. Slots are single-assignment across the whole tile.** `verify_cooperative_tile` builds one occupancy map spanning every phase and refuses a second write to any slot (`crates/tiler-ir/src/schedule/builder.rs:1192-1204`, `CooperativeTileRule::StagingConflict`, rule id `cooperative-staging-conflict`). Coverage is checked over the same map (`builder.rs:1205-1210`), so the two are one statement: the participants' writes are a bijection onto the allocation's slots. A round-reusing tile violates it by construction.

**2. Only producer-to-consumer edges are derivable.** `CooperativeTile::visibility_edges` (`cooperative.rs:371-402`) emits an edge only where `producer.id < consumer.id`, i.e. read-after-write. A reused tile also needs the *anti*-dependency — round `r+1`'s write must not overtake round `r`'s read — and a point declared to order it discharges no derived edge, so `verify_synchronization` rejects it as `SynchronizationRule::RedundantPoint` (`builder.rs:1323-1327`). This is a new evidence class, not a new field: the vocabulary can state no write-after-read obligation at all.

**3. A barrier may not sit inside a loop.** `verify_synchronization` refuses any barrier at nonzero block depth (`crates/tiler-ir/src/kernel/verify.rs:400-405`, `KernelDiagnostic::SynchronizationConvergence`), documented at `verify.rs:361-363` as "A barrier inside a predicate or a loop is reached by a dynamic subset of the participants — undefined execution rather than unsupported." **Inference — the rule is sound for a predicate and conservative for a loop.** `SerialLoopSpec` carries `start` and `end` as `u64` *literals* (`crates/tiler-ir/src/kernel/model.rs:685-692`), not values, so every invocation of a workgroup executes an identical trip count and a barrier in that body is reached by all of them at the same dynamic instance. The walk already tracks `loop_depth` separately from `block_depth` (`verify.rs:642-673`), so the distinction the sound rule needs is present and merely not used here. This is the one of the three that may be a narrowing of an existing check rather than new vocabulary — establish that before assuming it.

## Why unrolling is not the way around it

**Measurement — the numbers, at the pinned workload's own extents.** The `tiled` realization uses two 16×16 `f32` allocations, 2,048 bytes total, reused across `K/16` rounds. Giving each round its own allocations to satisfy rule 1 above:

| Contracted extent | Rounds | Phases needed | Staging slots | Threadgroup bytes |
| --- | --- | --- | --- | --- |
| 1024 | 64 | 128 | 32,768 | 131,072 |
| 2048 | 128 | 256 | 65,536 | 262,144 |
| 3072 | 192 | 384 | 98,304 | 393,216 |

Against `MAX_COOPERATIVE_PHASES = 64` and `MAX_COOPERATIVE_STAGING_SLOTS = 65,536` (`crates/tiler-ir/src/schedule/mod.rs:207`, `:205`), and against the 32,768-byte `LocalMemoryBytes` row the widened test profile declares (`crates/tiler-compiler/src/target.rs:3575`; the *governed* baseline declares 0, `target.rs:1686`). Every cell exceeds the phase bound; K=3072 exceeds the slot bound; all three exceed the memory row by 4× to 12×.

**Inference — and the performance claim would be fabricated anyway.** The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) attributes `tiled`'s 2.6×–4.3× prefill advantage to the staging, measured on a kernel holding 2 KB resident. A 128 KB variant is a different kernel with different occupancy, so its numbers are unmeasured whatever a bound says.

## Scope

Owns the vocabulary and its verification: whatever states a per-round staging lifetime, whatever states a write-after-read obligation and what discharges it, and the barrier-convergence rule's treatment of a constant-trip loop. It does **not** own the tiled contraction schedule, its `K` precondition, or its Metal emission — [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) keeps those and resumes on this.

## The identity question this must answer first, and it is Tom's

**Inference — at least one of the three changes moves retained identity bytes, and that was not true before the tree strategy landed.** `push_workgroup_staging` (`crates/tiler-ir/src/schedule/model.rs:1605-1611`) writes a fixed, unframed field sequence, so a per-round lifetime field lands at a fixed offset with no tag and no length and moves every cooperative tile's bytes — stepping `tiler.schedule.v3`, and the kernel and feasibility domains that fold it.

`push_cooperative_tile`'s own comment (`model.rs:1676-1679`) argues the `0x35` payload was safe to extend because "no cooperative region has ever been encodable into a retained identity — the structured-kernel verifier refused every one before a kernel, program, artifact, or cache entry could hold it." **That premise has since expired.** `implement-the-single-workgroup-synchronized-reduction-strategy` carries a cooperative region through planning to a verified kernel and executes it (`the_tree_matches_the_reference_at_its_declared_order_for_every_extent`, `crates/tiler-compiler/src/pipeline/tests.rs:3765`), and this repository now checks in a cooperative Metal golden. Re-derive the premise against the tree at the time this is picked up rather than inheriting either answer.

Design the representation so the common case is an *append* if that is reachable — a distinct tag, or a lifetime expressed as something the encoder already frames — and take the domain step to Tom explicitly if it is not. Per [AGENTS.md](../AGENTS.md), the new evidence class in item 2 is a validation authority and a consequential public boundary (`VisibilityEdge`, `WorkgroupStaging`, `SynchronizationRule`, `CooperativeTileRule` are all public), so its shape is Tom's decision regardless of how good the derivation is.

## Closes when

A cooperative tile that writes an allocation, hands it off behind a point, and rewrites the same slots in a later round verifies; the anti-dependency is derived rather than declared and has exactly one discharging point; a body realizing it passes the structured-kernel verifier; each new rule has been watched refusing its own defect; and the identity consequence is recorded with whichever of an append or an accepted domain step it turned out to be.
