---
id: admit-typed-byte-alignment-and-effective-program-view-guarantees
title: Admit typed byte alignment and effective program-view guarantees
status: done
priority: p1
dependencies: [separate-vector-operand-alignment-from-target-realization]
related: [derive-boundary-alignment-from-the-element-type, carry-the-byte-offset-of-a-partial-binding-view, accept-the-typed-byte-alignment-surface]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/foundation, contracts/optimizer, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [alignment, placement, ir, public-boundary, correctness]
---
## Acceptance

Tom accepted this exact boundary on 2026-08-12 in ChatGPT, relayed by the repository coordinator. The accepted surface is the role-safe checked alignment vocabulary, effective alignment derived from the actual view byte offset, and byte-identical encoding for every previously valid value. The acceptance does not authorize target-provider access requirements, runtime pointer observation, vector KIR, allocator policy, or a new artifact field.

## User-visible outcome

Every alignment value is checked once, requirements cannot be confused with guarantees, and a partial view advertises only the alignment its actual byte offset preserves.

## Facts at worker base `be47030991284cdd51840b9c5df9be3642365c1a`

Filing-base Facts were re-read in full at this commit before any edit. Coordinator-supplied Facts 1–3 match. Coordinator Fact 5 is discharged here: encoder pins, `check_stage_accesses`, and `StorageScalar::byte_width` were re-read rather than trusted from the ticket.

- **Verified.** `crates/tiler-compiler/src/boundary.rs`, anchor `pub(crate) struct ByteAlignment(u32)`, is crate-private. `new` refuses zero and non-powers of two; `satisfies` is one-directional divisibility; `natural_for` derives from `StorageScalar::byte_width` and panics only if a carrier width is unrepresentable.
- **Verified.** `AllocationSpec` documents `alignment` as the allocation's guarantee and `MaterializedValueSpec` / `MaterializedComponentSpec` document theirs as a requirement, but every one is `pub alignment: u32`. Artifact `BindingData` is `pub(super) alignment: u32`. The identical raw type still permits reversed comparisons and repeated validation.
- **Verified.** `KernelProgramBuilder::push_view` takes a `ByteWindow` and checks only range, packed-partial refusal, and duplicate windows. It does not derive the alignment of `base + offset`. `check_stage_accesses` checks arity, access mode, `ValueRole::fills`, component role, element type, addressed byte count, and the accessible-bytes ABI expression. It does not check address alignment.
- **Verified.** Identity encoders already write the alignment as a fixed-width big-endian `u32` (`value.alignment.to_be_bytes()` in `value_key` / `push_value`; `allocation.alignment.to_be_bytes()` in `allocation_key` / `push_allocation`) and write each view as `(offset, length)` with no derived alignment field. Artifact bindings encode the same four-byte run. A typed in-memory replacement that keeps that spelling preserves every old valid canonical byte.
- **Verified.** `StorageScalar::byte_width` is the single unpacked-width authority: exhaustive `U8 => 1`, `F32 => 4`, `Bf16 => 2`. There is no second carrier-width table.

## Required delivery

- Move the single checked `ByteAlignment` authority to the lowest shared physical-program vocabulary and expose opaque `AlignmentRequirement` and `AlignmentGuarantee` roles. Re-export the received type through `tiler-artifact` so `tiler-runtime` keeps ADR 0081's no-direct-IR dependency.
- Keep construction fallible and typed. Zero and non-power-of-two values are errors; there is no `Default`, unchecked public constructor, rounding, clamp, or integer sentinel.
- Put the comparison only on the guarantee: `guarantee.satisfies(requirement)`. Derive natural requirements from `StorageScalar::byte_width`; do not create a second carrier-width table.
- Implement `AlignmentGuarantee::after_offset(u64)` with checked power-of-two arithmetic. Offset zero preserves the base guarantee; a nonzero offset returns the greatest power-of-two guarantee common to the base and offset.
- Make program values and allocations use the correct role types. Derive each `ViewRef`'s effective guarantee and make the natural kernel-buffer access check consume it before a stage is verified.
- Preserve old valid encoding bytes. Reject a previously admitted misaligned partial view as a verifier bug fix rather than reinterpreting its bytes under a compatibility path.

## Required evidence

- Exhaust the storage-carrier population and prove each natural alignment is representable.
- Perturb zero, 3, 4, 8, and the largest representable power of two; show typed construction and one-directional satisfaction failures.
- For a 16-byte base, prove offsets 0, 4, 8, 16, and 20 derive 16, 4, 8, 16, and 4 respectively. Perturb the offset subject rather than the assertion.
- Drive one partial view through `KernelProgramBuilder`; a naturally aligned view succeeds and a one-byte-shifted F32 view fails before artifact construction.
- Pin every pre-existing valid kernel-program and artifact identity unchanged.

## Non-goals

Selected-provider access requirements, target realization facts, runtime pointer observation, vector KIR, allocator implementation, or a new artifact field.

## Closes when

The shared role-safe vocabulary is the only raw-alignment authority across IR/compiler/artifact APIs, effective view alignment is verified, and old valid identities remain byte-identical.

Coordinator `make full` on `90112675`: 3493 passed. Merged to `main` at `bcbf9f9bd975890157353a88282ec0edd2db418c`. The Rust spelling is a labelled draft; Tom's packet is [`accept-the-typed-byte-alignment-surface`](accept-the-typed-byte-alignment-surface.md).
