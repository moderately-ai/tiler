---
id: cost-model-bootstrap
title: Design the initial cost model and calibration experiments
status: done
priority: p1
dependencies: [region-search-oracle, scheduled-region-model, target-profile-feasibility-model]
related: []
scopes: [research/cost-model]
shared_scopes: []
paths: []
tags: [tiler-research, research, measurement, optimizer]
---
Define an initial separable cost model for traffic, allocation, dispatch, redundant computation, index arithmetic, synchronization, occupancy pressure, compilation time, and artifact size. Preserve hard feasibility as a prior gate rather than a large penalty.

Deliver calibration hypotheses, controlled microbenchmarks, required target-profile inputs, uncertainty reporting, and comparisons against the exhaustive tiny-DAG oracle. State what the first model deliberately cannot predict.

## Outcome

- Research: [bootstrap cost model](../docs/research/cost-model/bootstrap-cost-model.md)
- Experiment: [cost-model experiment](../spikes/cost-model/README.md)
- Contract: [cost model](../docs/compiler/cost-model.md)
- Result: defined a transparent interval-valued baseline after hard feasibility, with calibration records, uncertainty widening, and explicit unsupported interactions.
