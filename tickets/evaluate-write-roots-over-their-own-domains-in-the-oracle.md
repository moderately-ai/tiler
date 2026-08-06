---
id: evaluate-write-roots-over-their-own-domains-in-the-oracle
title: Evaluate write roots over their own domains in the oracle
status: in-progress
priority: p1
dependencies: [state-the-oracle-boundary-for-sub-domain-write-roots]
related: [lower-the-concatenate-occurrence-through-partitioned-writes, admit-sub-range-write-domains-for-unequal-partitions, decide-the-index-region-oracle-route-past-its-step-budget]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, oracle, indexing]
claimed_from: todo
assignee: agent-root-domains
lease_expires_at: 1785997200
---
## User-visible outcome

The reference oracle evaluates a partitioned output whose roots iterate different sub-domains, so a concatenation of unequally sized operands — including one that is empty — has an independent correctness oracle rather than only the verifier's own proof.

## Why this exists

**Fact — the evaluator has one walk and every root fires at every point of it.** `stage` builds one `ParallelWalk` from `parallel_domain()` (`crates/tiler-reference/src/oracle.rs:1424`, `:2006-2015`), which is the region's whole parallel dimension set read from `dimensions()` and never from an access. `evaluate_point` (`:2145-2183`) then seeds the frame with that point and evaluates **every** root of **every** plan at it.

**Fact — a sub-domain root has no correct behaviour under that walk.** Its coordinates cannot name the dimensions its domain omits (`IndexBuildError::CoordinateOutsideAccessDomain`), so at every full parallel point that agrees on the root's own dimensions it computes the same element — a duplicate — and a root over a zero-extent dimension zeroes the whole parallel product so that no root fires at all.

**Fact — the IR admits exactly those regions now.** [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) admits a write domain that is any subset of the parallel dimensions, and the construct it admits gives each root its own iteration space with the region's parallel set as their union.

**Inference — the refusal this ticket's dependency states is the honest interim, not the destination.** [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) emits precisely a sub-domain-rooted region at its pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence; while the oracle refuses that shape, the lowering has the verifier's proof and no independent check of it.

## What the work is

Walk each root over **its own** domain instead of walking one shared parallel space, and restate the span argument over the unit that then makes sense.

The unit question is the substance and must be decided rather than assumed. The current public surface counts and spans *parallel points* (`parallel_point_count`, `:1548-1558`; `evaluate_points`; `evaluated_points`), and the span-safety argument at `:1453-1486` says a partition of the parallel points is a partition of each output's elements — which is true only while every root iterates all of them. Under per-root domains the entity in bijection with an output's elements is the (root, root-domain point) pair, which the existing doc already names at `:1475-1477` as the general case. Decide whether the public unit becomes that pair, and say what a caller's existing division by `parallel_point_count` then means.

Keep the per-element `DuplicateWrite` and `IncompleteWrite` checks exactly where they are, over one buffer per output boundary. They are the oracle's own joint obligation, independent of the verifier's ownership proof, and they are what makes this an oracle rather than a second reading of the proof. `IncompleteWrite`'s attribution to `plan.roots.first()` (`:1987-1993`) needs revisiting under unequal roots, because the first root is no longer a meaningful blame target for a gap.

## Explicit non-goals

- The refusal this supersedes, which is the dependency and lands first so no window exists in which the oracle silently mis-evaluates.
- Symbolic extents, which this evaluator already refuses under `SymbolicDimensionExtent` and which are a separate question on the IR side.

## Closes when

A region whose roots partition one output into unequally sized contiguous pieces — including a zero-extent member — evaluates to the correct tensor; the superseded `UnsupportedRegionFeature` refusal is removed rather than left unreachable; a deliberate perturbation of one root's offset is shown refusing under `DuplicateWrite` or `IncompleteWrite`; and the staged-span argument is restated over the unit actually walked, with a test that spans of that unit compose to the same result as one span.

## Graph maintenance

- `implementation/reference` alone.
- Filed by [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md), which could not reach `crates/tiler-reference/`.
- Related rather than a declared dependency of the concatenate lowering: the lowering plainly wants an oracle for the region it emits, but whether its own Closes-when routes through `IndexRegionEvaluator` was not verified when this was filed, and a hard edge asserted on an unread path would block that ticket on a claim nobody checked.
