---
id: prototype-structured-kir-slice
title: Implement the structured kernel IR slice
status: todo
priority: p0
dependencies: [prototype-complete-physical-plan-selection]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, kernel-ir, compiler-foundation]
---
Implement backend-consumable structured KIR with typed values, address spaces, explicit indexing, loads/stores, conversions, loops, predicates, reductions, and effects/barriers where applicable. Verify scope, type, ownership, bounds, effect ordering, and output coverage; backends must not reconstruct graph-specific semantics.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Module-placement note (coordinator, 2026-07-23)

Per ADR 0070 the target-neutral structured-kernel IR belongs in `tiler-ir` as its own
module, `tiler_ir::kernel`, alongside the existing `tiler_ir::index`. Build it there
rather than growing `tiler-compiler/src/physical.rs`, which currently holds the
bounded serial-Sum prototype's schedule/kernel/program types in one ~1,300-line
file. Extract only your layer's concern, keep the serial-Sum path green, and
leave the shared `physical.rs` no larger than you found it (ideally smaller).
This keeps the crate's public surface modular so later layer work can proceed
without one monolith as a shared merge point; it is architecture ADR 0070
already mandates, not extra scope.
