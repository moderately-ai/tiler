---
id: implement-boundary-property-model
title: Implement the physical boundary-property model
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, physical-planning]
---
Generalize typed physical boundary requirements, guarantees, satisfaction,
subsumption, child requirement derivation, dominance, identity, and explain for
layout, alignment, materialization, placement, memory domain, ordering, and
synchronization. This ticket defines properties only; executable transfers,
materializations, conversions, synchronization, and lifetime verification are
owned by `implement-boundary-property-enforcers`.
