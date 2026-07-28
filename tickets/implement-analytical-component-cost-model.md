---
id: implement-analytical-component-cost-model
title: Implement an analytical component cost model
status: done
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

**What cost decides today, exactly one thing.** `PlanStructuralCost` "is never a feasibility input and never gates validity; it only prunes a dominated plan from the `SelectedPortfolio::non_dominated` view" (`selection.rs:127-129`). `select_non_dominated` (`pipeline/planning.rs:598`) then takes the *first* retained plan in canonical order — it does not minimize. So the cost model's entire observable effect is which plans survive Pareto pruning, and a change to it is only meaningful if it changes that set.

**The constraint that forces the slice.** Dominance returns `false` between estimates carrying different `model_key`s — "estimates from different cost models are incomparable" (`selection.rs:175-178`) — and the frontier *rejects* any proposal whose cost is not attributed to `tiler.cost.structural.v1` with `FrontierError::MalformedCostProvenance` (`frontier.rs:1374`). `aggregate_cost` separately refuses a plan whose regions mix keys (`selection.rs:1512-1515`). Three independent places enforce one model. Line numbers re-verified at `01264be`; the earlier citations (`frontier.rs:1181`, `selection.rs:1518`, `planning.rs:565`) named the right constructs at the commit they were written and have drifted, so the constructs are named beside them.

That eliminates both obvious readings:

- **Replace the structural model.** All plans stay comparable, but pruning would then rest entirely on numbers this ticket's own text calls "explicitly analytical and uncalibrated", with `calibrate-device-cost-models` owning activation. Discarding a genuinely better plan on an uncalibrated estimate is a regression, not a trade-off.
- **Add it alongside as a second key.** The frontier gate would have to be widened, and once it is, plans carrying different keys never dominate each other — the non-dominated set becomes the whole set and selection silently degenerates to first-in-canonical-order. Pruning would go dark with nothing reporting it. This is the cheaper-looking option and it is the one that quietly removes the only decision cost makes.

**What survives, and is therefore the slice.** Compute the analytical component costs and record them in typed explain, while structural cost continues to be the sole pruning input. Nothing about the retained set moves, so the change is verifiable by the portfolio being byte-identical before and after; the analytical numbers become the subject `calibrate-device-cost-models` calibrates and activates. The `model_key` gate stays exactly as it is — the analytical estimate is not a `PhysicalCostEstimate` and does not enter dominance.


## Landed — the framework, with one component modelled (2026-07-27)

`crates/tiler-compiler/src/component_cost.rs`, reported through `pipeline/trace.rs::record_analytical_costs`.

- Nine governed components, closed vocabulary, canonical order matching the derived ordering, each with a unit fixed by an exhaustive match on the component so a value and a unit cannot disagree.
- `CostValue` keeps three evidence classes apart — `Exact`, `Bounded { low, high }`, `Unknown` — rather than one confidence scalar, because an unmodelled component and an imprecisely modelled one are not the same claim and only the second is safe to calibrate against.
- `Allocation` is computed exactly, as the sum of the temporary bytes each region's implementation already states. The other eight are `Unknown`.
- Every retained plan reports its modelled components plus a count of its unmodelled ones, under `tiler.cost.analytical.v1`.
- **Nothing entered dominance.** The structural model remains the sole pruning input. `frontier::tests::an_analytical_cost_key_is_refused_by_the_frontier` confirms an estimate claiming the analytical key is refused by name.
- The explain census test caught the four new records before this was committed, which is the check working; it now names them.

**Why eight are `Unknown` rather than estimated.** Each needs an input that does not exist — per-region element traffic, an occupancy model, a resource-pressure model, artifact sizes that only exist after encoding. A formula invented to fill one would be unfalsifiable exactly where it mattered, and `Unknown` is deliberately not zero (`component_cost::tests::unknown_is_not_a_zero`), because a caller substituting zero would report a plan as free. Modelling the eight is `model-the-eight-unmodelled-cost-components`, which carries the per-component reason each was left out.

## Closes when

- ~~All nine components computed~~ — split. The framework, the governed key, the units, the three evidence classes, the explain reporting, and one exactly-computed component have landed; the remaining eight are a live dependent ticket.
- Component costs never enter dominance — **met and asserted** by `frontier::tests::an_analytical_cost_key_is_refused_by_the_frontier` (`crates/tiler-compiler/src/frontier.rs:2425`). The retained plan set was unchanged across the landing commit, verified by comparison at the time; nothing asserts it on an ongoing basis, because "unchanged" is a claim about a diff and no golden over the plan set exists. The nearest standing checks are a `plan-count == 2` census row (`pipeline/tests.rs:607`) and `stable_id == plan.identity().label()` (`pipeline/tests.rs:535`), and neither would notice a different pair of plans being retained.
