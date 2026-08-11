---
id: lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability
title: Lower the sub-tensor selection occurrence through one index-access capability
status: done
priority: p2
dependencies: [admit-the-sub-tensor-selection-family]
related: [scope-the-sub-tensor-selection-fusion-role, admit-a-fusion-role-for-the-sub-tensor-selection-slice, lower-a-two-region-occurrence-through-one-index-access-capability, lower-the-concatenate-occurrence-through-partitioned-writes, admit-the-structural-families-into-the-scheduled-region-vocabulary, decide-the-source-bearing-slice-offset-boundary, admit-an-index-realization-law-for-the-literal-offset-slice]
scopes: [implementation/compiler, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, lowering, indexing, slice]
---
## User-visible outcome

`tiler::slice-f32@1` resolves an index-access lowering capability like the two structural families beside it, and the operation-family delivery graph's track **O-06** points at that owner instead of carrying a bare M5 cell that reads *owed* with no ticket link in that document.

## Why this exists

**Fact — the delivery graph's O-06 M5 cell is still bare *owed* and its owners section names no lowering ticket.** The [operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md)'s physical-rung table gives track O-06 an M5 cell reading *owed*. Its owners section says only `live for the literal-offset form` and points to neither a capability ticket nor a lowering provider. The [support-matrix](../docs/roadmap.md#operation-family-support-matrix) Sub-tensor selection row already names this ticket as the index-access lowering between R5 and R6, so the corpus is not ownerless — the *delivery graph* specifically still lacks a followable M5 owner. This ticket is that delivery-graph owner once the capability lands; the family-admission ticket and the two reserved relations are separate work. Adjacent corpus lag on the same track (out of this ticket's close condition): the delivery graph's O-06 M4 cell still reads *owed* while the matrix is R5 and [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md) is done.

**Source-first correction — 2026-08-11, at `099c6e2d`.** The first provider execution exposed a premise this ticket omitted: `refine_index_region` refuses the emitted slice before comparing it with an expected realization, as `IrVerifier(MissingRealizationLaw)`. The delivery graph's M5 definition says a capability must emit a region, but its worked softmax example also classifies a missing `IndexRealizationLaw` as an M5 fact. Therefore this ticket delivers the narrower capability half of M5 — exact resolution and an emitted, structurally verified region — and must not mark the whole cell delivered. [`admit-an-index-realization-law-for-the-literal-offset-slice`](admit-an-index-realization-law-for-the-literal-offset-slice.md) owns the identity-bearing remainder. That split narrows the maturity wording without changing this ticket's implementation subject: adding a law would widen the closed `IndexRealizationLaw` vocabulary, its encoding, and the law-registry identity, so it is outside this Terra-bounded carrier rather than worked around.

**Source-first correction — 2026-08-11, identity consequence of this capability row.** The narrower compiler change is not identity-neutral. `request_subject` retains `authorities.installed.registry_identity()`, and `canonical_explain_subject_bytes` length-frames that complete lowering-registry identity. Adding the governed slice row therefore changes the canonical governed lowering-capability registry identity and the derived explain request qualifier even for the existing unrelated multiply fixture: `deterministic_trace_is_sealed_and_rendered_separately` moved from `request=7ba3d77a66f04638` to `request=c4d76aa0d4fbe72e`, while both rendered event lines stayed byte-for-byte unchanged. The test first failed with the new value on the left and the old pin on the right; re-running the isolated test reproduced the same value before rebaselining it. This is an appended governed capability population, not an identity-schema change: this ticket changes no capability identity encoder, schema version, domain tag, or `IndexRealizationLaw`/law-registry byte. The residual law ticket owns that separate future law-registry identity change. The identity-retaining request consumer and pin move require the strongest independent review even though the encoder itself is unchanged.

**Fact — the literal-offset lowering is not a fork, on the evidence below, which is why this is an implementation ticket rather than a second scoping record.** [Sub-tensor selection fusion role](../docs/research/indexing/sub-tensor-selection-fusion-role.md) established that M5 and M4 do not block each other. Three further facts, re-read at this dispatch's exact base `099c6e2d`, make the literal shape settled:

- `governed_index_access_capabilities` registers the reindex and the broadcast with the unary F32 signature and an empty `emitted` set. Its source-safe anchor is `scalar operation at all: the value written is the value read`: a selection likewise writes the value it reads and needs **one** capability rather than the per-arity registrations the concatenate's variadic key forces.
- The literal read coordinate is `t + offset` per restricted axis. The semantic module's current anchor `The refusal stands at this layer's selection boundary` establishes the important distinction: `SourcedIndexInteger` now makes `t + C` expressible in the index language, but `SliceAxisSelection::Window` still stores `offset: u64` and `decode_axis` refuses `symbolic-window`. This ticket asks the coordinate-expression language only for the already-supported literal form. [`decide-the-source-bearing-slice-offset-boundary`](decide-the-source-bearing-slice-offset-boundary.md) owns the separate semantic choice between a source-bearing attribute and an operand.
- The write is one root over the whole result and is total over it by construction, so none of the four sites that refuse a *partitioned* write — the ones [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) inventoried — is reached. This family needs no write-ownership widening, which is the entire content of the concatenate's fork.

## What the work is

Register one `GovernedIndexAccess` for `slice_f32_op()` with the unary `f32` signature and an empty `emitted` set, beside the reindex and broadcast entries, and implement its `IndexAccessLoweringProvider` so that a `SliceAxisSelection::WholeAxis` entry emits the bare dimension and a `SliceAxisSelection::Window { offset, .. }` entry emits that dimension plus the literal offset. `SliceAxisSelection` is deliberately not `#[non_exhaustive]` under the source anchor `Deliberately **not** \`#[non_exhaustive]\`` precisely so a third admitted relation is a build error here rather than a silent fall-through; keep the match total and add no wildcard arm.

Prove the lowering can say no. The perturbation that matters is a dropped offset: a selection whose window starts at a nonzero coordinate must produce a region that differs from the one an offset-dropping implementation produces, and the difference must be observed rather than asserted. `crates/tiler-reference/tests/slice_conformance.rs` retains exactly that perturbation at the reference layer and is the model for the shape of the evidence.

## Explicit non-goals

- Any fusion role. That is [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md), and neither ticket waits on the other.
- Lifting the request boundary or reaching a `VerifiedKernel`. Reindex and broadcast capabilities now resolve and compile through ordinary paths after [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) admitted `LogicalAccess::ReindexBijection` and `LogicalAccess::BroadcastReplication`. Slice still has no selection/window `LogicalAccess` map, so a program stating only a slice is refused under `operation-set` for that missing spelling — a residual that ticket closed for reindex/broadcast only and that **no live ticket currently owns** for slice (concatenate's parallel gap is [`admit-the-concatenate-family-into-the-scheduled-region-vocabulary`](admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md); slice has no counterpart). This capability lands without clearing that wall; admitting a slice scheduled-region map is separate work.
- The strided and symbolic relations, which the key refuses by name. Symbolic selection is no longer blocked by index-expression vocabulary; it remains blocked by the semantic selection grammar and is owned by the source-bearing-offset decision.
- A view-versus-copy physical realization. That is M6 and a physical-candidate applicability question, not this capability's.
- Any `IndexRealizationLaw`, semantic-refinement claim, or compile-feasibility claim. The missing law is the remaining M5 owner above; this ticket drives the resolved provider directly through the host-owned emission context and the independent region oracle, and does not treat that structural evidence as refinement.

## Stop conditions

Two discoveries end this dispatch instead of being pushed through, because either would mean the shape above is wrong rather than merely harder:

- The write domain or the coordinate construction needs anything the reindex and broadcast lowerings do not already have — in particular any widening of `IndexNode`, of `AccessData`, or of the write-ownership contract. That would make this a fork like O-07's, and the correct outcome is a scoping record and correctly-ordered tickets rather than a partial capability.
- The exact-signature resolver cannot distinguish this capability from the reindex's or the broadcast's. The three share an operand and result type list, so the distinction rests entirely on the operation key inside the resolution triple; if it does not, that is an identity defect to file rather than to work around.

## Closes when

The capability is registered and resolves for a slice occurrence, its emitted set is empty and justified in a comment rather than by citation, the relation match is total with no wildcard, the resolved real provider emits one structurally verified region whose offset-dropping perturbation is shown to fail on the independent oracle, the derived explain-request pin is rebaselined for the changed governed lowering-registry population without moving an identity schema/domain/tag or any law byte, and the delivery graph's O-06 M5 cell distinguishes this delivered capability half from the separately owned missing realization law. This close condition claims no refinement or compile feasibility.

## Graph maintenance

- `contracts/navigation` is deliberately **not** declared. This capability moves no support-matrix rung: the matrix's ladder puts the index-access lowering between R5 and R6, and the `Sub-tensor selection` row's text about it is the fusion-role ticket's to update when R5 lands.
- The delivery graph cell this ticket answers lives in `research/semantic-graph`. That scope is now declared on this ticket so the same carrier can replace the unowned *owed* cell with the exact delivered capability half and the followable owner of the still-owed law; do not collapse the two M5 facts.
