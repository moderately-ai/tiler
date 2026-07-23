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

## Module-placement note (coordinator, 2026-07-23)

Per ADR 0070 the target-neutral kernel-program IR belongs in `tiler-ir` as its own
module, `tiler_ir::program`, alongside the existing `tiler_ir::index`. Build it there
rather than growing `tiler-compiler/src/physical.rs`, which currently holds the
bounded serial-Sum prototype's schedule/kernel/program types in one ~1,300-line
file. Extract only your layer's concern, keep the serial-Sum path green, and
leave the shared `physical.rs` no larger than you found it (ideally smaller).
This keeps the crate's public surface modular so later layer work can proceed
without one monolith as a shared merge point; it is architecture ADR 0070
already mandates, not extra scope.
