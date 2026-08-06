---
id: derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order
title: Derive the exact evaluator for a multi-round cooperative fold order
status: deferred
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ, accept-adr-0100-multi-round-reduction-composition]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, reductions, conformance]
---
## User-visible outcome

An exact host evaluator for every reduction order a schedule can declare, so that a plan whose topology the reference cannot evaluate is refused by name rather than compared against the nearest order that happens to exist.

## Why this is a deferral rather than work

**Fact — `strict_partitioned_sum` expresses exactly one realization shape.** It folds `partition * chunk + within` (`crates/tiler-reference/src/evaluate.rs:484`), i.e. blocked uniform contiguous partitions, serial within a partition, ascending across them, at the element width. `ContributorPartition::covers` admits only an exact product, so a non-uniform split is unrepresentable by construction.

**Fact — three declared shapes fall outside it.** `ReductionTopology::CooperativeWorkgroup` (`crates/tiler-ir/src/schedule/model.rs`) documents that on a loop-carried tile "participant `p` of round `r` owns the contiguous range at index `r * partitions + p`" over `partitions * contributors_per_partition * tile.rounds` contributors — a different index map from the flat one above. Both `MultiPass` and `CooperativeWorkgroup` carry `accumulation: ArithmeticType`, "the width every combining step is performed at", and the oracle has no such parameter. And any future non-uniform split is a third.

**Inference — none of the three is reachable today.** `workgroup_tree_tile` fixes `rounds: 1`, which the corpus already records as the reason the tree's and the split's declared groupings are identical at every contributor count; and every current plan accumulates at the element width. So the oracle is correct for every plan that exists and would be wrong for the first one that does not.

**Fact — filed `deferred` rather than `todo` because the board must not offer non-work.** There is nothing to evaluate until a topology outside the shape exists.

## Trigger

Either: a `ReductionTopology` in `crates/tiler-ir` realizes ADR 0100's multi-round composition with `rounds > 1` reachable from a constructed plan; or a plan declares an `accumulation` width other than its element type; or a non-uniform split is admitted.

## What this ticket must produce once fired

- An exact evaluator per newly admitted shape, written as that shape's definition rather than as an approximation of it, with the index map stated and matched against the topology's own documented map.
- A `RealizationNotEvaluable` refusal for a topology no evaluator covers, watched failing.
- A case at which the new shape and the existing blocked shape produce different bits, so the addition is evidence about something.

## Trigger check log

- 2026-08-05 — **not fired.** `grep -rn 'rounds' crates/tiler-ir/src/schedule/cooperative.rs` shows the field, and ADR 0100's `implementation_status` is `not-started`; no constructed plan carries `rounds > 1` and no plan declares an `accumulation` other than its element type. Reproduce with `grep -rn 'rounds: *[2-9]' crates/ --include='*.rs'` — an empty result is the not-fired verdict.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as refusal class 2's population.
