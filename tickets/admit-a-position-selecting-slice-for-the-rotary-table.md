---
id: admit-a-position-selecting-slice-for-the-rotary-table
title: Admit a position-selecting slice for the rotary table
status: todo
priority: p2
dependencies: [integrate-the-autoregressive-decode-loop, reclassify-language-model-work-as-a-conformance-track]
related: [design-autoregressive-state-and-kv-cache, admit-the-reindex-and-broadcast-operation-families, compose-rotary-position-embedding-from-reindex-and-broadcast]
scopes: [implementation/ir, implementation/reference, contracts/foundation]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, semantics, operation-families, rope, kv-cache, language-model]
---
## User-visible outcome

The absolute position of a decode step's new tokens becomes a checked index expression over a bound extent instead of a host convention — so a caller that states position inconsistently is refused rather than silently returning a wrong token.

## Why this is a correctness trigger and not a bytes trigger

The sub-tensor-selection row's existing trigger is a prefill pass needing only the final position's logits, which is about 4,978,634,752 F32 bytes against 607,744. [Rung L5](../docs/research/runtime/autoregressive-state-and-kv-cache.md) supplies a second and stronger one: at batch 1 with a contiguous cache, **absolute position enters the decode program through `cos` and `sin` alone**. The mask is derivable from `T` and `S`, the cache extents from `C`, and the residual stream carries no position — so the rotary rows are the one input whose correct value depends on where the token sits, and a wrong row is a `[1, 128]` F32 tensor with the same shape, dtype, accessible range, and launch geometry as the right one. Every layer accepts it.

Binding the whole `[max_positions, 128]` table and selecting rows `C … C + T` by an index expression over the same bound extent that fixes the cache moves that from a convention into a coordinate map.

## The claim's exact limit, stated so it is not overread

A slice removes the **inconsistency** mode — the cache saying `C = 14` while the rotary rows say 0 — and does not remove the **wrong-cursor** mode, where a consistently wrong `C` produces a consistently wrong program that only the conformance oracle detects. Two different failures; this buys one of them.

## Required design

A slice is injective and not surjective, so it is outside `tiler::reindex-f32@1` — whose `reindex.split.not-surjective` refusal already names it as such — and outside `Broadcast`. Whether the required form is a general `Slice` family, a bounded offset-and-extent selection along one axis, or something the index vocabulary already reaches with a symbol-carrying coordinate, is the question; **note that no `IndexNode` variant currently carries an extent symbol outside a `FloorDiv` or `Modulo` divisor**, so a symbolic offset is itself a gap and must be costed as part of the answer.

## Closes when

Either a family is admitted with its refusals and its evaluator, or the approach is rejected with a ground and the support-matrix row keeps both triggers with this one's evidence attached.
