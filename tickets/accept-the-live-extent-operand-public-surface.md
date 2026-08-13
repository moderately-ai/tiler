---
id: accept-the-live-extent-operand-public-surface
title: Accept the live-extent operand public surface
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-live-extent-operands-to-payload-indexing]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public/schema surface of the live input-extent operand draft, so the labelled items stop being a draft and the remainder tickets can carry an accepted spelling through the artifact envelope.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes every concrete public surface to Tom. [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) produced a tested labelled draft at `9a8f53c937dc9b9f777a1d4b361cadc1a0b0316e` on `tkt/admit-live-extent-operands-to-payload-indexing` and correctly refused to invent an unaccepted artifact-envelope feature. This node is not implementation work. Only Tom closes it.

The parent stays in `review` with that branch preserved. Do not merge `9a8f53c9` as a complete outcome, and do not treat this ticket as authority to land the draft before Tom answers.

## The surface, as drafted at `9a8f53c9`

Re-read this session at that commit: `InputExtentParameter`, `LoopBound`, `OperationView::InputExtent`, `KernelBuilder::{declare_input_extent, input_extent}`, `KernelInputExtentId`, `VerifiedInputExtentId`, `MAX_KERNEL_INPUT_EXTENTS`, `LogicalAccess::LiveRowMajor`, `ReductionTopology::LiveContraction`, `RoutedExtentParameter`, Metal `constant ulong& e{ordinal} [[buffer({index})]]`, and the private compiler `HostAbi` construction that pushes existing `AbiRoot::InputExtent` for symbolic axes. Reproduce: `rg -n "Draft surface, not yet accepted" crates/tiler-ir/src/kernel crates/tiler-ir/src/schedule crates/tiler-runtime/src/load/route.rs`.

**Included — structured kernel.** `tiler_ir::kernel::InputExtentParameter { tensor, axis }` with `tensor` required to be `TensorRole::Input`; `KernelBuilder::declare_input_extent` / `input_extent`; `KernelInputExtentId` and `VerifiedInputExtentId` as a separate handle space from buffers and staging; `MAX_KERNEL_INPUT_EXTENTS = 16`; `OperationView::InputExtent { parameter }` on the existing `#[non_exhaustive]` view enum; `LoopBound::{Literal(u64), Value(VerifiedValueId)}` so a live extent can be a trip count without a second builder path that could bake the same range; `VerifiedKernel::{input_extents, declared_input_extents, input_extent}`.

**Included — scheduled region.** `LogicalAccess::LiveRowMajor { inner_axis }` and `ReductionTopology::LiveContraction { live_input, live_axis, order, permits_reassociation, permits_permutation }`. Both are tagged draft. The iteration domain / free indices stay static; only the named inner or contracted extent is live. The live value is excluded from schedule identity.

**Included — Metal emission.** After the buffer table, each declared extent is `constant ulong& e{ordinal} [[buffer({index})]]` with `index = buffer_count + ordinal`. Builtins stay after extents. Empty extent lists emit nothing.

**Included — routed runtime.** `tiler_runtime::load::route::RoutedExtentParameter` with `transport_slot`, `value`, and `parameter_bytes` as little-endian `u64`. Frozen before `Preflight::commit`. The committed authority owns the bytes; a backend binds the declared transport and does not re-evaluate the fact.

**Included — compiler construction, not a new public type.** Private `HostAbi` already pushed `AbiRoot::UnsignedLiteral` for static axes. Symbolic axes now push the existing public `AbiRoot::InputExtent { key, axis }` root. `HostAbi` itself stays crate-private. `AbiRoot::InputExtent` is not new.

**Identity claim on the draft.** `KERNEL_DOMAIN` stays `tiler.kernel.v7`. Empty extent lists write nothing, so previously encodable static kernels keep their bytes. A nonempty list is appended; the live value is excluded from kernel, artifact, payload, library, and pipeline identity.

**Excluded, each by a stated reason rather than by omission.** Artifact construction, codec, decode, and validation of an operand row — the worker refused to invent an unaccepted envelope feature; that remainder is [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) and will need its own surface packet if the accepted kernel/runtime spelling forces a new envelope field. Adapter binding methods beyond `RoutedExtentParameter`. Dynamic target properties, physical layout facts, arbitrary caller scalars, unbounded loops, ragged per-row extents, negative values, and cursor/capacity specialization. `compile()` of a symbolic program — first refuse remains strategy selection; see [`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md). The working draft path is `ScheduledRegionBuilder` + `lower_scheduled_region`. One artifact + payload + pipeline at `N = 14` and `N = 15`. Schedule-verified `LiveContraction` end-to-end.

## The questions that are genuinely Tom's

1. **Accept the kernel operand as drafted?** `InputExtentParameter` names a scheduled input axis and is the structured-kernel spelling of the existing `AbiRoot::InputExtent` root. The alternative is a buffer-shaped scalar or a caller-supplied second list, both of which invent a second authority over the same fact.
2. **Accept `LoopBound::Value` on the public loop spec?** It is how a live extent becomes a trip count without a parallel builder that could bake the same range. The alternative is a dedicated live-loop constructor, which would reintroduce the implicit-static choice the parent ticket forbids.
3. **Accept `LiveRowMajor` and `LiveContraction` now, or only after their E2E evidence exists?** They are the scheduled-region carriers the parent required. They have no schedule-verified end-to-end evidence on `9a8f53c9`. Accepting them now lets remainder tickets consume an accepted spelling; waiting keeps them labelled until [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md) lands.
4. **Accept the Metal `eN` buffer-table ABI and `RoutedExtentParameter` freeze?** Extents occupy the next buffer indices after the tensor table. A named constant-buffer namespace or a bytes binding distinct from `[[buffer(N)]]` would be a different backend ABI.

## Recommendation

Accept questions 1, 2, and 4 as drafted. Accept question 3 as drafted as well: the variants are the only scheduled-region spelling that keeps the live value out of identity, and the E2E remainder cannot proceed against a moving unaccepted shape. **Strongest counterpoint:** accepting `LiveRowMajor` / `LiveContraction` before E2E freezes a schedule vocabulary that the missing contraction evidence might still force to change. Tom should answer the four questions independently.

Do not treat acceptance as authority to invent the artifact-envelope row. That row is a later public/schema surface.

## Options eliminated before ranking

Inventing a second caller-supplied scalar list, baking the live value into kernel or artifact identity, or self-accepting the draft, can silently give one `S` two meanings or release dependents against an unaccepted boundary. Those are defects, not candidates.

## Closes when

Tom accepts, accepts with named exclusions, or revises. The implementing agent then records provenance, applies every consequence, and only then may the artifact-envelope and E2E remainder tickets proceed against the accepted spelling. Nothing on this node merges `9a8f53c9`.

## Graph maintenance

- Only Tom approves or revises.
- [`carry-live-extent-operands-through-the-artifact-envelope`](carry-live-extent-operands-through-the-artifact-envelope.md) and [`prove-a-schedule-verified-live-contraction-consumes-s`](prove-a-schedule-verified-live-contraction-consumes-s.md) depend on this node, so they stay off the ready board until acceptance.
- The parent remains `review` until the accepted draft is integrated with the remainders. Do not set the parent `done` on this packet alone.
