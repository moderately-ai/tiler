---
id: prototype-scheduled-region-ir
title: Implement checked scheduled-region IR
status: todo
priority: p0
dependencies: [prototype-semantic-index-refinement]
related: [scheduled-region-model]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, verifier]
---
Implement reviewed target-neutral ScheduledRegion and KernelSchedule builders,
canonical identities, and intrinsic verifier. Validate axes, work ownership,
loops, vector/tail organization, staging, reduction topology, synchronization,
launch expressions, and specialization before target feasibility is queried.
No cost or provider callback can repair malformed schedule intent.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Module-placement note (coordinator, 2026-07-23)

Per ADR 0070 the target-neutral scheduled-region IR belongs in `tiler-ir` as its own
module, `tiler_ir::schedule`, alongside the existing `tiler_ir::index`. Build it there
rather than growing `tiler-compiler/src/physical.rs`, which currently holds the
bounded serial-Sum prototype's schedule/kernel/program types in one ~1,300-line
file. Extract only your layer's concern, keep the serial-Sum path green, and
leave the shared `physical.rs` no larger than you found it (ideally smaller).
This keeps the crate's public surface modular so later layer work can proceed
without one monolith as a shared merge point; it is architecture ADR 0070
already mandates, not extra scope.
