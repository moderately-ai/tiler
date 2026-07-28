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
| `Dispatch` | nothing — this one is *nearly* free, since the structural estimate already counts dispatches; it was left out only because reporting it under a second key duplicates a number already reported under the first, and whether that is useful or confusing is worth deciding rather than assuming |
| `RedundantWork` | a model of what fusion recomputes, which needs the access relations rather than the cover shape |
| `Indexing` | per-element address arithmetic, derivable from the index region but not currently summarized anywhere |
| `Synchronization` | the barrier structure, which lives in the kernel program rather than the plan |
| `ResourcePressure` | a register and threadgroup-memory model per target profile; none exists |
| `CompileTime` | measurement, not analysis — it is the one component whose honest form may be `Bounded` from observed compiles rather than derived |
| `ArtifactSize` | encoded bytes, which only exist after encoding; the plan precedes it |

**The rule that made the framework honest and must survive here:** a component whose input does not exist stays `Unknown`. `Unknown` is a measurement boundary and is deliberately not zero — `component_cost::tests::unknown_is_not_a_zero` pins that, because a caller treating it as zero would report a plan as free. Do not close this ticket by filling eight slots with plausible arithmetic; close it by modelling the ones whose inputs arrive, and leaving the rest stated.

## Constraints inherited from the parent

- Nothing here may enter dominance. A `ComponentCost` is not a `PhysicalCostEstimate`, and `frontier::tests::an_analytical_cost_key_is_refused_by_the_frontier` guards the remaining route. Admitting a second model key into Pareto comparison makes plans mutually incomparable and turns pruning off silently.
- Units are fixed per component by `CostComponent::unit`, so a value and a unit cannot disagree. A new component's unit is an exhaustive-match arm, not a field.
- `CostValue::Bounded` exists and is asserted well formed but is not yet constructed. It is the shape the first genuinely modelled component should carry; a point estimate for something that is not exact should be a refuted-if-wrong range, not a number.

## Closes when

- Each of the eight is either computed (`Exact` or `Bounded`) or carries a stated reason it remains `Unknown`, recorded at the match arm rather than in a document.
- The explain census in `pipeline/tests.rs` is updated in the same change, since its count of `tiler.cost.analytical.v1` records grows as components become modelled. That test is what catches an unreported component.
- The retained plan set and the selected plan are unchanged, asserted as they were for the parent.
- No component consulted for feasibility.
