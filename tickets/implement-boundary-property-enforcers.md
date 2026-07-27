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
Make physically compatible region implementations composable by inserting
explicit, value-preserving materialization, layout conversion, encoding
repacking, placement transfer, synchronization, and storage-handoff steps.
Verify ownership, ordering, resource lifetime, failure boundary, feasibility,
and cost. A boundary enforcer may change storage, addressing, placement, or
delivery, but never semantic dtype or tensor value.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
