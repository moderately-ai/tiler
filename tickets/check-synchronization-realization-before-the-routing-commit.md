---
id: check-synchronization-realization-before-the-routing-commit
title: Check synchronization realization before the routing commit
status: done
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal]
scopes: [implementation/compiler, implementation/runtime, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A host refuses a route whose entries require a synchronization realization the executing backend does not declare, before the routing commit and with a typed reason naming the exact subject.

## Why this is not already covered

Facts re-read in full at base `80837d0de5072774ad0e633728cf4774236749df`. **Every line-number citation this ticket carried was stale**, and each landed on real but unrelated code — the shape a spot-check does not catch. Citations below are searchable anchors.

**Fact (verified in substance, citations repaired) — the requirement reaches the artifact and nothing reads it.** `ResourceRequirements::synchronization` is an `Option<SynchronizationSubject>` carrying the complete five-dimension subject a region's schedule requires. It is derived by `cooperative_synchronization_requirement` in `derive_requirements`, encoded by `push_resources` → `push_synchronization` (`crates/tiler-artifact/src/program/model.rs`), decoded by `fn synchronization` in `crates/tiler-artifact/src/program/codec/decode.rs`, and published through `pub fn resources` on `DecodedEntry` (`crates/tiler-artifact/src/program/codec/view.rs`).

- *Stale:* `model.rs:871` → the field is at `:1226`; `871` is inside the `MultiPass` reduction-topology variant.
- *Stale:* `view.rs:673` → `pub fn resources` is at `:692`; `673` is the `DecodedEntry` struct definition, which is exactly plausible enough to stop a reader.
- *Imprecise:* the reproduce one-liner is **not** empty. `rg -n 'resources\(\)\.synchronization' crates prototypes spikes` returns two hits, both in `tiler-artifact`'s own round-trip tests asserting `== None`. A worker running it as written sees output and may conclude the opposite of what the Fact claims. The conclusion — that no loader, adapter, or prototype *compared* it against anything — was nonetheless correct.

**Fact (premise verified, conclusion FALSE) — the compiler's comparison authority is crate-private, and it is the wrong authority.** `CheckedTargetProfile::resolve_synchronization` does compare a whole subject against `DeclaredSynchronizationRealization` rows, and `SynchronizationRealization` is `pub(crate)` in `crates/tiler-compiler/src/target/feasibility.rs` (`resolve_synchronization` is in fact wholly private, not `pub(crate)`).

The ticket's conclusion — "a runtime-side check therefore needs a decided public boundary" in `tiler-compiler` — **does not follow and is refuted by the crate graph**. `tiler-runtime`'s dependency closure is fixed at `[tiler-artifact]` under ADR 0081, and its `Cargo.toml` states that a `tiler-compiler` edge is forbidden because a loader that could "rebuild a plan instead of validating the one it was handed" is the boundary the crate split exists to enforce. A compile profile's declared rows are a *compile-time* fact about a target the producer built for. The delivery-time authority is what the **executing backend's own vocabulary** can deliver, and publishing the compiler's rows to the runtime would mint the second, independently editable statement about one fact that this repository forbids everywhere else.

**Fact (verified in substance, citation repaired) — the refusals that do exist are at emission, not preflight.** `MetalEmitError::UnsupportedBarrier` with `BarrierRejection::{MemoryVisibility, FencedSpace}` refuses an unrealizable barrier while emitting, and those two arms are covered by `no_metal_barrier_establishes_device_wide_visibility`, `a_simd_group_barrier_cannot_claim_workgroup_visibility`, and `a_space_without_a_fence_flag_is_rejected` — all three verified present in `crates/tiler-metal/src/tests.rs`.

- *Stale:* `emit.rs:1601` → `1601` is inside `emit_binary`'s `BinaryRealization::MaximumF32` arm. The barrier refusals were at `:1968`, `:1990`, `:2003`, and `:2035`. Anchor on `fn barrier_realization` instead.

**Fact (verified in substance, both citations stale) — two of `BarrierRejection`'s arms are unreachable rather than untested.** `ExecutionScope` declares exactly `Subgroup` and `Workgroup` and both are accepted by the preceding arm; `BarrierOrdering` declares exactly `AcquireRelease` and it is accepted likewise.

- *Stale:* `kernel/model.rs:595` → `pub enum ExecutionScope` is at `:788`; `595` is in `BinaryOp`'s `operand_type`.
- *Stale:* `kernel/model.rs:633` → `pub enum BarrierOrdering` is at `:826`; `633` is inside `UnaryOp::F32Rsqrt`'s doc comment.

This Fact **survives the change made here**, deliberately. The delivery-time inversion refuses an unspellable arrival or ordering by its own name *before* the emission authority is consulted, so neither arm becomes reachable and neither gains a test.

## Implementation keys

**Corrected: this is a derived requirement, not a route requirement.** The ticket directed the work at "the same split `tiler.metal.route-requirement.minimum-gpu-family` already takes, where `tiler-runtime` owns the decision and the adapter owns the observation". That is wrong twice.

- It conflates two different things sharing a name. `tiler.metal.route-requirement.minimum-gpu-family` is a producer-emitted `RouteRequirement::BackendFeature` row. `tiler_metal::direct_requirement::minimum_gpu_family` is a *derived* requirement whose module documentation is headed **"Why this is not a route requirement"**.
- For a `BackendFeature` row the **adapter** owns the decision, not `tiler-runtime`: `LiveDeviceObservation::Feature` is documented as "The owning adapter decided this qualitative row for the bound device", and the loader only reads the boolean. The runtime owns the comparison for a `Resource` row.

`crates/tiler-artifact/src/program/requirement.rs` settles which family applies: a row belongs to the backend-feature family only when it is consumed by the selected executable route and "not already derivable from its verified program". `synchronization` is derived by `cooperative_synchronization_requirement` from a cooperative tile's visibility edges, so it fails the second half exactly as `index_arithmetic` does. It is categorically a **derived requirement**, checked directly off the routed entry's `ResourceRequirements`, and a route-requirement row restating it would be the second authority that contract exists to prevent.

The subject is matched as one whole value and never dimension by dimension: each of its five dimensions is separately true of some realization, so a conjunction inferred from independent facts is not a statement about any realization. Refuse with a typed reason naming the required subject whole, classified as a route miss — another variant may declare a subject this backend does realize.

The check belongs before the routing commit. Concretely it belongs in the **derived-requirement discharge stage**, which `resolve_prepared_route` already runs *one stage earlier* than the live-device requirement resolution the ticket pointed at — the correct order, because a synchronization realization needs no device at all.

## Required evidence

A negative fixture refuses an entry requiring a subject the backend does not declare, before the commit, and the refusal names the subject. A positive neighbour routes, so the refusal is evidence about the perturbation. Every new check is mutation-proved. Targeted tests plus `make full` pass.

## Worker outcome

`tiler_metal::synchronization_requirement` decides whether this backend realizes an entry's required subject, and `prototypes/serial-sum-run`'s `check_direct_requirements` runs it over every entry before any pipeline is prepared — one stage ahead of the live-device rows, and well before the commit. The discharge witness `DirectRequirementsDischarged` already gated `prepare_pipelines`, so the new check inherits the compile-time ordering guarantee rather than adding a convention.

**One authority, not a table beside emission.** `barrier_call` was split into `barrier_realization` (the decision, returning `BarrierRejection`) and the statement formatting. Both emission and the delivery-time check now consult the same function, which is demonstrated rather than asserted: perturbing `barrier_realization` to admit device-wide visibility reddens the pre-existing emission test `no_metal_barrier_establishes_device_wide_visibility` **and** the new `a_device_wide_publication_is_refused_and_names_the_whole_subject` together.

The module's own work is the *inversion* — the neutral schedule vocabulary is wider than the kernel spelling vocabulary on three axes, and each gap is refused by name rather than rounded onto a neighbour. Rounding is the dangerous repair: a device-wide arrival narrowed to a workgroup one, or a sequentially-consistent ordering weakened to acquire-release, spells a barrier that emits cleanly and orders less than the schedule proved it needed.

**Feasibility, not cost.** Nothing added ranks or prices anything. A refused subject is a plan whose barrier could never have been written.

**Why no device observation is taken.** Metal's barrier builtins and their coupled visibility are fixed by the language; no `MTLDevice` property varies them. `tiler-compiler`'s own feasibility authority records the same thing where it explains why such a fact can never be deferred to a runtime query. The two evidence classes stay in separate modules so no reader concludes that changing the device could change this answer.

## Graph maintenance

- Keep the emission-time barrier refusals where they are; this added a delivery-time check rather than moving one.
- The two unreachable `BarrierRejection` arms stay unreachable and untested; see the corrected Fact above.
- **Remainder, not done here:** `prototypes/candle-metal-adapter` discharges *no* derived requirement — neither the new synchronization check nor the pre-existing `evaluate_index_arithmetic`. That gap predates this ticket and sits in `implementation/candle`, which this ticket does not hold. It needs its own ticket.
- **Unmeasured:** the wiring in `prototypes/serial-sum-run` is exercised only by a device-bound run, which this coordination host cannot perform. The authority itself is fully covered device-free.
