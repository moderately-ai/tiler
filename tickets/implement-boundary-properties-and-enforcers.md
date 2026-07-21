---
id: implement-boundary-properties-and-enforcers
title: Implement boundary properties and enforcers
status: todo
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, optimizer, physical-planning]
---
Generalize physical boundary requirements, guarantees and explicit enforcers for layout, placement, memory domain, materialization, ordering and synchronization. Inserted transfers/materializations/conversions are real typed plan operations with costs and lifetimes, not implicit annotations.
