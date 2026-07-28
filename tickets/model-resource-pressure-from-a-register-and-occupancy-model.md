---
id: model-resource-pressure-from-a-register-and-occupancy-model
title: Model resource pressure once a register and occupancy model exists
status: deferred
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, target-profiles]
---
Split from `model-the-eight-unmodelled-cost-components`, which modelled seven of nine components and reached its floor. `ResourcePressure` is the one component with a genuine, checked blocker.

## Why it is blocked rather than merely unwritten

`CostComponent::ResourcePressure` means "register and threadgroup-memory pressure, and the occupancy it implies". Threadgroup memory was split into its own exact component and is done. What remains is **registers per thread** and the **occupancy model** that would combine the two into pressure, and neither exists anywhere in the compiler.

*The check, stated so it can be reproduced or refuted in one line:* `ResourceRequirements` (`schedule/model.rs:413`) carries `buffer_bindings`, `threads_per_workgroup`, `local_memory_bytes`, `barriers`, `requires_device_memory`, and four numerical fields — no register count. `grep -rn 'register' crates/tiler-compiler/src/capability.rs` returns only capability *registration*, never a GPU register. The target capability axes exercised in the explain census are barriers, buffer-bindings, device-memory, grid-axis, index-bits, local-memory-bytes, threads-per-workgroup, and the numerics dimensions — again no register axis.

This is a missing **model**, not a missing summary. That distinction earned its emphasis: six of the nine original "unreachable" notes on the parent ticket turned out to be missing summaries that were in fact one read away, and this one was re-checked twice before being written down.

## The constraint that must survive

Do not repair this by widening `CostComponent::unit`. Reporting some other quantity under a `Registers` unit would be a unit lie, and units here are contract rather than documentation — an uncalibrated model whose numbers have no true stated unit cannot be calibrated, because nothing says what the device measurement should be compared against. A missing number is recoverable; a number in the wrong unit is what a calibration pass silently trusts. That reasoning is why threadgroup memory became its own component instead of being folded in here.

## Closes when

- A target profile declares register-per-thread and occupancy axes, typed like the existing capability axes.
- `ResourcePressure` is computed from them, in `Registers`, with `Bounded` rather than `Exact` unless the derivation is genuinely exact.
- The explain census in `pipeline/tests.rs` is updated in the same change; its `tiler.cost.analytical.v1` count grows as components become modelled, and that test is what catches an unreported one.
- The retained plan set and the selected plan are unchanged, as for every other component: nothing here enters dominance.

## Trigger for reconsideration

Any work that adds register or occupancy information to a target profile. `implement-opaque-physical-call-providers` is the nearest candidate — its body calls for "uncertain `ResourceEstimate`-class pressure estimates with provenance and an explicit `Unknown` state, including registers, occupancy, and source size", which is the same vocabulary this component needs. If that ticket lands its estimate class, check whether this becomes computable from it.
