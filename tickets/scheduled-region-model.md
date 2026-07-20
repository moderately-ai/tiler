---
id: scheduled-region-model
title: Validate a first-class scheduled-region model
status: done
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract, target-profile-feasibility-model]
related: []
scopes: [research/scheduling]
shared_scopes: []
paths: [docs/decisions/0007-first-class-kernel-schedules.md, docs/decisions/README.md]
tags: [tiler-research, spike, scheduler, gpu]
---
Take several legal tensor regions and represent alternative mappings to grids, threadgroups, SIMD groups or warps, vector lanes, staging, reductions, tails, and launch geometry. Determine the normalized ScheduledRegion fields and verifier responsibilities needed before structured kernel lowering.

Deliver worked schedules for elementwise, broadcast-plus-elementwise, transpose-like access, and a reduction. Show why schedule choices are not node properties and how rejected alternatives are explained.

## Outcome

- Research: [scheduled-region model](../docs/research/scheduling/scheduled-region-model.md)
- Experiment: [scheduled-region experiment](../spikes/scheduling/README.md)
- Adopted decision: [ADR 0007](../docs/decisions/0007-first-class-kernel-schedules.md)
- Result: established normalized schedule identity, intrinsic verification, separate target feasibility, and a distinct non-authoritative transform trace.
