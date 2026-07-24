---
id: prototype-structured-kir-slice
title: Implement the structured kernel IR slice
status: in-progress
priority: p0
dependencies: [prototype-complete-physical-plan-selection]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, kernel-ir, compiler-foundation]
claimed_from: todo
assignee: agent-prototype-structured-kir-slice
lease_expires_at: 1784906005
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

## Outcome (2026-07-24)

**Fact — what exists.** `tiler_ir::kernel` is a new public module in `tiler-ir` (`handles.rs`, `error.rs`, `model.rs`, `builder.rs`, `verify.rs`, `lower.rs`, `tests.rs`). It owns the backend-consumable structured kernel IR, its transactional `KernelBuilder`, whole-kernel verification, the canonical lowering `lower_scheduled_region`, and a canonical `CanonicalKernelIdentity`. A `KernelBuilder` can only be opened against a `tiler_ir::schedule::VerifiedScheduledRegion`, and `VerifiedKernel`'s fields are module-private, so no consumer can forge or thaw one.

**Fact — what the IR makes explicit.** Typed SSA values (`KernelType`); governed address spaces (`AddressSpace`) on every `BufferParameter` together with element type, access mode, and exact addressable element count; explicit element-offset arithmetic (`BinaryOp::Index*`) over admitted launch builtins and loop induction variables; typed `Load`/`Store` carrying the scheduled `BoundsWitnessId` and `OwnershipWitnessId`; a named conversion (`ConvertOp::CanonicalizeF32Nan`) for the numerical contract's NaN normalization that was previously an implicit rule; a bounded `SerialLoop` with an explicit trip count and typed loop-carried accumulator instead of an opaque reduce; an explicit `Predicated` region for iteration-domain guarding; and a `BarrierSpec` with separately named execution scope, memory scope, fenced address spaces, and ordering. A reduction-contributor address is emitted as the row-major linearization of the scheduled access, so a backend never re-derives an access relation, a reduction order, or a numerical rule.

**Fact — extraction.** `crates/tiler-compiler/src/physical.rs` fell from 1,676 to 1,053 lines. The removed layer concern was the whole ad-hoc kernel vocabulary (`KernelValueType`, `BufferAccess`, `KernelBuffer`, `BinaryF32`, `StructuredBody`, `StructuredKernel`, `VerifiedStructuredKernel`), its five-template lowering, and its five `*_body_refines` checkers. `physical::lower_structured_kernel` is now a thin forwarder that re-attributes a `KernelLoweringError` to the region for the explain trace. `physical.rs` retains only compiler-owned refinements: semantic-occurrence binding, request-subject binding, and target feasibility.

**Fact — verification rules and their negative tests.** Insertion-time (`KernelBuildError`): foreign handle, out-of-scope value, type mismatch, buffer access-mode violation, undeclared/duplicate builtin, non-constant or zero divisor, invalid loop range, empty accumulators, yield arity and yield type, single-assignment components. Whole-kernel (`KernelDiagnostic`): incomplete kernel, buffer contract, address-space contract, builtin contract, numerical realization, resource requirements, predicate dominance, bounds evidence, ownership evidence, output coverage, effect ordering, barrier count, reduction contract, and finally body refinement. Each has a dedicated negative test in `crates/tiler-ir/src/kernel/tests.rs` that builds a deliberately wrong kernel through the public builder. `ScheduleAccessCount`, `ContributorDomain`, and `ElementCountOverflow` are defence-in-depth boundaries that a `VerifiedScheduledRegion` already excludes and therefore carry no negative test.

**Inference — why derive-and-compare is the last gate.** Specific rules run first so a rejection names the exact violated obligation. The final gate re-derives the canonical body from the verified scheduled region and requires structural equality. This is a deliberately bounded profile: a semantically equivalent but differently spelled body is rejected as `BodyRefinement` rather than admitted by an unproven equivalence argument. The canonical derivation itself runs through the same public builder, so it cannot bypass an insertion-time invariant.

**Measurement.** `uv run --locked python scripts/check_repository.py` passes on this branch (nightly-2026-07-19, macOS arm64). `cargo nextest run --workspace` reports 408 tests run, 408 passed. The KIR interpreter added to `pipeline.rs` tests reads only the structured kernel — no semantic graph, request, or schedule — and reproduces the reference evaluator bit-for-bit on the fused serial-Sum conformance vectors, including a leading reduced axis (`[3, 2]` over axis 0) and a middle reduced axis (`[2, 3, 2]` over axis 1) that force the emitted index division and remainder.

**Fact — collateral.** Three `crates/tiler-ir/tests/typed-handles/fail/*.stderr` fixtures were re-blessed: rustc now prints `tiler_ir::semantic::F32` instead of `F32` because the new module introduces a second crate-level `F32` name (`KernelType::F32`). The rejections asserted by those fixtures are unchanged.

**Proposal — open for review.** The public surface of `tiler_ir::kernel` is a draft until Tom accepts the exact commit. The `BufferAccess::ReadWrite`/atomic modes, additional conversions, collectives, multi-result operations, and a non-zero barrier profile are reserved typed extension points that currently reject explicitly rather than approximate.

**Deferred.** `MAX_KERNEL_IDENTITY_BYTES` has no negative test (a 16 MiB body is impractical to construct in a unit test). The structured KIR is not yet consumed by `tiler-metal` or `tiler-artifact`; wiring a backend to it is separate work.
