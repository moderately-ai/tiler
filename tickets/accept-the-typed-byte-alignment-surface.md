---
id: accept-the-typed-byte-alignment-surface
title: Accept the typed byte-alignment surface
status: done
priority: p1
dependencies: []
related: [admit-typed-byte-alignment-and-effective-program-view-guarantees]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft Rust spelling of the role-safe alignment vocabulary he accepted as a model on 2026-08-12.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public types to Tom. The 2026-08-12 packet on [`admit-typed-byte-alignment-and-effective-program-view-guarantees`](admit-typed-byte-alignment-and-effective-program-view-guarantees.md) accepted the *model*. This node is the spelling landed at `90112675f7def768d76a3153c7dff324be1ff502`. Only Tom closes it.

## The surface, as landed at `90112675`

**Included.**

- `tiler_ir::program::{ByteAlignment, ByteAlignmentError, AlignmentRequirement, AlignmentGuarantee}`, re-exported from `tiler_artifact::program`.
- Fallible `new`, `natural_for` from `StorageScalar::byte_width`, `bytes`, `from_alignment`, `alignment`.
- `AlignmentGuarantee::satisfies(AlignmentRequirement)` and `after_offset(u64)`.
- Value specs carry `AlignmentRequirement`; allocations carry `AlignmentGuarantee`; `ViewRef::alignment()` is the allocation guarantee after the window offset.
- Artifact `BindingRef` / `DecodedBinding` return `AlignmentRequirement`.
- `KernelProgramBuildError::StageAccessAlignment` and `ArtifactCodecError::InvalidAlignment`.

**Excluded.**

- `Default`, `From<u32>`, unchecked constructors. Public requirement ↔ guarantee conversion. `satisfies` on any type except `AlignmentGuarantee`. Rounding, clamp, integer sentinel. `after_offset` on a requirement. Selected-provider access requirements, target facts, runtime pointer observation, vector KIR, allocator implementation. A new artifact field or identity-domain step.

## Recommendation

Accept as drafted. The spelling follows the accepted model: one checked quantity, opaque roles, comparison only on the guarantee, effective view alignment from the actual offset, valid encodings still four big-endian bytes of the same quantity. **Strongest counterpoint:** promoting compiler-private `ByteAlignment` to a public IR type, plus two new public role wrappers, is a larger public surface than the model packet named.

## Accepted — 2026-08-13

**Tom accepted the exact surface as drafted**, with no named exclusion, in the live coordination session. The included set is `ByteAlignment`, `ByteAlignmentError`, `AlignmentRequirement`, `AlignmentGuarantee`, fallible `new`, `natural_for`, `bytes`, `from_alignment`, `alignment`, `AlignmentGuarantee::satisfies` / `after_offset`, value-spec requirements, allocation guarantees, `ViewRef::alignment()` as the allocation guarantee after the window offset, artifact `BindingRef` / `DecodedBinding` returning `AlignmentRequirement`, `KernelProgramBuildError::StageAccessAlignment`, and `ArtifactCodecError::InvalidAlignment`.

The 2026-08-12 model acceptance remains the earlier packet. This node is the included/excluded Rust spelling at `90112675`. In-code labels flip from labelled draft to accepted public surface.

## Closes when

Tom accepts, accepts with named exclusions, or revises.
