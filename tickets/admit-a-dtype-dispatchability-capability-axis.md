---
id: admit-a-dtype-dispatchability-capability-axis
title: Admit a dtype-dispatchability capability axis
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [metal]
---

Split from `decide-per-dtype-dispatchability-as-a-target-capability`, which settled the placement. Read its Outcome section first — the elimination is done and must not be re-litigated.

## What is already decided

- **The evidence is profile-owned, keyed by target family.** The iOS Simulator dispatches on the *same physical GPU* as macOS — identical `device_registry_id` — and refuses `bf16` pipeline creation while macOS accepts it. The discriminator is the family's runtime, not the device, so a per-device query would interrogate a GPU that demonstrably can dispatch the format.
- **Device preflight cannot carry the rejection.** The refusal occurs at `newComputePipelineStateWithFunction:`, which is `AvailabilityPhase::PreparedKernelPreflight` — after `LiveDevicePreflight`, and therefore after the one-way routing commit that ADR 0051 fixes. It keeps a role as a **defect report** when a device disagrees with its family's profile; it is not a route.
- **The axis carries the dtype**, rather than one axis per dtype. An axis per dtype grows the enum with every admitted format and changes every target-profile descriptor ever produced each time, because the axis tag table is durable identity — the hazard ADR 0074 convention 3 names.
- **An unmeasured `(family, dtype)` pair rejects.** `bf16` is `Unknown` for `IOsDevice`, never asked because no device is attached, and `Unknown` is not `dispatchable`.

## What to do

- Add the axis to `CapabilityAxis` in `crates/tiler-compiler/src/feasibility.rs` with its dtype parameter, its requirement and guarantee spaces, and its descriptor tag. **The tag table is an exhaustive match by design** — adding a variant is a build error at every encoder, which is what keeps a profile descriptor from changing silently.
- **The governed descriptor bytes will move.** `physical.rs`'s `the_governed_descriptor_bytes_do_not_move` pins them exactly and will fail; that failure is the point of the pin. Rebaseline it deliberately, recording the old value, the new value, and the regeneration command, and check `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES` still admits the result.
- Record the measured `IOsSimulator` `bf16` refusal as a profile fact, citing finding 26 rather than restating a cause: whether the cause is a missing `bfloat` lowering, an absent simulated-GPU capability, or a runtime defect is **unmeasured** and must not be asserted.
- Make the planner's rejection name the dtype and the target, per the parent's user-visible outcome.

## Closes when

A program using a dtype its selected target family cannot dispatch is rejected before an artifact is produced, with a diagnostic naming the dtype and the target; an unmeasured pair rejects rather than defaults; the descriptor rebaseline is recorded with its regeneration command; and `make full` passes.
