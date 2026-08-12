---
id: admit-predicated-fixed-vector-map-tails
title: Admit predicated fixed-vector map tails
status: todo
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir, declare-cpu-vector-realization-facts-in-the-target-profile]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/cpu, contracts/decisions, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, scheduling, predication, public-boundary]
---
## User-visible outcome

A nondivisible fixed-vector map may execute only when every inactive lane is explicit in KIR and every inactive memory access has the exact fault-suppression and effect semantics the selected CPU realization provides.

## Required delivery

- Add a distinct predicated iteration-tail policy for `FixedVectorMap`; do not reuse contributor padding or infer a mask from launch rounding.
- Derive the active-lane predicate from the exact logical extent and lane index. Prove every real output has one owner and every inactive lane performs no unguarded load, store, or arithmetic effect.
- Require explicit lane-mask KIR and exact masked load/store forms. Masking a store does not authorize an out-of-range load.
- Compose the complete operation-specific requirement collection against exact target declarations and the host-earned CPU execution approach. Unknown or unsupported mask/fault behavior refuses before publication.
- Implement and exercise the path in a real native `tiler-cpu` producer and `tiler-cpu-runtime`; no mock, simulator, Candle path, or reference vector mode counts.
- Keep `TailPolicy::Exact` byte-for-byte and behaviorally unchanged. Never fall back from predicated to scalar epilogue or scalar execution.

## Closes when

The real CPU path executes a tail-bearing fixed-vector artifact correctly, every inactive load/store perturbation fails by its own typed rule, and exact schedules remain unchanged.
