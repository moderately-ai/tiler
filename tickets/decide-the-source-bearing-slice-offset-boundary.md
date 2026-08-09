---
id: decide-the-source-bearing-slice-offset-boundary
title: Decide the source-bearing slice offset boundary
status: awaiting-decision
priority: p2
dependencies: [admit-the-sub-tensor-selection-family, accept-the-symbolic-index-coefficient-surface]
related: [admit-a-position-selecting-slice-for-the-rotary-table, correct-the-symbolic-coefficient-era-index-vocabulary-claims]
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

## Closes when

Tom accepts one exact included and excluded public boundary; its validation, inference, reference, lowering, and identity consequences are mapped; the rotary consumer and any implementation carrier are updated to depend on that answer; and rejected alternatives retain the reason they were rejected.
