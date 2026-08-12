---
id: admit-a-position-selecting-slice-for-the-rotary-table
title: Derive the decode step's rotary position through the sub-tensor selection family
status: todo
priority: p2
dependencies: [integrate-the-autoregressive-decode-loop, reclassify-language-model-work-as-a-conformance-track, admit-the-sub-tensor-selection-family, admit-source-bearing-slice-selection-semantics, preserve-source-bearing-slice-offsets-through-index-refinement, admit-live-extent-operands-to-payload-indexing, evaluate-retained-shape-relations-before-routing-commit]
related: [design-autoregressive-state-and-kv-cache, admit-the-reindex-and-broadcast-operation-families, compose-rotary-position-embedding-from-reindex-and-broadcast]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, operation-families, rope, kv-cache, language-model, class-conformance-fixture]
---
## User-visible outcome

The absolute position of a decode step's new tokens becomes a checked index expression over a bound extent instead of a host convention — so a caller that states position inconsistently is refused rather than silently returning a wrong token.

## Why this is a correctness trigger and not a bytes trigger

The sub-tensor-selection row's existing trigger is a prefill pass needing only the final position's logits, which is 4,978,634,752 bytes against 607,744. [Rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) supplies a second and stronger one: at batch 1, **absolute position enters the decode program through `cos` and `sin` alone**. The mask is derivable from `T` and `S`, the cache extents from `C`, and the residual stream carries no position — so the rotary rows are the one input whose correct value depends on where the token sits, and a wrong row is a `[1, 128]` F32 tensor with the same shape, dtype, accessible range, and launch geometry as the right one. Every layer accepts it.

Binding the whole `[max_positions, 128]` table and selecting rows `C … C + T` by an index expression over the same bound extent that fixes the cache moves that from a convention into a coordinate map.

## The claim's exact limit, stated so it is not overread

A slice removes the **inconsistency** mode — the cache saying `C = 14` while the rotary rows say 0 — and does not remove the **wrong-cursor** mode, where a consistently wrong `C` produces a consistently wrong program that only the conformance oracle detects. Two different failures; this buys one of them.

## Required design

**Split on 2026-08-04 under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md), corrected 2026-08-09.** This section previously carried the family's design as well as its trigger, which made a generic operation family reachable only behind a complete consumer decode loop. [`admit-the-sub-tensor-selection-family`](admit-the-sub-tensor-selection-family.md) has since landed the literal-offset form. Its former index-vocabulary premise is retired: `crates/tiler-ir/src/index/model.rs`, anchor `pub(super) struct LinearTermData`, now carries `SourcedIndexInteger` coefficients, so `t + C` is expressible as `C * 1`. The remaining gap is the semantic selection grammar: `crates/tiler-ir/src/semantic/slice.rs`, anchors `pub enum SliceAxisSelection`, `offset: u64`, and `(SLICE_RELATION_SYMBOLIC_WINDOW, _)`, carries only a literal offset and refuses the reserved symbolic relation before decoding its fields.

What stays here is the consumer application: **binding the whole `[max_positions, 128]` table and selecting rows `C … C + T` from the same bound source that fixes the cache**, so the decode program cannot state position two ways. `C` is a bound symbol, so this application needs a source-bearing Slice selection rather than the delivered literal one. [`decide-the-source-bearing-slice-offset-boundary`](decide-the-source-bearing-slice-offset-boundary.md) accepted `SourcedExtent` in the canonical window relation with `ShapeEnv` as the sole binding authority. The semantic/reference and index-refinement carriers implement that boundary; the retained-relation and live-extent carriers make its runtime proof and payload use executable. This ticket consumes those answers rather than restating or weakening them.

## Closes when

The literal family has landed; the source-bearing boundary is accepted and implemented; the conformance decode program derives its rotary rows through it over the same bound source that fixes the cache; an inconsistent position is refused rather than executed; and the support-matrix row keeps both triggers with this one's evidence attached. This ticket cannot close on the literal family alone.
