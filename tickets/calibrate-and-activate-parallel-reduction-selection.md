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

## Implementation keys

Measure the three retained alternatives over a predeclared shape/workgroup matrix on the exact qualified Metal environment. Fit or select the smallest analytical calibration that predicts the measured crossover without folding infeasibility into cost. Preserve all close alternatives in the portfolio and explain the measured assumptions and winning terms.

Do not encode an arbitrary preference for parallel plans. Current structural dominance favors fewer dispatches and analytical costs do not participate in dominance; activation must deliberately connect reviewed cost evidence to selection rather than altering a constant until the desired plan wins.

## Required evidence

Retained raw measurements identify stable crossover regions or explicitly report that none was established. Calibration predicts held-out rows within a stated error bound, serial remains selected below its measured crossover, and an unavailable environment makes no performance claim. Perturbing the calibrated term or environment identity changes or refuses the selection evidence.

## Closes when

Selection uses measured target-specific evidence, explain output names why the winning strategy won, no infeasible plan is represented as expensive, every check is mutation-proved, and the performance record plus targeted gates pass.

## Graph maintenance

- Keep this ticket after Metal realization so calibration measures executable strategies rather than synthetic cost constants.
- Close `implement-parallel-reduction-strategies` only after this ticket connects retained measurements to selection and the three-strategy rollup is true on one merged tree.
- File a bounded environment-specific measurement follow-up instead of asserting a crossover when the qualified host or stable region is unavailable.
