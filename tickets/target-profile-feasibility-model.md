---
id: target-profile-feasibility-model
title: Define target profiles and physical feasibility constraints
status: done
priority: p0
dependencies: []
related: []
scopes: [research/target-profiles, contracts/core, contracts/compiler, contracts/artifacts, contracts/integrations, research/numerics, research/runtime]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research, gpu]
---
Model device-specific physical dimensions without turning the tensor graph into a hypergraph. Define target capabilities and hard constraints for grids, threadgroups, SIMD groups or warps, memory spaces, barriers, binding limits, vector widths, registers, occupancy, and launch limits across Metal with CUDA and CPU/SIMD as comparison points.

Deliver a target-neutral schema proposal and examples showing which fields are correctness constraints, optimizer properties, or estimated costs. Identify capabilities that cannot be known until runtime.

## Outcome

- Research: [target profiles and phased feasibility](../docs/research/target-profiles/physical-feasibility-model.md)
- Adopted decision: [ADR 0043](../docs/decisions/0043-use-typed-phased-target-feasibility.md)
- Result: separated compile guarantees, phased typed capability facts, hard feasibility predicates, runtime guards, and non-authoritative costs.
