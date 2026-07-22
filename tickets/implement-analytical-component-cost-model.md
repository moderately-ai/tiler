---
id: implement-analytical-component-cost-model
title: Implement an analytical component cost model
status: todo
priority: p1
dependencies: [implement-boundary-property-model]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, cost-model, performance]
---
Implement deterministic symbolic component costs for memory traffic,
allocation, dispatch, redundant work, indexing, synchronization, resource
pressure/occupancy, compile time, and artifact size. Preserve units,
assumptions, uncertainty, target-profile subjects, and typed explain; hard
feasibility remains separate. This is explicitly analytical and uncalibrated.
`calibrate-device-cost-models` owns later device measurements and activation.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
