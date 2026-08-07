---
id: accept-the-measured-cost-row-public-surface
title: Accept the measured cost row public surface
status: awaiting-decision
priority: p1
dependencies: []
related: [activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, target-profiles]
---
## What is being accepted

Tom accepted the **model** on 2026-08-07 — a measured cost row admitted as a distinct kind from a capability axis, with silence meaning *no preference* rather than *no plan*. That acceptance explicitly did **not** settle the spelling: the `declare_*` / `declare_measured_*` pair is a public boundary under ADR 0075 and comes back with the built surface. [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md) built it; this node carries it. **Only Tom closes this.**

## The exact surface

New in `tiler_compiler::target`:

- `TargetCostRowResolution` — the resolution a cost row takes, including the `Unknown` a silent profile resolves to.
- `TargetProfileBuilder::declare_saturated_parallel_fold_steps` and `::declare_measured_saturated_parallel_fold_steps`, the pair whose measured constructor carries a `TargetCompileProfileMeasurementSource` so its validity stays `MeasuredEnvironment` and cannot widen into a portable claim.
- `TargetProfile::saturated_parallel_fold_steps` — the reader.
- `TargetProfileBuildError::DuplicateCostRow`.

Labelled a draft in `target.rs`'s header, in the shape the accepted evaluation-order family uses, stating that the acceptance covers the model and not the spelling.

## The choice worth objecting to, and it is not the spelling

**The measured term ranges over the retained valid plans and can prefer a structurally dominated plan.** That is a change to what selection *is*, and it was reached by measurement rather than by preference.

The cheaper shape — a term that breaks ties *inside* the non-dominated set — **cannot express the retained measurement at all.** The serial fold issues no more dispatches, launches strictly fewer threads, and allocates no more temporary storage than either parallel strategy, so it **structurally dominates both**; `the_parallel_reduction_plans_are_structurally_dominated` asserts the portfolio holds all three strategies while `non_dominated()` holds exactly one. The same fold was measured costing up to **50.7x** the best parallel plan. A tie-break inside a singleton decides nothing.

So the term ranges over every plan hard feasibility and boundary composition admitted. **It can never prefer an infeasible plan, because no infeasible plan is in the set** — feasibility was decided by the frontier and is not consulted, weakened, or re-run.

If you would rather selection stayed a pure structural relation and the measured preference lived somewhere else entirely, this is the thing to say so about. The consequence of that would be that the measurement cannot be acted on at all on this program family.

## Both reserved constraints, answered structurally rather than argued

**No second cost-model key is ever minted.** `PlanStructuralCost`, `dominates`, `aggregate_cost`, the frontier's single-key check and `non_dominated` are untouched; `measured_cost` has no `dominates` and carries no key into `aggregate_cost`. So the failure mode `component_cost.rs` warns about — plans with differing keys never dominating, the non-dominated set silently becoming the whole set, and Pareto pruning going dark with nothing reporting it — cannot arise, because nothing new enters the relation.

**`PlanStructuralCost` was not widened with a latency dimension.** Its four dimensions stay exact counts.

**It is not a latency estimate.** The retained measurement states the fitted model is a selector whose magnitude accuracy is much weaker than its decision accuracy, so the module reports fold steps and only ever compares them; it never reports seconds.

## Evidence

Mutation-proved at the sweep's own perturbation scale: at 1,024 rows x 64 contributors the fitted 1,056 and a quartering select the fold, and **quadrupling selects a parallel plan** — moving both selection points, since `select_global_non_dominated` would otherwise re-prune the winner. The unchanged-golden proof holds at a shape the row *does* move: a silent profile's descriptor is byte-identical, the row resolves `Unknown`, and the structural winner is still selected. Explain names the term and both sides of the `max` verbatim. The reduced selector was scored against the sweep's own triples, reproducing `perturbations.txt`'s held-out worst penalties of 1.81x / 3.04x / 1.20x — measured because dropping `step` is provably order-preserving while dropping `encoder` is not.

## Identity

Four pins moved and were **recomputed on the merged tree** by the coordinator rather than taken from the branch: descriptor length 1,999 → **2,099**, artifact identity `23c46a19…` → **`357f0676…`**, cache subject `e89c4d82…` → **`c626e43b…`**, fixed content 64,542 → **65,242**. The delta is encoding-predicted to the byte: 100 bytes of section, embedded seven times, plus the section itself.

## A measurement boundary that must not be lost

The sweep dispatched the tree at the **balanced** split; `MEASURED_TREE_PARTICIPANT_CAP` landed after it. That moves which *parallel* plan is preferred, not whether the program parallelizes, so the contour this row turns on is unaffected — but it is a real bound on the evidence and is recorded in the ledger, the spike README, and the test rather than only here.

## Closes when

Tom accepts the spelling, accepts with a named exclusion, or rejects it. The behaviour is landed and gated meanwhile; what is parked is the surface.
