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

## Not startable as written — no stated outcome (2026-07-27)

**This ticket has no `## Closes when` and no sections.** "Executable boundary-property enforcers" names a mechanism without naming which properties, at which boundaries, or what an enforcer refuses.

**What it needs before it is claimable.** The list of properties to enforce and, for each, what currently checks it and what would instead. `implement-boundary-property-model` created the `AccessMode` total-map site recorded under ADR 0074 convention 5b, so the model exists; this ticket is the enforcement half and should name which of the model's properties are unenforced today.
