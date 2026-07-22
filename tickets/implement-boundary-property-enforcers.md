---
id: implement-boundary-property-enforcers
title: Implement executable boundary-property enforcers
status: todo
priority: p1
dependencies: [implement-boundary-property-model, transfer-synchronization-and-resource-lifetime-contract]
related: [device-placement-and-memory-domain-contract]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, transfers, lifetimes]
---
Implement explicit materialization, layout/dtype conversion, placement transfer,
synchronization, and storage-handoff operations after the property vocabulary
exists. Verify device/memory ownership, ordering, resource lifetimes, failure
boundaries, and costs; never satisfy a property through an implicit annotation.
