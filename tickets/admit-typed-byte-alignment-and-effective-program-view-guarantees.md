---
id: admit-typed-byte-alignment-and-effective-program-view-guarantees
title: Admit typed byte alignment and effective program-view guarantees
status: todo
priority: p1
dependencies: [separate-vector-operand-alignment-from-target-realization]
related: [derive-boundary-alignment-from-the-element-type, carry-the-byte-offset-of-a-partial-binding-view]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [alignment, placement, ir, public-boundary, correctness]
---
## User-visible outcome

Every alignment value is checked once, requirements cannot be confused with guarantees, and a partial view advertises only the alignment its actual byte offset preserves.

## Facts at filing base `f199b26376612e4b39c35569b084dda4c67490ce`

- **Verified.** Compiler `ByteAlignment` already enforces positive powers of two and the correct divisibility relation, but it is crate-private while IR program and artifact surfaces carry raw `u32` alignments.
- **Verified.** `AllocationSpec` calls its alignment a guarantee and `MaterializedValueSpec` calls its alignment a requirement, but their identical raw type permits reversed comparisons and repeated validation.
- **Verified.** `push_view` accepts a `ByteWindow` without deriving the alignment of `base + offset`; `check_stage_accesses` checks extent, role, component, type, and access mode but no address alignment.
- **Verified.** Existing identity encoders already carry the fixed-width alignment values and view offsets, so a pure typed replacement and derived verifier can preserve every old valid canonical byte.

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
