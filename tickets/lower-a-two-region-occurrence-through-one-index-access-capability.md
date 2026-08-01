---
id: lower-a-two-region-occurrence-through-one-index-access-capability
title: Lower a two-region occurrence through one index-access capability
status: todo
priority: p1
dependencies: []
related: [admit-the-rms-normalization-family, admit-the-softmax-family, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, lowering, capability, normalization]
---
## User-visible outcome

An operation whose realization needs more than one index region — a reduction that produces a shared intermediate, then an elementwise pass that consumes it — can resolve an index-access lowering capability, so that a normalization or a softmax is not held below R6 by the shape of the lowering vocabulary rather than by anything about the family.

## Why this is filed

**Fact.** `admit-the-rms-normalization-family` registered `tiler::rms-norm-f32@1` with a fusion role, a numerical capability row, structured-kernel constructs, and a Metal emission, and deliberately registered **no** `GovernedIndexAccess` row. The reason is structural rather than a deferral of effort.

**Inference — one region cannot express the occurrence.** `IndexAccessLoweringProvider::lower` emits one index region per occurrence, and a region evaluates one scalar expression per point of one iteration domain. RMS normalization is shape-preserving, so its output domain is the whole tensor, while its reduction's result is shared by every point of a normalized row. Emitting it as one region would re-evaluate the whole fold at every point: at the workload's extent of 1024 that is an unrolled expression of about 1024 nodes per output point and about 10⁶ nodes per row, which the index region's structural bounds refuse long before it becomes merely slow.

**Fact — the two-region shape already exists elsewhere.** The physical planner's materialized serial-sum path plans exactly this: a reduction region writing an intermediate, then a pointwise region reading it. What is missing is a way for a *capability* to describe it, so an occurrence that needs two regions currently resolves nothing and fails closed.

## Non-goals

Widening `select_supported_strategy`, which [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md) owns; choosing between a fused and a materialized plan, which is selection's; and any new scalar key beyond what the two regions already emit.

## Closes when

1. A lowering capability can declare that it emits an ordered sequence of regions with a named intermediate between them, and the intermediate's shape, ownership, and lifetime are explicit physical contracts rather than implied by the order.
2. The capability's declared emitted-scalar set covers every region it emits, so the refinement containment check still sees the whole realization.
3. `tiler::rms-norm-f32@1` resolves a capability, and a deliberate perturbation — a capability declaring one region for a two-region occurrence — refuses with a typed reason rather than emitting a truncated realization.
4. The explain output names both regions and the intermediate, because a reader asking why an occurrence produced two dispatches must not have to infer it from the dispatch count.
