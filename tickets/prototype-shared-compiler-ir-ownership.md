---
id: prototype-shared-compiler-ir-ownership
title: Establish shared compiler IR ownership
status: done
priority: p0
dependencies: [reconcile-implementation-delivery-graph]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/workspace, contracts/foundation, contracts/artifacts, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, milestone-0b]
---
Place occurrence references, symbolic access/index IR, schedules, structured KIR, KernelProgram, and neutral ABI-facing types in reviewed owning layers with an acyclic crate graph. Expose only the smallest ordinary non-test compiler path required downstream; keep IR levels distinct and compile-check dependency direction. Tom reviews consequential public crate, module, trait, type, and call-site boundaries before acceptance.

## Outcome

ADRs 0070 and 0071 establish `tiler-ir` ownership for the index, schedule,
structured-kernel, executable-program, and ABI-expression layers together with
their checked-builder and opaque-verified-wrapper lifecycle. The accepted
prototype crate graph now removes the unused backwards
`tiler-compiler -> tiler-artifact` edge, and the workspace conformance gate
enforces that direction. The bounded compiler pipeline is compiled as ordinary
library code rather than existing only under `cfg(test)`.

The current serial-Sum-shaped occurrence, physical, KIR, and program structs
remain private rather than being promoted as provisional public IR. Their
replacement by the accepted public representations and authoritative
verifiers is dependency-ordered through the operation-capability, canonical
index-region, physical-frontier, structured-KIR, neutral-program, and optimizer
conformance tickets. No public compiler facade is accepted before those seams
become general, as required by ADR 0069.
