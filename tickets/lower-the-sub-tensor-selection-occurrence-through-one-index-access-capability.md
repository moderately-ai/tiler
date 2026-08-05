---
id: lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability
title: Lower the sub-tensor selection occurrence through one index-access capability
status: todo
priority: p2
dependencies: []
related: [scope-the-sub-tensor-selection-fusion-role, admit-a-fusion-role-for-the-sub-tensor-selection-slice, lower-a-two-region-occurrence-through-one-index-access-capability, lower-the-concatenate-occurrence-through-partitioned-writes, admit-the-structural-families-into-the-scheduled-region-vocabulary, admit-the-sub-tensor-selection-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, lowering, indexing, slice]
---
## User-visible outcome

`tiler::slice-f32@1` resolves an index-access lowering capability like the two structural families beside it, so the operation-family delivery graph's track **O-06** stops carrying an M5 cell that reads *owed* with no owner named anywhere in the corpus.

## Why this exists

**Fact — O-06's M5 is owed and unowned.** The [operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md)'s physical-rung table gives track O-06 an M5 cell reading *owed*, and its owners section lists O-06 among the tracks with exact owners already — but the owner it names covers the family's semantic admission and its two reserved relations, not its lowering. Unlike O-07, no ticket held this cell.

**Fact — it is not a fork, on the evidence below, which is why this is an implementation ticket rather than a second scoping record.** [Sub-tensor selection fusion role](../docs/research/indexing/sub-tensor-selection-fusion-role.md) established that M5 and M4 do not block each other. Three further facts, read at `3cca2a3f`, are what make the shape look settled:

- `governed_index_access_capabilities` (`crates/tiler-compiler/src/governed.rs:222-337`) registers the reindex and the broadcast with `LoweringSignature::new([f32], [f32])` and a deliberately empty `emitted` set, because "a reindex applies no scalar operation at all: the value written is the value read" (`:294-300`). A selection has exactly that property and exactly that signature (`crates/tiler-ir/src/semantic/slice.rs:820-821`, `:948-957`), so it needs **one** capability rather than the seven per-arity registrations the concatenate's variadic key forces.
- The read coordinate is `t + offset` per restricted axis for a literal offset, and the family's own module states that this is expressible today while a symbolic offset is not (`slice.rs:74-89`). So the coordinate-expression language is asked for nothing.
- The write is one root over the whole result and is total over it by construction, so none of the four sites that refuse a *partitioned* write — the ones [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) inventoried — is reached. This family needs no write-ownership widening, which is the entire content of the concatenate's fork.

## What the work is

Register one `GovernedIndexAccess` for `slice_f32_op()` with the unary `f32` signature and an empty `emitted` set, beside the reindex and broadcast entries, and implement its `IndexAccessLoweringProvider` so that a `SliceAxisSelection::WholeAxis` entry emits the bare dimension and a `SliceAxisSelection::Window { offset, .. }` entry emits that dimension plus the literal offset. `SliceAxisSelection` is deliberately not `#[non_exhaustive]` (`slice.rs:222-239`) precisely so a third admitted relation is a build error here rather than a silent fall-through; keep the match total and add no wildcard arm.

Prove the lowering can say no. The perturbation that matters is a dropped offset: a selection whose window starts at a nonzero coordinate must produce a region that differs from the one an offset-dropping implementation produces, and the difference must be observed rather than asserted. `crates/tiler-reference/tests/slice_conformance.rs` retains exactly that perturbation at the reference layer and is the model for the shape of the evidence.

## Explicit non-goals

- Any fusion role. That is [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md), and neither ticket waits on the other.
- Lifting the request boundary or reaching a `VerifiedKernel`. The reindex and broadcast capabilities are registered and, in the delivery graph's own words, "delivered and never resolved, because no admitted program shape contains one"; this capability lands in the same state, and [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) owns the difference.
- The strided and symbolic relations, which the key refuses by name.
- A view-versus-copy physical realization. That is M6 and a physical-candidate applicability question, not this capability's.

## Stop conditions

Two discoveries end this dispatch instead of being pushed through, because either would mean the shape above is wrong rather than merely harder:

- The write domain or the coordinate construction needs anything the reindex and broadcast lowerings do not already have — in particular any widening of `IndexNode`, of `AccessData`, or of the write-ownership contract. That would make this a fork like O-07's, and the correct outcome is a scoping record and correctly-ordered tickets rather than a partial capability.
- The exact-signature resolver cannot distinguish this capability from the reindex's or the broadcast's. The three share an operand and result type list, so the distinction rests entirely on the operation key inside the resolution triple; if it does not, that is an identity defect to file rather than to work around.

## Closes when

The capability is registered and resolves for a slice occurrence, its emitted set is empty and justified in a comment rather than by citation, the relation match is total with no wildcard, the offset-dropping perturbation is shown to fail, and the delivery graph's O-06 M5 cell has an owner a reader can follow.

## Graph maintenance

- `contracts/navigation` is deliberately **not** declared. This capability moves no support-matrix rung: the matrix's ladder puts the index-access lowering between R5 and R6, and the `Sub-tensor selection` row's text about it is the fusion-role ticket's to update when R5 lands.
- The delivery graph cell this ticket answers lives in `research/semantic-graph`, which is a different owner's scope. Report the cell's new owner at integration rather than editing that table from here.
