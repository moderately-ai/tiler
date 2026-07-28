---
id: implement-general-dag-partitioning
title: Implement general DAG partition search
status: todo
priority: p1
dependencies: [implement-boundary-property-enforcers, implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning, mature-product]
---
Extend partition planning to realistic DAGs with fan-out, named/multi-result outputs, legal shared-work duplication, materialization choices, and budgeted memoized search. Verify complete coverage and boundaries against exhaustive small-graph oracles and explain pruning.

## Dependency note (2026-07-28)

`implement-boundary-property-enforcers` is **`deferred`**, not in progress, so this ticket is not waiting on work someone is doing. Its deferral is a finding rather than a scheduling choice — the bounded profile admits no boundary mismatch for an enforcer to reconcile — and its restart condition is a **failing test** rather than a person: `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` (`crates/tiler-compiler/src/frontier.rs:2107`). The full derivation, the per-dimension table showing why no mismatch is currently expressible, and the list of changes that would fire the trigger are recorded at `tickets/implement-boundary-property-enforcers.md:23-50`; do not restate them here, and do not treat that ticket's `deferred` status as an invitation to start it.

The consequence for *this* ticket is specific: a general DAG partition search introduces exactly the variation that fires the trigger. Materialization choices and legal shared-work duplication both make one region's guarantee differ from another's requirement, which the single-region bounded profile cannot do. So this work is likely to be what unblocks the enforcers rather than something blocked behind them, and the dependency should be re-read — not merely re-checked — when this ticket is picked up.

## Closes when

1. Partition planning handles a DAG with fan-out: a value consumed by two or more regions is planned without duplicating it into incomparable partitions or silently serializing them.
2. Named and multi-result outputs are planned as ordered graph outputs, not reduced to a single root, and a plan naming fewer outputs than the program declares is rejected rather than accepted as a subset.
3. Legal shared-work duplication is a candidate the search can *choose*, with the legality condition stated and checked, and never a rewrite applied because it happened to be cheaper.
4. Materialization is a modelled choice per edge rather than a consequence of partition shape, and a deliberate materialization can win on cost.
5. The search is budgeted and memoized, and exhausting the budget yields an explainable partial result — the best plan found plus the statement that the space was not exhausted — never a silently truncated one presented as complete.
6. Coverage and boundaries are verified against exhaustive small-graph oracles: for every graph up to a stated size, the search's admitted set equals the oracle's, and each rejected candidate carries a feasibility reason rather than an absence.
7. Explain output names every pruned candidate and the reason it was pruned, distinguishing infeasible from dominated, and `make full` passes.
