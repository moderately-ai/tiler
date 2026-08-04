---
id: implement-general-dag-partitioning
title: Implement general DAG partition search
status: review
priority: p1
dependencies: [implement-analytical-component-cost-model]
related: [implement-boundary-property-enforcers]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning, mature-product]
claimed_from: todo
assignee: agent-dag-partition
lease_expires_at: 1785876199
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

## Outcome, 2026-08-04 — implemented; the search semantics go to review

All seven `Closes when` conditions are implemented in `crates/tiler-compiler`, crate-private throughout: no public type, trait, or facade item changed, so no public boundary is proposed by this ticket. The search semantics are consequential, so the ticket ends at `review` rather than `done`.

**Where the work landed.** `cover.rs` became the general DAG partition search: it carries a stated legality contract (`CoverPolicy`), a per-member duplication legality condition with typed refusals (`DuplicationRefusal`), an anchored search generalized to admit legal overlap and extended by an augmentation phase, a memoized completion table, a partition-level cost model (`CoverCost`, `tiler.cost.partition-structural.v1`) with a pure dominance view, and a bounded typed refusal channel. `pipeline/trace.rs` reports both pruning channels. `frontier.rs` resolves the `Intermediate` work scaling from the cover edge. `request.rs` gained the contract predicate the recomputation condition is decided against.

**Condition-by-condition.**

1. *Fan-out.* `fan_out_is_materialized_once_and_read_by_every_consumer` — one edge, both consumers on it, distinct regions, no duplication under the exact-partition contract.
2. *Ordered multi-result outputs.* `multi_result_outputs_are_retained_and_a_dropped_one_is_rejected` — the two-output fixture is planned with both outputs retained by every cover, none reduced to a root, and a cover naming fewer is refused. `verify_cover` now checks each ordered named output is produced by **exactly one** region rather than merely retained by some region. The *exactly-one* half is defence in depth and is stated as such at its site: a second producer of a named output would need a second region containing that output's producer, and the duplication condition refuses a named-result producer first. It is driven directly by `the_named_output_and_observability_checks_can_say_no`, because a check nothing has shown can fail is one a reader may not rely on.
3. *Duplication as a chooseable candidate.* `shared_work_duplication_is_a_candidate_the_search_chooses` and `duplication_refusals_state_which_condition_refused_them`. The condition is purity, no named result, and a contract granting no realization freedom — each stated at `duplication_refusal` and each refutable.
4. *Materialization as a per-edge choice that can win.* `a_deliberate_materialization_dominates_a_partial_recomputation`. A partial duplication pays the same edge *and* the recomputation, so the materializing cover strictly dominates it; absorbing every consumer instead is an incomparable trade and both stay retained.
5. *Budgeted and memoized, partial results explainable.* `cover_budget_stops_report_bounded_loss_and_keep_the_required_covers` plus `CoverEnumeration::is_exhaustive`, which the explain trace states on every compile rather than only when a budget fires.
6. *Exhaustive small-graph oracles.* `enumeration_matches_the_exhaustive_partition_oracle` and `duplicating_enumeration_matches_the_exhaustive_cover_oracle`, over programs of four and five operations with up to seventeen candidates (2^17 subsets). Every rejected candidate carries a typed reason rather than an absence, and `the_oracle_comparison_rejects_a_perturbed_admitted_set` proves the comparison can say no.
7. *Explain names every pruned candidate.* `record_cover_refusals` emits a disproved check per refusal and `record_dominated_covers` emits a cost assessment per beaten cover, naming both it and the cover that beat it — a rejection disposition and a cost disposition, so infeasible and dominated are never read as each other.

**The oracle found a real defect, recorded because it generalizes.** The anchored search admits a candidate only when it covers the branch's minimum uncovered operation, so a region every one of whose operations is already covered can never be chosen — and such a region is not idle: `{shared}` beside `{constant, shared, left}` is one of the two ways to spell a partial duplication. The exhaustive oracle named the missing covers; `Partitioner::emit` now enumerates the anchored base plus every augmentation by such regions, which is complete because running the anchor rule over any legal cover selects a base and leaves exactly that remainder.

## Graph maintenance, executed

- **`verify_cover`'s exactly-once check is relaxed behind `CoverDuplication`, and `component_cost`'s `RedundantWork` note is corrected rather than re-trusted.** The note said the trigger was that contract change. It has landed, and the value still does not move — so the premise was necessary but not sufficient. The fold reads the *selected implementations'* claimed members, so a duplicating cover must reach a `SelectedPlan`, which needs a physical implementation per duplicated subject; the bounded provider proposes for three member sets and program assembly implements three plan shapes. The arithmetic is therefore asserted directly instead: `repeated_work` is extracted and unit-tested against two stages claiming one member set, and it moves off zero there (`repeated_work_moves_when_two_stages_claim_one_member_set`). The refined trigger is recorded at the arm and filed as [`activate-shared-work-duplication-on-the-compile-path`](activate-shared-work-duplication-on-the-compile-path.md).
- **The enforcers trigger did not fire, checked rather than assumed.** `frontier.rs::the_bounded_profile_admits_no_undischarged_boundary` still passes, and no property set was widened. The reason is the same one above: duplication is admitted by the *search*, and the compile path enumerates under `CoverPolicy::governed`, so no plan carries two regions whose boundary contracts differ. This ticket therefore does **not** supply the enforcer's first case; the 2026-08-02 re-read's expectation that it would is corrected here, and [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md) keeps the restart condition its own 2026-07-28 restatement gives it.
- **The `Intermediate` work-scaling decline is resolved.** `MaterializationEdge` now carries the materialized value's element count, and `FrontierRegionSubject::reading_intermediates` states the counts of the edges a cover hands each region, so `resolve_work_items` resolves `PerElementOf` bound to an intermediate from the edge that exists. It still declines in the two cases where nothing derived an answer — a subject stated outside any cover, and a subject reading intermediates of different sizes — and both are asserted. The falsified-premise history at the site is preserved and extended rather than replaced.
