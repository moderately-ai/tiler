---
id: admit-the-grouped-query-head-layout-reindex-profile
title: Admit the grouped-query head-layout reindex profile
status: in-progress
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families]
related: [design-attention-program-vertical, admit-the-attention-contraction-structures, compose-rotary-position-embedding-from-reindex-and-broadcast, assemble-the-causal-self-attention-block-program]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, structural, attention, gqa, language-model]
claimed_from: todo
assignee: worker-admit-the-gr
lease_expires_at: 1785561991
---
## User-visible outcome

A program can say that a `[T, 2048]` query projection is eight groups of two heads of width 128, and that a `[T, 1024]` key projection is the eight heads those groups read — so that grouped-query attention is a coordinate map rather than a materialized repetition.

## Evidence prerequisite

**Fact — the shapes force the asymmetry.** `q_proj` is `[2048, 1024]` because 16 query heads × 128 = 2048; `k_proj` and `v_proj` are `[1024, 1024]` because 8 key/value heads × 128 = 1024. `num_key_value_groups` is 16 / 8 = 2. Head dimension is the declared 128 and **not** `hidden_size / num_attention_heads = 64`; a planner that divides produces a silently wrong shape on this checkpoint.

**Measurement — there are two readings of the repetition and one of them is wrong at fourteen of the sixteen heads.** `repeat_kv` is repeat-interleave, so query head `h` reads key head `h // 2`. Splitting the 16-head axis as `(g = 8, r = 2)` with `h = 2g + r` reproduces it at **0 differing elements** of a `[1, 16, 10, 128]` comparison. The repeat-tile reading `h mod 8` differs at **17,920** elements over 14 of the 16 heads, and produces an identically shaped tensor. The [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains both counts. **Inference — a check that compared only shapes would have passed both**, which is why the mapping is a delivery obligation rather than a comment.

**Inference — the layout is what makes the repetition free.** Under [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md)'s structure-carrying key, `grtd,gsd->grts` mentions `r` in the query operand and the output and never in the key operand, so the repetition is not an operation at all. Under fixed-arity keys it would be a `Broadcast` to sixteen heads plus a `Reindex`, forming a `[16, S, 128]` intermediate — 67,108,864 bytes for `K` and again for `V`, per layer, at the B1-d row. This ticket delivers the coordinate maps that let the free form be stated.

## Required delivery

- **Four coordinate maps, each a checked `Reindex` over admitted initial forms**: `[T, 2048] -> [T, 16, 128]` and then `[T, 16, 128] -> [8, 2, T, 128]` for the query; `[T, 1024] -> [T, 8, 128] -> [8, S, 128]` for the key and the value; and the inverse `[8, 2, T, 128] -> [T, 2048]` for the attention output. Each is a split, a merge, or an axis permutation — no coordinate arithmetic is needed here, which is what distinguishes this profile from [the rotary composition](compose-rotary-position-embedding-from-reindex-and-broadcast.md).
- **The `h = 2g + r` mapping as a tested property**, bit-compared against the reference's `repeat_kv` at the C1 prefill shape, with the `h mod 8` reading retained as the perturbation that must differ. State the direction explicitly in the normative reference: group index is the *major* axis of the split, because a row-major split of 16 into `(8, 2)` is what makes `h // 2` the group.
- **Totality of every map over its declared output domain**, and the shape constraints that make each split and merge legal — 2048 = 16 × 128 and 1024 = 8 × 128 are static, so these resolve without a symbolic requirement.
- **Explainable refusal.** A split whose factors do not multiply to the axis extent, an axis permutation that is not a permutation, and a merge over non-adjacent axes each refuse at construction under their own named rule.

## Non-goals

The contraction that consumes these maps, the materialized-repetition alternative, any physical layout claim, and a general `repeat` or `expand` family. A `Reindex` "does not claim that storage was transposed or copied"; whether one of these maps costs a dispatch is a planning question owned by [`plan-the-materialized-attention-decomposition`](plan-the-materialized-attention-decomposition.md).

## Closes when

All four maps verify, refuse their malformed neighbours under named rules, and the `h = 2g + r` mapping is bit-compared against `repeat_kv` with the `h mod 8` perturbation demonstrated failing.
