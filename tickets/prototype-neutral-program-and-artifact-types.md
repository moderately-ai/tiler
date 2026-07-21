---
id: prototype-neutral-program-and-artifact-types
title: Implement neutral program and artifact types
status: todo
priority: p0
dependencies: [prototype-structured-kir-slice]
related: [prototype-artifact-slice]
scopes: [implementation/ir, implementation/artifact, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, artifact, program-planning]
---
Implement reviewed neutral KernelProgram and artifact-facing types: stage DAG, checked ABI/launch expressions, values/views/allocations/lifetimes/handoffs, named outputs, complete portfolios/routing predicates, and target/provider provenance. Runtime consumers must not depend on optimizer internals. Tom reviews consequential public boundaries before acceptance.
