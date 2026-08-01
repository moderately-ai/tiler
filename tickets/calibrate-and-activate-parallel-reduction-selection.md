---
id: calibrate-and-activate-parallel-reduction-selection
title: Calibrate and activate parallel reduction selection
status: todo
priority: p1
dependencies: [realize-parallel-reduction-strategies-on-metal]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/compiler, research/program-planning, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Target-aware selection chooses serial, single-workgroup, or multi-pass reduction from measured cost evidence and hard feasibility, so larger reductions stop serializing by default only where the qualified profile demonstrates that choice is faster.

## The measured pair, corrected

**Fact — the stated measurement target named a pair that could not both hold, and now can.** "The three retained alternatives on the exact qualified Metal environment" was unreachable: a split and a single-workgroup tree each consume ordered regrouping, the measured Apple `f32` row flushes subnormals in every math mode, and none of the four registered contracts both flushed and permitted regrouping — so on that environment only the serial fold was ever retained and there were no three alternatives to measure. `compose-the-numerical-contract-from-its-decided-dimensions` closed that: the contract is composed from its dimensions, and `NumericalContract::FLUSH_AND_REASSOCIATE_F32` is the named point that resolves both.

**The measurement target is therefore the pair, stated together:** the three retained alternatives under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, against `BoundMetalCompileDeclaration::first_macos_apple9`, on the qualified Metal host. Any other contract on that declaration retains fewer than three alternatives and cannot supply a crossover; `crates/tiler-build/src/metal_plan.rs`'s `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is what pins that the pair reaches a portfolio at all, and it also records the shape constraint the fixture ran into — this profile's grid axis admits four threads, so the widest stage of the measured program has to fit it.

## Implementation keys

Measure the three retained alternatives over a predeclared shape/workgroup matrix on the exact qualified Metal environment, under the contract named above. Fit or select the smallest analytical calibration that predicts the measured crossover without folding infeasibility into cost. Preserve all close alternatives in the portfolio and explain the measured assumptions and winning terms.

Do not encode an arbitrary preference for parallel plans. Current structural dominance favors fewer dispatches and analytical costs do not participate in dominance; activation must deliberately connect reviewed cost evidence to selection rather than altering a constant until the desired plan wins.

## Required evidence

Retained raw measurements identify stable crossover regions or explicitly report that none was established. Calibration predicts held-out rows within a stated error bound, serial remains selected below its measured crossover, and an unavailable environment makes no performance claim. Perturbing the calibrated term or environment identity changes or refuses the selection evidence.

## Closes when

Selection uses measured target-specific evidence, explain output names why the winning strategy won, no infeasible plan is represented as expensive, every check is mutation-proved, and the performance record plus targeted gates pass.

## Graph maintenance

- Keep this ticket after Metal realization so calibration measures executable strategies rather than synthetic cost constants.
- Close `implement-parallel-reduction-strategies` only after this ticket connects retained measurements to selection and the three-strategy rollup is true on one merged tree.
- File a bounded environment-specific measurement follow-up instead of asserting a crossover when the qualified host or stable region is unavailable.
