---
id: prototype-kernel-program-ir
title: Implement verified target-neutral KernelProgram IR
status: todo
priority: p0
dependencies: [prototype-structured-kir-slice]
related: [prototype-artifact-program-model]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, program-planning]
---
Implement reviewed verified target-neutral KernelProgram IR: stage DAG, exact
selected scheduled/KIR refinements, checked values/views/allocations and
lifetimes/handoffs, named outputs, dependencies, and complete coverage. This is
compiler execution intent, not the artifact manifest or codec. Tom reviews
consequential public boundaries before acceptance.

Carry the ADR 0072 layers explicitly: semantic graph identity, bound
refinements/implementations and complete coverage in program identity. The
artifact-facing projection owns packaged admission/provider provenance and
routing/ABI representation.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
