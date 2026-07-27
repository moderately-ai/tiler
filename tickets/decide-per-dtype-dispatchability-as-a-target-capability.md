---
id: decide-per-dtype-dispatchability-as-a-target-capability
title: Decide whether per-dtype dispatchability is a stated target capability
status: done
priority: p2
dependencies: []
related: [measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [research, metal, target-profiles, feasibility]
---
Finding 26 of [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md) measures a case the physical contracts have no name for: the iOS Simulator compiles every `bfloat` probe kernel to LLVM IR, to AIR, and links it to a metallib without a diagnostic, and its device then fails `newComputePipelineStateWithFunction:` with `XPC_ERROR_CONNECTION_INTERRUPTED` — on both the offline and runtime compilation paths. The same simulator runs every `f32` and `f16` kernel in the same invocation. The refusal is measured to be about the *format* rather than any operation on it: the arithmetic-free `materialize_bf16`, which emits zero floating-point operations, is refused too.

So "it compiled for this target, therefore it runs there" is false on a measured row, and the dtype is what makes it false.

**User-visible outcome.** A program using a dtype that the selected device
cannot dispatch must be rejected at the earliest point where Tiler can know
that fact, with an explanation that identifies the dtype and target. It must
never be packaged as though it were executable, reach the one-way routing
commit, and then silently fall back.

The work is to determine which component has enough reliable evidence to make
that rejection. Eliminate any candidate that cannot preserve that outcome
before asking for an architectural choice:

- **Profile-owned capability evidence.** A planner can reject a `bfloat` plan
  before producing an artifact, but only if the capability is scoped by the
  target family and other authorities that can change the answer. An unmeasured
  entry must reject rather than default.
- **Device preflight evidence.** Pipeline creation can test the actual device,
  but the result arrives after artifact production. Establish whether that
  check can occur before routing commit and program work and whether the
  artifact must disclose that it still requires the check.

The failure is loud rather than silent today, so this is not evidence of a
wrong result. It is a placement and explainability gap.

**What this is not.** Not the cause of the simulator's refusal, which is unmeasured and is not on the critical path for the decision — whether *this* runtime lacks `bfloat` lowering does not change whether the architecture should carry per-dtype dispatchability. Not the subnormal-flush fact's dtype, which [carry-the-dtype-on-the-metal-subnormal-flush-fact](carry-the-dtype-on-the-metal-subnormal-flush-fact.md) holds.

**What closes this.** The reliable evidence source and earliest rejection point
are established; candidates that cannot meet the user-visible outcome are
discarded with their derivation; and the surviving contract is recorded, or a
genuine remaining product choice is presented atomically with a recommendation
and counterpoint.

## Outcome — profile-owned, and the candidates are eliminated by measurement (2026-07-27)

Both candidates were tested against the user-visible outcome. One survives, and the fact that decides it is in the record already.

### The decisive measurement: the simulator's GPU *is* the Mac's

`docs/research/apple-targets/numerical-behaviour.md` records that `environment.family.ios-simulator.device_registry_id` is **identical to the macOS one** — the iOS Simulator dispatches on the host GPU under a different device name. So the same physical GPU runs every `bf16` kernel under `MacOs` and refuses to create a pipeline for it under `IOsSimulator`.

**Inference: the discriminator is the target family, not the device.** A per-device capability query would be asking the wrong subject — it would interrogate a GPU that demonstrably *can* dispatch the format, in a runtime that will not let it. Whatever the cause is (a missing `bfloat` lowering in the simulator runtime, an absent capability on the simulated device, or a defect in that runtime build — all three **unmeasured**, and not guessed at here), it travels with the runtime the family names.

### Candidate 2 — device preflight as the source — is eliminated

**Fact: the refusal occurs at `newComputePipelineStateWithFunction:`**, which is pipeline preparation. `AvailabilityPhase` already names that moment: `PreparedKernelPreflight`, one step *after* `LiveDevicePreflight`.

**Fact: a fact first knowable at that phase is too late to route around.** `AGENTS.md` fixes the ordering — preflight before routing commit, fallback only before program work, and no fallback after allocation, partial encoding, or submission — and ADR 0051 makes the routing commit one-way. `crates/tiler-ir/src/index/sourced.rs` records the same ceiling for the same reason: `EXTENT_PHASE_CEILING` is `LiveDevicePreflight`, because anything later creates a dependency from a decision back to the thing it decided.

So device preflight cannot be the *earliest* point and cannot be the *only* point: by the time it speaks, the artifact exists and the route is committed. It keeps a role — a device that disagrees with its family's declared profile is a **defect to report**, not a route to take — but it cannot carry the rejection the outcome requires.

### Candidate 1 — profile-owned capability evidence — survives

`tiler_metal::target::MetalPlatform` already declares `IOsSimulator` as a distinct family beside `MacOs` and `IOsDevice`, and `AppleSdk` mirrors it. The measured refusal is a property of that family, so a profile for it can carry the fact and the planner can reject before an artifact exists — which is exactly the outcome.

### What blocks it, and why that is not a product choice

**Fact: no existing `CapabilityAxis` can express this.** The vocabulary is `GridAxisThreads`, `WorkgroupThreads`, `BufferBindings`, `IndexWidthBits`, `DeviceAddressSpace`, `LocalMemoryBytes`, `Barriers` — every one a scalar bound or a boolean, and none indexed by a dtype. Two spellings exist and the constraints pick one:

- **One axis per dtype** (`Bf16Dispatchable`, …) puts a dtype vocabulary inside the axis enum and grows it with every admitted format. Every target-profile descriptor ever produced changes each time a dtype is added, since the axis tag table is durable identity.
- **One axis carrying a dtype** (`DtypeDispatchable { dtype }`) keeps the axis count fixed and leaves the dtype vocabulary where it already lives.

The second is not a preference: the first makes admitting a dtype a breaking identity change for every profile, which is the hazard ADR 0074 convention 3 names. **No genuine product choice remains, so none is escalated.**

### The rule an unmeasured pair must follow

An unmeasured `(family, dtype)` pair **rejects rather than defaults**, as the ticket requires. `bf16` is `Unknown` for `IOsDevice` — never asked, because this host has no attached device — and `Unknown` is not `dispatchable`. A profile that defaulted to permitting would package an artifact for a family nothing has measured, which is precisely the "compiled therefore runs" inference the measurement refuted.

## Split out

`admit-a-dtype-dispatchability-capability-axis` carries the implementation: the axis shape, its requirement and guarantee spaces, its descriptor tag, the `IOsSimulator` profile fact, and the reject-on-unmeasured rule. This ticket held `contracts/decisions` scope only and the decision is what it owed.
