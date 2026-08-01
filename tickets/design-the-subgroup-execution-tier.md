---
id: design-the-subgroup-execution-tier
title: Design the subgroup execution tier in the schedule vocabulary
status: in-progress
priority: p2
dependencies: [implement-the-single-workgroup-synchronized-reduction-strategy]
related: [qualify-the-simdgroup-matrix-contraction-realization, add-subgroup-memory-scope-when-collectives-land, represent-cooperative-workgroup-reduction-dataflow]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, scheduling, execution-hierarchy, subgroup]
claimed_from: todo
assignee: worker-subgroup
lease_expires_at: 1785598736
---
## User-visible outcome

The subgroup tier — CUDA warps, Metal simdgroups, WebGPU subgroups — has a target-neutral schedule representation the optimizer's four surfaces cover, so a warp-level reduction or shuffle-based combine becomes one more alternative the existing selection machinery enumerates, checks, and costs without modification.

## Why this is the next rung, and what already exists

**Fact.** The adopted [scheduled-region model](../docs/research/shapes/../../docs/research/scheduling/scheduled-region-model.md) designs the tier: `Subgroup` and `Lane` bindings distinct from per-thread vector lanes, and a `SubgroupTree(contributors, combine_steps, result_lane, masked_identity)` combine construct. None of it is implemented; the live vocabulary reaches the workgroup tier only (the 2026-08-01 cooperative tile and synchronization authority).

**Fact.** The workgroup rung's landing evidence is the pattern to repeat: representation first with verifier-owned rejections, synchronization kinds statable-and-refused until admitted, target realization as one atomic fact, Metal support downstream. `add-subgroup-memory-scope-when-collectives-land` (deferred) and the refused `Collectives` synchronization kind are the reserved seams this design activates.

**Inference — the tier's model differs from the workgroup tier in ways the design must decide rather than inherit.** A subgroup combine moves values through shuffles, not staged memory, so visibility edges and staging lifetimes may not be the right obligations; convergence within a subgroup is a different (and per-device weaker) guarantee than workgroup convergence — CUDA's independent thread scheduling and Metal's simdgroup semantics are not the same fact; and the subgroup *width* is a target fact (32/64/queried-at-runtime) that the schedule must treat as symbolic or profile-bound, never assumed.

## Questions this must decide, each with its elimination stated

- Whether `SubgroupTree` is a `ReductionTopology` sibling of `CooperativeWorkgroup` (the tile precedent) or a construct inside a cooperative phase (a subgroup combine per staged slot), and what each does to identity.
- What the subgroup analogue of a visibility edge is, if any — a shuffle has no memory to fence, but masked-lane identity and width-tail behaviour are correctness obligations something must own.
- How the subgroup width enters feasibility: an atomic realization fact per (kind, width) in the synchronization-subject shape, or a width-symbolic schedule with a profile-bound resolution — and which the optimizer's feasibility surface can answer without backend code.
- Which numerical permissions a shuffle-tree combine consumes — it is a reassociation by construction; whether masked-identity injection also requires the permutation permission.
- What the CPU story is at this tier: a fixed-vector lane fold is the model's *different* binding, and the design must say why the two stay separate (the adopted record already refuses the synonym) with the consequence for a shared combine-tree shape stated.

## Non-goals

Implementation of any construct; a Metal simdgroup realization row (`qualify-the-simdgroup-matrix-contraction-realization` owns the measured half, and its strict-profile elimination stands); any CUDA or WebGPU backend claim.

## Closes when

Each question is answered with its elimination or explicitly deferred with a trigger, the surviving representation is written with worked examples in the record beside the workgroup tier's, the public drafts it would require are enumerated for Tom, and the outcome is an accepted design, a recorded deferral, or a bounded experiment.
