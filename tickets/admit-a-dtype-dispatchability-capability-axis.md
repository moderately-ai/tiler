---
id: admit-a-dtype-dispatchability-capability-axis
title: Admit a dtype-dispatchability capability axis
status: awaiting-decision
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

## Blocked — the parent's premise does not hold, and the remainder is Tom's (2026-07-27)

The parent decided the axis should "carry the dtype" and leave "the dtype vocabulary where it already lives". Attempting the implementation found that **there is no dtype vocabulary the compiler can reach that names `bf16` at all**, so there is nowhere for it to already live.

**Fact.** `grep -rn "Bf16\|bfloat" crates/tiler-ir/src` returns nothing. `tiler_ir::kernel::KernelType` is `Bool | Index | F32`. The only vocabulary in the workspace naming `bf16` is `tiler_metal::target::MetalFloatArithmeticType`, and `crates/tiler-compiler/Cargo.toml` depends on `tiler-ir` and not on `tiler-metal`.

**Fact.** `CapabilityFact` is `{ axis: CapabilityAxis, bound: u64, phase, authority, validity, provenance }`. The axis is a bare `Copy` enum carrying no parameter, and the quantitative space is documented as "every axis has a `u64` bound, a `Quantity` unit, and a comparison `Relation`". Encoding a dtype into the `u64` bound is not an option: that is the same conflation `name-the-capability-api-version-authority-or-retire-the-requirement` recorded — a real value in the wrong slot.

**Fact.** `TypeKey` derives `Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd` and **not** `Copy`, so parameterizing the axis by it makes `CapabilityAxis` non-`Copy`.

## The question

**Which vocabulary names a dtype for target-capability purposes?** Three candidates, each with a different blast radius, and the choice places a dtype vocabulary in a layer for the long term.

1. **Widen `tiler_ir::kernel::KernelType` with `Bf16`.** *Enables:* the axis stays `Copy` and the vocabulary is already shared and already in artifact identity. *Prevents:* nothing — but it asserts the structured kernel IR admits `bf16` values, which no lowering, emitter, or reference oracle implements. It is a total map into artifact identity, so the tag table and every encoder change, and the claim "the IR supports this type" becomes true by declaration rather than by support. `AGENTS.md` separates a type-system reservation from implemented support precisely here.

2. **Parameterize the axis by `TypeKey`,** the extensible semantic vocabulary. *Enables:* `bf16` is a registered semantic type like any other, extension-friendly, and no shared IR claims support it does not have. *Prevents:* `CapabilityAxis` stops being `Copy`, which ripples through the feasibility surface, and a capability fact starts carrying a heap value into a descriptor that is durable identity.

3. **A separate dispatchability vocabulary owned by the target-profile layer,** distinct from both. *Enables:* the axis space stays quantitative and untouched, and dispatchability is modelled as its own fact family rather than squeezed into a bound comparison. *Prevents:* a second dtype vocabulary exists, which is the duplication `AGENTS.md` warns about — two lists that must agree and that nothing checks.

**Recommendation: 2.** It is the only one that neither asserts unimplemented IR support nor creates a second dtype vocabulary, and the extensible semantic registry is already the layer that decides what types exist. **Counterpoint:** it is also the only one that changes a `Copy` type into a non-`Copy` one on a hot feasibility path, and today's `admit-a-caller-declared-target-profile` work shows exactly how far that kind of ripple reaches — a similar `Copy` removal produced 57 compiler errors across four files.

This is reserved rather than decided because it is a durable placement of a vocabulary, not an implementation detail: whichever layer owns it will own every dtype admitted afterwards.

### What is ready to proceed the moment it is answered

The elimination in the parent stands and is not affected by the answer: profile-owned, keyed by target family, rejecting on unmeasured, with device preflight as a defect report rather than a route. The descriptor rebaseline procedure is recorded above.
