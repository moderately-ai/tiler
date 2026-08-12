---
id: admit-scalar-epilogue-fixed-vector-map-tails
title: Admit scalar-epilogue fixed-vector map tails
status: todo
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir, declare-cpu-vector-realization-facts-in-the-target-profile, establish-vector-execution-form-numerical-authority]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/cpu, contracts/decisions, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, scheduling, numerics, public-boundary]
---
## User-visible outcome

A fixed-vector map may finish a nondivisible tail with scalar execution only when the artifact proves both selected execution forms honour every required numerical dimension for the exact operations they run.

## Required delivery

- Add a scalar-epilogue iteration-tail policy distinct from exact and predicated forms; derive vector packet count and scalar remainder exactly from `N` and `W`.
- Carry both provider-versioned execution subjects for every operation reached on both paths. A scalar fact never licenses packed execution and a vector fact never licenses the epilogue.
- Require complete numerical evidence for both paths, including operation, attributes, types, provider, variant, lane form, dimension, behavior, and provenance. A missing row is `Unknown`, not a strict/default substitute.
- Bind both forms to the same artifact entry and exact output ranges without giving either path overlapping or missing ownership.
- Exercise the policy through a real native `tiler-cpu` producer and `tiler-cpu-runtime`; never use the scalar image, reference evaluator, simulator, or a mock provider as proof of mixed execution.
- Keep exact and predicated schedules separate and prohibit fallback among them.

## Closes when

One real mixed-path artifact agrees with the independent reference oracle, removing either path's numerical row refuses that path by name, and range ownership perturbations cannot pass.
