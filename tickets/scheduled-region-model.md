---
id: scheduled-region-model
title: Validate a first-class scheduled-region model
status: todo
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract, target-profile-feasibility-model]
related: []
scopes: [research/scheduling]
shared_scopes: []
paths: []
tags: [tiler-research, spike, scheduler, gpu]
---
Take several legal tensor regions and represent alternative mappings to grids, threadgroups, SIMD groups or warps, vector lanes, staging, reductions, tails, and launch geometry. Determine the normalized ScheduledRegion fields and verifier responsibilities needed before structured kernel lowering.

Deliver worked schedules for elementwise, broadcast-plus-elementwise, transpose-like access, and a reduction. Show why schedule choices are not node properties and how rejected alternatives are explained.
