---
id: carry-the-data-flow-of-a-stage-dependency
title: Carry which slots a stage dependency's data flows through
status: closed
priority: p1
dependencies: []
related: [carry-the-stage-execution-order-in-the-envelope, preflight-every-entry-of-a-multi-stage-route]
scopes: [contracts/artifacts, implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, runtime, correctness]
closed_reason: obsolete
closed_note: The pairing is derivable from the envelope as it already stands; the runtime derives it and fails closed when it cannot.
---
Exposed by planning `preflight-every-entry-of-a-multi-stage-route`. `carry-the-stage-execution-order-in-the-envelope` made a multi-stage variant **sequenceable**; it did not make it **executable**, and the difference is where the data goes.

## The gap, verified by reading

**Fact — `crates/tiler-artifact/src/program/builder.rs`.** `binding_target` derives `(MaterializedOrigin::Internal, ValueRole::Temporary) => BindingTargetData::Internal`, which carries no discriminator. In a materialized serial sum, stage 1 writes the intermediate and stage 2 reads it; both bind the *same value*, so both encode as a bare `Internal`. A loader sees two anonymous internal slots in two entries and cannot tell they are one buffer.

**Fact — `prototypes/serial-sum-run/src/proof.rs`.** A loader allocates internals per binding: `Placement::Internal => device.new_buffer(needed, StorageModePrivate)`.

**Inference — this fails open, which nothing else in this stack does.** Extended naively to two entries, stage 2 receives a fresh private buffer and reads uninitialized device memory. No digest fails, no preflight refuses, and the values returned are plausible garbage. Every other refusal here fails closed; this one would return a wrong answer silently, which is the failure class the artifact layer exists to prevent.

## What to carry

The two binding slots a `Data` edge's data flows through: the predecessor's writing slot and the successor's reading slot.

**Slots rather than a name for each intermediate.** A durable intermediate identity is the design `BindingTarget`'s own doc rules out — the shared IR's canonical value key is crate-private with no read view publishing it, so it would force a new public surface onto `tiler-ir`. An edge already names its two entries; naming two slots inside them says exactly what a loader needs and nothing more.

**Derivable, so a producer still states nothing.** `DependencyReasonView::Data` carries the `MaterializedValueRef`; `StageRef::accesses()` yields accesses in kernel buffer-parameter order, which is binding-slot order, each with `view().value()` and `mode()`; `MaterializedValueRef` has exact equality and `StageAccessMode` separates `Read` from `Write`. The predecessor's slot is the position of its access whose value equals the edge's and whose mode is `Write`; the successor's is the matching `Read`. Same source `binding_target` reads, same posture — a producer cannot state a correspondence its plan contradicts.

**`StorageHandoff` edges carry no slots, deliberately.** They name an allocation opportunity rather than a data path: a loader that allocates separately is wasteful and correct. Encoding slots on an edge that does not need them would invite a reader to bind through them.

## Closes when

A `Data` edge names the slot its predecessor writes and the slot its successor reads; a decoder proves both are in range, that the predecessor's slot writes and the successor's reads, and that both address `BindingTarget::Internal`; `ARTIFACT_DOMAIN` steps for the reason its own doc gives; `DecodedStageDependency` publishes the pair; the producer's multi-stage case asserts the recovered data path and not only the order; and `make full` passes.
