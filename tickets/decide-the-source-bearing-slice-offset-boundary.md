---
id: decide-the-source-bearing-slice-offset-boundary
title: Decide the source-bearing slice offset boundary
status: done
priority: p2
dependencies: [admit-the-sub-tensor-selection-family, accept-the-symbolic-index-coefficient-surface]
related: [admit-a-position-selecting-slice-for-the-rotary-table, correct-the-symbolic-coefficient-era-index-vocabulary-claims, admit-source-bearing-slice-selection-semantics, preserve-source-bearing-slice-offsets-through-index-refinement, admit-live-extent-operands-to-payload-indexing, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, slice, symbolic-extents, public-boundary]
---
## Decision requested

Choose the exact public and identity-bearing representation by which a bound source reaches a `Slice` window offset: a source-bearing attribute field, an operation operand, or another explicitly validated form. The accepted answer must also state the bounds and result-shape rule, the refusal boundary when the source is unavailable or unproved, and every semantic-program, registry, compiler, artifact, and cache identity consequence.

## Fact audit at `901b7fca286af58f1be7aecf6cfaf4240af8ec93`

- **Verified — the index-expression trigger has fired.** `crates/tiler-ir/src/index/model.rs`, anchors `pub(super) struct LinearTermData` and `LinearCombination {`, stores `SourcedIndexInteger` coefficients. A bound symbol can therefore reach the coordinate `t + C` as the term `C * 1`; no new `IndexNode` variant is required for that expression.
- **Verified — the semantic selection still cannot carry the source.** `crates/tiler-ir/src/semantic/slice.rs`, anchors `pub enum SliceAxisSelection` and `offset: u64`, exposes only a literal window offset. `decode_axis`, anchor `(SLICE_RELATION_SYMBOLIC_WINDOW, _)`, refuses the reserved relation before decoding relation-specific fields.
- **Verified — this is not only a spelling choice.** The same file, anchors `SLICE_SELECTION_DOMAIN`, `encode_selection`, `SLICE_F32_NORMATIVE_DEFINITION`, and `register_standard_slice`, makes the admitted grammar part of canonical selection bytes, operation-definition identity, and standard-registry identity. `SliceF32::infer`, anchor `request.static_operand_shape(0)`, also has only a static bounds/result rule today.
- **Verified — the first named consumer is blocked here.** [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) needs rows `C … C + T` derived from the same bound cursor that fixes the cache. The literal family is already delivered; this boundary, not the index vocabulary, is its remaining symbolic-offset prerequisite.

## Alternatives that must be compared

1. Add a source-bearing offset to the canonical axis relation. This keeps the selection self-contained but changes the public selection vocabulary and its canonical encoding.
2. Add an explicit offset operand or binding. This follows the taxonomy's named dynamic-form candidate, but changes operation arity, authoring, validation, result inference, and occurrence identity.
3. Keep the literal-only family and require the rotary consumer to express the position through a different already-governed composition. This is admissible only if the composition makes inconsistent cache and rotary positions unrepresentable rather than moving the same host convention elsewhere.

For each alternative, state how a source is bound, how non-negativity and `offset + extent <= axis_extent` are proved, whether a sourced result extent remains source-derived or must be resolved, how reference evaluation obtains the value, and which identity domains step or provably hold. Do not infer the answer from the currently reserved `symbolic-window` name; the source comment explicitly reserves a name, not a design.

## Non-goals

No implementation, tag assignment, identity-domain step, or rotary-program construction belongs in this decision. Do not reopen the already accepted symbolic index coefficient surface, and do not conflate the separate strided-window question with the source-bearing offset.

## Accepted — 2026-08-12

**Decision.** Tom accepted the revised source-bearing attribute boundary in the Codex coordination thread by replying `sounds good, accept`. The relay source is Tom's direct response to the coordinator's source-first decision packet. This ticket records that decision and releases the implementation carriers below; it does not claim their implementation or evidence.

`SliceAxisSelection::Window` carries its offset through the existing shape-owned `SourcedExtent` vocabulary rather than through a second Slice-specific scalar:

```rust
SliceAxisSelection::Window {
    offset: SourcedExtent,
    extent: Extent,
}
```

The exact public spelling may gain convenience constructors, but the semantic product is fixed: an axis is exactly `Whole` or `Window`, and a window offset is exactly `Static` or `Symbol`. `ShapeEnv` remains the sole authority that binds a symbol to `Static`, `InputDimension`, `InterfaceParameter`, or `TargetProperty` provenance. There is no optional offset, zero sentinel, inferred axis, caller-provided duplicate scalar, or separate cursor carrier. The reserved `symbolic-window` name fixes no design and must not become a second semantic variant for the same relation.

Map construction is shape-independent. Applying the selection to a program is fallible against that program's exact `ShapeEnv` and sourced operand shape. A static or sourced window is admitted only when the environment proves, with checked unsigned arithmetic, `offset + extent <= available_axis` for every admitted model and proves the family’s existing proper-window rule. `SourcedExtent` supplies non-negativity. The explicit window extent remains the result extent on that axis; untouched axes preserve their sourced extents. A foreign, undeclared, unavailable-at-phase, overflowing, or unproved source refuses by typed cause. No clamp, wrap, default, host convention, or post-commit fallback is admitted.

Runtime-bound relations used by that proof are evaluated against their authoritative invocation bindings before `RoutingCommit`, through the separately accepted retained-shape-environment evaluator. Reference evaluation derives the same bindings from the program environment and declared input facts. A reference callback never receives a second cursor value. The first implementation may support only binding kinds for which that evaluator has an authenticated value source; every other kind remains a named refusal.

The semantic source must remain whole through refinement. The Slice law and compiler lowering carry the exact environment, construct source-aware index regions, and spell `t + C` through the accepted sourced coefficient/addend vocabulary. Syntax alone is not a proof: total index access remains refused until the verifier can discharge the symbolic bound from the same retained environment or a compiler-minted checked proof derived from it. Payload execution then consumes the live extent through the separately owned live-extent operand and artifact/runtime boundary; neither lowering nor a backend may rebind or specialize the cursor independently.

**Identity rule.** Existing literal selection bytes and their meaning remain byte-identical. A symbolic offset gets a fresh, injective field spelling under the existing length framing, so `tiler.slice-selection.v1` and `tiler::slice-f32@1` need not step merely because a new value becomes representable. Literal and symbolic offsets remain distinct even when they resolve to the same number because their provenance differs. The operation definition, standard semantic registry, Slice law registration, reference capability, and physical-provider revisions must each be audited and advanced where their admitted behavior changes. Registry snapshots, request evidence, and downstream cache subjects may therefore move transitively. No artifact or manifest schema step is authorized by this decision alone; the existing retained-environment and live-extent delivery tickets own those schema boundaries.

**Rejected alternatives.** An extra tensor operand is reserved for a future genuinely data-dependent Slice whose offset comes from tensor element data; for this metadata-bound cursor it adds dtype, storage, lifetime, arity, and equality obligations while creating a second authority that can disagree with the cache cursor. A separate interface key duplicates `ShapeEnv`. Generated gather indices, literal specialization, and physical buffer offsets either retain the same host convention, add work, or fail to define semantic/reference meaning. Clamping, inference from equal extents, and defaults are silent wrong-result paths and remain forbidden.

**Strongest counterpoint retained.** The attribute design requires source-aware semantic, reference, refinement, verifier, payload, and runtime plumbing. That plumbing is already required to use one authenticated cursor for shapes, cache indexing, and payload execution; an operand does not remove it and adds an inconsistent second spelling. Evidence that would reverse this decision is a real operation whose offset originates in tensor element data and is not semantically equal to a `ShapeEnv` root. That evidence would justify a distinct data-dependent Slice form, not a weakening of this one.

**Fact repairs made by the decision.** The semantic grammar is the first remaining blocker, not the only one. Current Slice law and compiler contexts do not preserve the source environment, the index verifier does not yet discharge symbolic coefficient bounds, reference evaluation has no authenticated source resolver, and payloads do not yet consume live extents. Implementation is therefore split across the linked semantic, index-refinement, retained-relation, and live-payload carriers before the rotary consumer can close.

## Closes when

Tom accepts one exact included and excluded public boundary; its validation, inference, reference, lowering, and identity consequences are mapped; the rotary consumer and any implementation carrier are updated to depend on that answer; and rejected alternatives retain the reason they were rejected.
