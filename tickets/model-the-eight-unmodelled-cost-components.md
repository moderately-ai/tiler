---
id: model-the-eight-unmodelled-cost-components
title: Model the eight unmodelled analytical cost components
status: todo
priority: p2
dependencies: [implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, performance]
---
The analytical cost framework landed with all nine governed components enumerated and one of them, `Allocation`, computed exactly. The other eight report `CostValue::Unknown`. This ticket models them.

## Why they were left Unknown rather than estimated

Each needs an input the compiler does not have. `Allocation` was computable because every region's implementation already states its temporary bytes and the plan's total is their sum — exact, not modelled. The rest are not close to that:

| Component | What it needs that does not exist |
| --- | --- |
| `MemoryTraffic` | per-region element traffic; `launched_threads` is a proxy for elements, not bytes moved, and reading it as bytes would be wrong by the element width and by every reuse |
| ~~`Dispatch`~~ | **Modelled 2026-07-27.** The duplication question resolved in favour of reporting it: calibration compares a device measurement against this model component by component, so a component absent here cannot be correlated with anything measured, and dispatch overhead is among the first things a device measurement sees. The structural count exists to be *pruned* on and this one to be *calibrated* against; the two uses share no consumer. A `debug_assert_eq!` in `record_analytical_costs` pins the two counts to agree, since a duplicated number that drifted would be worse than none — a calibration pass would attribute the difference to the device. Its failure path was verified by perturbing the analytical sum by one and watching it fire. |
| `RedundantWork` | a model of what fusion recomputes, which needs the access relations rather than the cover shape |
| ~~`Indexing`~~ | **Modelled 2026-07-27.** The premise was wrong twice over: `IndexRegion` states both `accesses` and `iteration_shape` directly, and `index_expressions()` is an `ExactSizeIterator`, so the summary I claimed did not exist is O(1). Computed as one address per logical access per iteration point, summed over regions. This is the first component that can legitimately fail to have a value — an iteration shape whose element count overflows `u64` has no stateable total, and the component reports `Unknown` rather than a saturated number, because a saturated total is something a calibration pass could compare against and silently disagree with. |
| ~~`Synchronization`~~ | **Modelled 2026-07-27.** The premise was wrong: it does not need the kernel program's barrier structure. Every satisfied cross-region handoff is a producer/consumer edge whose consumer requires `AfterProducingDispatch`, discharged by the producer's `AfterOwnDispatch`, so each (producer, consumer) pair is exactly one ordering constraint and the plan already carries them. Counted per consumer rather than per handoff, since a handoff with three consumers imposes three waits. Stated at the match arm so it can be refuted: if a target ever orders a whole handoff with one barrier, this becomes an upper bound and must be restated as `Bounded` rather than quietly redefined. A `debug_assert!` pins it at or above the materialization count — equality holds only in the single-consumer case — and its failure path was verified by zeroing the sum and watching it fire, which also proved the count is genuinely non-zero rather than vacuously satisfied. |
| `ResourcePressure` | a register and threadgroup-memory model per target profile; none exists |
| `CompileTime` | measurement, not analysis — it is the one component whose honest form may be `Bounded` from observed compiles rather than derived |
| `ArtifactSize` | encoded bytes, which only exist after encoding; the plan precedes it |

**The rule that made the framework honest and must survive here:** a component whose input does not exist stays `Unknown`. `Unknown` is a measurement boundary and is deliberately not zero — `component_cost::tests::unknown_is_not_a_zero` pins that, because a caller treating it as zero would report a plan as free. Do not close this ticket by filling eight slots with plausible arithmetic; close it by modelling the ones whose inputs arrive, and leaving the rest stated.

## Constraints inherited from the parent

- Nothing here may enter dominance. A `ComponentCost` is not a `PhysicalCostEstimate`, and `frontier::tests::an_analytical_cost_key_is_refused_by_the_frontier` guards the remaining route. Admitting a second model key into Pareto comparison makes plans mutually incomparable and turns pruning off silently.
- Units are fixed per component by `CostComponent::unit`, so a value and a unit cannot disagree. A new component's unit is an exhaustive-match arm, not a field.
- `CostValue::Bounded` exists and is asserted well formed but is not yet constructed. It is the shape the first genuinely modelled component should carry; a point estimate for something that is not exact should be a refuted-if-wrong range, not a number.

## Progress

- `Dispatch`, `Synchronization`, and `Indexing` modelled (see the struck rows above). Five remain `Unknown`.
- **Two of the three came off the list because the source contradicted this table, not because anything changed.** `Synchronization` was recorded as needing the kernel program's barrier structure; it needed the plan's handoff edges, which it already had. `Indexing` was recorded as not summarized anywhere; `IndexRegion` states it directly. Both notes were written from the same reading, in one sitting, with the same confidence as the five that remain. **Treat those five as unverified**: re-derive each against the source before accepting that it cannot be modelled. That is the cheapest work left on this ticket and it has now paid twice.
- What the checks do and do not cover: the explain census pins *reachability* — a component silently reverting to `Unknown` drops the record count and fails, verified by forcing `Indexing` to `Unknown` and watching the count fall from 10 to 8. It does **not** pin the *values*. `Synchronization` has a value cross-check (at or above the materialization count); `Allocation`, `Dispatch`, and `Indexing` do not, and a wrong-but-non-zero value in those three would pass today.

## Closes when

- Each of the eight is either computed (`Exact` or `Bounded`) or carries a stated reason it remains `Unknown`, recorded at the match arm rather than in a document.
- The explain census in `pipeline/tests.rs` is updated in the same change, since its count of `tiler.cost.analytical.v1` records grows as components become modelled. That test is what catches an unreported component.
- The retained plan set and the selected plan are unchanged, asserted as they were for the parent.
- No component consulted for feasibility.
