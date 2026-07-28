---
id: implement-analytical-component-cost-model
title: Implement an analytical component cost model
status: todo
priority: p1
dependencies: [implement-boundary-property-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, performance]
---
Implement deterministic symbolic component costs for memory traffic,
allocation, dispatch, redundant work, indexing, synchronization, resource
pressure/occupancy, compile time, and artifact size. Preserve units,
assumptions, uncertainty, target-profile subjects, and typed explain; hard
feasibility remains separate. This is explicitly analytical and uncalibrated.
`calibrate-device-cost-models` owns later device measurements and activation.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.


## Scoped — compute and explain, do not prune (2026-07-27)

The ticket had no closing criteria. Reading the cost path settles what the first slice must be, and eliminates two approaches that look like alternatives.

**What cost decides today, exactly one thing.** `PlanStructuralCost` "is never a feasibility input and never gates validity; it only prunes a dominated plan from the `SelectedPortfolio::non_dominated` view" (`selection.rs:127`). `select_non_dominated` (`pipeline/planning.rs:565`) then takes the *first* retained plan in canonical order — it does not minimize. So the cost model's entire observable effect is which plans survive Pareto pruning, and a change to it is only meaningful if it changes that set.

**The constraint that forces the slice.** Dominance returns `false` between estimates carrying different `model_key`s — "estimates from different cost models are incomparable" (`selection.rs:176`) — and the frontier *rejects* any proposal whose cost is not attributed to `tiler.cost.structural.v1` with `FrontierError::MalformedCostProvenance` (`frontier.rs:1181`). `aggregate_cost` separately refuses a plan whose regions mix keys (`selection.rs:1518`). Three independent places enforce one model.

That eliminates both obvious readings:

- **Replace the structural model.** All plans stay comparable, but pruning would then rest entirely on numbers this ticket's own text calls "explicitly analytical and uncalibrated", with `calibrate-device-cost-models` owning activation. Discarding a genuinely better plan on an uncalibrated estimate is a regression, not a trade-off.
- **Add it alongside as a second key.** The frontier gate would have to be widened, and once it is, plans carrying different keys never dominate each other — the non-dominated set becomes the whole set and selection silently degenerates to first-in-canonical-order. Pruning would go dark with nothing reporting it. This is the cheaper-looking option and it is the one that quietly removes the only decision cost makes.

**What survives, and is therefore the slice.** Compute the analytical component costs and record them in typed explain, while structural cost continues to be the sole pruning input. Nothing about the retained set moves, so the change is verifiable by the portfolio being byte-identical before and after; the analytical numbers become the subject `calibrate-device-cost-models` calibrates and activates. The `model_key` gate stays exactly as it is — the analytical estimate is not a `PhysicalCostEstimate` and does not enter dominance.

## Closes when

- The nine component costs the body names — memory traffic, allocation, dispatch, redundant work, indexing, synchronization, resource pressure/occupancy, compile time, artifact size — are computed per plan as deterministic symbolic values carrying units, assumptions, and uncertainty.
- Each is attributed to its own governed analytical model key, distinct from `tiler.cost.structural.v1`, and none is admitted as a `PhysicalCostEstimate`.
- They appear in typed explain output for every retained plan.
- The selected plan and the non-dominated set are unchanged: an existing end-to-end compile test asserts the portfolio identity is byte-identical to its pre-change value.
- A test confirms an analytical estimate cannot reach dominance — that widening it into `PhysicalCostEstimate` is a build error or an explicit rejection, not a silent pass.
- Hard feasibility remains untouched; no component cost is consulted for validity.

**Note on the last two.** They are what stop the slice from becoming the eliminated second option by accident. A component cost that leaks into dominance turns pruning off, and nothing else in the compiler would report that it had.
