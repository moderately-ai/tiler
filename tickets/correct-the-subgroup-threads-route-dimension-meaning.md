---
id: correct-the-subgroup-threads-route-dimension-meaning
title: Correct what RouteResourceDimension::SubgroupThreads means
status: in-progress
priority: p2
dependencies: []
related: [design-the-subgroup-execution-tier]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, subgroup, defect, public-boundary]
claimed_from: todo
assignee: worker-subthreads
lease_expires_at: 1785603010
---
## User-visible outcome

The one live-device route dimension the artifact vocabulary carries names a property a device can actually be asked about, compared by a relation that is sound for the routes that state it.

## The defect, in two separable halves

**Fact.** `RouteResourceDimension::SubgroupThreads` (`crates/tiler-artifact/src/program/requirement.rs`) is documented as "Threads one subgroup must execute in lockstep for the route to be correct", and its satisfaction test is `is_satisfied_by(observed) = self.minimum <= observed` — a floor.

**Half one: the stated property is not one current GPU families provide.** **Fact — CUDA Programming Guide.** "In GPUs of compute capability 7.0 and later, *independent thread scheduling* allows full concurrency between threads, regardless of warp", and "*Warp-synchronous* code assumes that threads in the same warp execute in lockstep at every instruction, but the ability for threads to diverge and reconverge at sub-warp granularity makes such assumptions invalid." A floor over "threads that execute in lockstep" therefore bounds a quantity no adapter can soundly observe. That every implemented adapter answers `Unrecognized` is consistent with this, though they answer it for the different reason that Metal publishes no device-scoped width.

**Half two: a floor is the wrong relation for the route that would state it.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md) §3 derives that a width-`W` shuffle tree is sound on a wider device only if lanes `0..W` of each subgroup are all active, and that conjunct is exactly what a floor does not carry. What such a route needs is an equality on the width together with a full-participation obligation. This is the same shape as the argument [CPU vector realization facts](../docs/research/target-profiles/cpu-vector-realization-facts.md) makes for why a lane width "looks quantitative and is not".

**This is a live defect in landed public vocabulary and is independent of whether the subgroup tier is accepted.** No route states the row today, so nothing is currently wrong at run time — but the vocabulary is what a future route would reach for, and it is deliberately not `#[non_exhaustive]` precisely so that changing it is a build error at every adapter.

## What to decide

- Whether the fix is a corrected doc comment plus an equality relation, a renamed dimension, or removing the dimension until a route actually states one — noting that removal is cheapest now and most expensive later, and that the family's own module doc argues the dimension is "the one that survives" a derivation.
- What an adapter is being asked to observe, stated so that an adapter can answer it rather than answering `Unrecognized` — which on Metal means confronting that `threadExecutionWidth` is a prepared-pipeline property and not a device one.
- Whether the full-participation obligation belongs in this vocabulary at all, or stays a schedule-side intrinsic obligation with only the width crossing the artifact boundary.

## Public boundary

`RouteResourceDimension` is `pub` in `tiler-artifact` and is deliberately not `#[non_exhaustive]`. Any change to its variants, its comparison relation, or its wire tag is Tom's, and a wire-tag change is an artifact identity step.

## Non-goals

Implementing a subgroup route. Declaring a Metal subgroup width. Anything in `tiler-ir`'s schedule vocabulary.

## Closes when

The dimension's documented meaning matches a property an adapter can observe, its comparison relation is sound for the routes that would state it or the dimension is removed with the reason recorded, and the decision on the public boundary is Tom's rather than self-accepted.
