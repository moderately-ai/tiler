---
id: implement-general-dag-partitioning
title: Implement general DAG partition search
status: todo
priority: p1
dependencies: [implement-analytical-component-cost-model]
related: [implement-boundary-property-enforcers]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning, mature-product]
---
## User-visible outcome

The planner can partition a real DAG — fan-out, multi-result outputs, deliberate duplication, materialization as a modelled per-edge choice — instead of the current single-chain covers, verified against exhaustive small-graph oracles.

Extend partition planning to realistic DAGs with fan-out, named/multi-result outputs, legal shared-work duplication, materialization choices, and budgeted memoized search. Verify complete coverage and boundaries against exhaustive small-graph oracles and explain pruning.

## The re-read this ticket asked for, run 2026-08-02 — the edge was backwards and is removed

The note below asks that the enforcer dependency "be re-read — not merely re-checked — when this ticket is picked up", and then makes that impossible: a `todo` p1 behind a `deferred` dependency never reaches `ready`, so it is never picked up. [`re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed`](re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed.md) exists for exactly that deadlock, and this is its outcome. Both tickets were read in full against each other, and the two facts it named as unsettled were checked rather than assumed.

**Fact — nothing this ticket must deliver consumes an enforcer.** All seven `Closes when` conditions are about fan-out, ordered multi-outputs, duplication legality, materialization as a per-edge choice, budget and memoization, oracle agreement, and explain output. None requires a boundary mismatch to be reconciled; none names an enforcer.

**Fact — this ticket's own Graph maintenance says it *produces* the enforcer's input.** It records that materialization choices "likely fire the enforcers trigger", that "the failing constant test is the signal", and that "its mismatch is the enforcer's first case". A ticket that supplies another ticket's first real case is upstream of it, not blocked behind it. The dependency is therefore **removed** and the relation kept as `related`. It is not deleted to unblock a frontier — it is inverted because both tickets' own texts say so, and the inversion is recorded here so it can be refuted.

**Fact — the other dependency is satisfied.** `implement-analytical-component-cost-model` is `done`, so with the enforcer edge removed this ticket's dependencies are met and it can reach `ready`.

**Fact — the enforcer's restart condition now has a graph edge, and it is not this ticket.** [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md)'s own 2026-07-28 restatement supersedes the constant-test trigger the note below cites: the startable condition is "a compile-path provider proposes an opaque call whose contract the composing consumer refuses", which "arrives with caller-supplied physical providers". That is [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md), which is now a dependency of the enforcer ticket rather than a sentence in it.

**Fact — `PhysicalAuthorities::composed` still has no production caller, checked rather than assumed.** `grep -rn 'PhysicalAuthorities' crates/` returns the composed constructor only inside `crates/tiler-compiler/src/pipeline/tests.rs`; the sole production construction is `PhysicalAuthorities::governed()` at `crates/tiler-compiler/src/pipeline.rs:591`. So the refused handoff is still unreachable on the production path, exactly as the enforcer ticket says.

**Finding, recorded rather than resolved — the provider route may not be sufficient on its own, and whoever picks up the enforcer ticket must settle it.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125 states that "out-of-crate opaque-call registration stays compiler-owned and crate-private per ADR 0078's correction" and that "no caller of any kind registers one on the compile path". `register-opaque-calls-on-the-compile-path`, which ADR 0090 names as owning that internal wiring gap, is `done` — yet `composed` still has no production caller. So whether a *caller-supplied* provider can produce the refused handoff at all, given that registration stays crate-private, is an open question the enforcer's restart condition assumes an answer to. This is flagged, not decided: deciding it needs the provider work in front of it.

## Dependency note (2026-07-28) — superseded above, retained for its derivation

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

## Graph maintenance

- **Legal shared-work duplication requires relaxing `verify_cover`'s exactly-once check** (`CoverError::IllegalDuplication`) behind the reserved `CoverDuplication` policy — and the moment you do, `component_cost`'s `RedundantWork` arm becomes genuinely exercisable: its comment names this exact contract change as the trigger and asks you to check the value moves. Add that assertion when you relax the contract.
- **Materialization choices likely fire the enforcers trigger** — same protocol as `implement-parallel-reduction-strategies`: the failing constant test is the signal, its mismatch is the enforcer's first case, and the property sets are not to be widened.
- **The `Intermediate` work-scaling decline** in `resolve_work_items` (frontier.rs) declines because an intermediate's element count is a property of the cover — your cover edges will finally carry that shape, so resolve it here (the falsified-premise history is recorded at the arm; read it before re-resolving).
