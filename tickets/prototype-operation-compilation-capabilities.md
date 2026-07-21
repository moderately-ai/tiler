---
id: prototype-operation-compilation-capabilities
title: Implement operation compilation capabilities
status: todo
priority: p0
dependencies: [prototype-shared-compiler-ir-ownership]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, compiler-foundation, extensions]
---
Add versioned typed provider capabilities for index/access lowering, scalar lowering, materialization constraints, numerical/effect declarations, and physical implementations or opaque calls. Resolution and provenance must be deterministic; disjoint providers may contribute evidence; missing or ambiguous capability fails closed.
