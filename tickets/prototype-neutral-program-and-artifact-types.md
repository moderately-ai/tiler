---
id: prototype-neutral-program-and-artifact-types
title: Implement neutral program and artifact types
status: todo
priority: p0
dependencies: [prototype-structured-kir-slice, harden-compiler-verifier-subject-binding-and-totality]
related: [prototype-artifact-slice]
scopes: [implementation/ir, implementation/artifact, implementation/compiler]
shared_scopes: []
paths: []
tags: [implementation, artifact, program-planning]
---
Implement reviewed neutral KernelProgram and artifact-facing types: stage DAG, checked ABI/launch expressions, values/views/allocations/lifetimes/handoffs, named outputs, complete portfolios/routing predicates, and target/provider provenance. Runtime consumers must not depend on optimizer internals. Tom reviews consequential public boundaries before acceptance.

Carry the ADR 0072 layers explicitly: semantic graph identity, bound
refinements/implementations and complete coverage in program identity, and only
reached admission plus selected capability-provider provenance in packaged
artifacts. Unused registry providers remain request-environment provenance.
