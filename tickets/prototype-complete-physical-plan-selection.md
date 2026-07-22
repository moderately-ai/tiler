---
id: prototype-complete-physical-plan-selection
title: Select and verify complete physical plans
status: todo
priority: p0
dependencies: [prototype-region-cover-enumeration, prototype-physical-implementation-frontier]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, program-selection]
---
Join independently verified legal covers with compatible per-region physical
frontiers. Verify complete occurrence/output coverage, boundary agreement,
materializations, dependencies, deliberate duplication, guards, and
deterministic portfolio retention. Emit a non-forgeable checked selected-plan
or selected-portfolio receipt distinct from structured KIR and `KernelProgram`.
The P0 selector may use proved structural dominance; it must not invent
uncalibrated latency authority.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.
