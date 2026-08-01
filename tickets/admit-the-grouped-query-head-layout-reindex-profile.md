---
id: admit-the-grouped-query-head-layout-reindex-profile
title: Admit the grouped-query head-layout reindex profile
status: done
priority: p1
dependencies: [admit-the-reindex-and-broadcast-operation-families]
related: [design-attention-program-vertical, admit-the-attention-contraction-structures, compose-rotary-position-embedding-from-reindex-and-broadcast, assemble-the-causal-self-attention-block-program]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, structural, attention, gqa, language-model]
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

## Outcome

The profile lands as `crates/tiler-reference/tests/grouped_query_head_layout.rs`, a conformance module carrying the four maps as named artifacts and their evidence. **No public API was added anywhere.** The maps are compositions of forms `tiler::reindex-f32@1` already admits, so nothing in the family needed widening; a constructor named for this checkpoint's grouped-query layout would have put a consumer-specific workload concept inside the consumer-agnostic compiler core, and the generic forms it would wrap are already public. `crates/tiler-ir` is unchanged.

**The four maps, as landed.** Each is a `Vec<ReindexForm>` applied in order through `F32Reindex::apply`, so every occurrence is admitted by the registered operation authority against its own operand and every result shape is the family's derivation rather than a declaration.

| map | shapes | forms, verbatim |
| --- | --- | --- |
| `query_projection_split` | `[T, 2048] -> [T, 16, 128]` | `split_axis(1, [16, 128])` |
| `query_head_group_layout` | `[T, 16, 128] -> [8, 2, T, 128]` | `split_axis(1, [8, 2])`, `permute_axes([1, 2, 0, 3])` |
| `key_value_head_layout` | `[S, 1024] -> [8, S, 128]` | `split_axis(1, [8, 128])`, `permute_axes([1, 0, 2])` |
| `attention_output_head_merge` | `[8, 2, T, 128] -> [T, 2048]` | `permute_axes([2, 0, 1, 3])`, `merge_axes([1, 2])`, `merge_axes([1, 2])` |

The key and the value are one map applied to two edges, not two maps: both projections are `[S, 1024]` and both feed a `[g, s, d]` operand, so a second spelling would be two names for one thing. `the_key_and_value_edges_are_one_map` states that by comparing canonical encodings, so a future divergence fails a check rather than waiting for a reader to notice.

**The direction.** The group is the major axis of the `(8, 2)` split, so `h = 2g + r` and the group a head reads is `h / 2`. That is not a convention chosen by this profile: `split-axis` is already normatively a row-major factorization with the major factor first, and `ReindexForm::split_axis` already names this exact consequence. The profile's module states the direction and cites that rule rather than restating it as a second authority.

**Bit comparison.** `the_head_split_reproduces_repeat_kv_and_the_tile_reading_does_not` recomputes the `repeat_kv` materialization in the test from the repeat-interleave rule alone — head `h` of `[16, S, 128]` is group `h / 2` — rather than reading the retained TSV, then applies both readings to it. Both produce an identically shaped `[8, 2, 10, 128]` result over 20,480 elements. The interleave reading `h = 2g + r` differs from group-constancy at **0** elements and **0** heads; the tile reading `h % 8` differs at **17,920** elements over **14** heads, and the two readings agree at exactly the 2 heads where `h / 2` and `h % 8` coincide. Those independently recomputed counts equal the probe's retained `gqa_repeat_kv_matches_floor_div_differing_elements`, `gqa_repeat_kv_matches_modulo_differing_elements`, and `gqa_heads_whose_source_differs_between_the_two_readings` rows. `the_query_head_to_key_head_table_is_floor_division` reproduces the record's `gqa_query_head_to_key_head` row `0 0 1 1 … 7 7` from the map itself, and the tile reading's table is `h % 8`.

**Totality.** Every map is evaluated over distinct ascending payloads at the C1 shapes and compared element for element against a closed form hand-derived from the profile's statement (`(g, r, t, d)` reads projection column `(2g + r) · 128 + d` of row `t`), not against a second implementation of split and permute. `assert_total_bijection` then requires the payload to be a permutation of `0..count`, which is totality over the declared output domain and bijectivity onto the operand's domain in one check. `the_output_merge_inverts_the_query_layout` shows the output map is the exact inverse: the round trip returns the projection unchanged in both payload and shape. The static constraints `2048 = 16 × 128` and `1024 = 8 × 128` are decided by the family at each split, so no symbolic requirement arises.

**Refusals, all under the family's own named rules — none invented.**

- `reindex.split.not-surjective` — `split_axis(1, [16, 64])` on `[T, 2048]`: the `hidden_size / num_attention_heads = 64` divide the evidence prerequisite names accounts for half the projection, so the map is a slice.
- `reindex.split.not-total` — `split_axis(1, [16, 128])` on `[S, 1024]`: the query head count applied to the key projection reads past the axis.
- `reindex.permute.not-a-permutation` — at construction for a repeated axis (`[1, 2, 0, 1]`), and at the occurrence for a distinct order naming an axis the rank-four operand lacks (`[1, 2, 0, 4]`).
- `reindex.permute.rank` — the key layout's rank-three order applied to the rank-four query layout.
- `reindex.merge.non-adjacent-axes` — the output inverse spelled without first moving the position axis out from between the group and repeat axes (`merge_axes([0, 2])`); adjacency is a property of the axes alone, so it refuses at construction.

Each refusal is paired with its admitted neighbour in the same test, so the checks are known to discriminate rather than to refuse every form this profile presents.

**Every check was watched failing** — not inferred to be capable of it.

- Query map respelled as the tile reading (`split_axis(1, [2, 8])` + `permute_axes([2, 1, 0, 3])`, the *same* `[8, 2, 10, 128]` shape): `the_query_layout_reads_head_two_g_plus_r_at_every_coordinate` fails on the element comparison, not on the shape.
- Interleave split replaced by the tile split: the `repeat_kv` comparison fails with `left: 17920, right: 0`, and the head table fails with `[0, 1, …, 7, 0, 1, …, 7]` against `[0, 0, 1, 1, …, 7, 7]`.
- Query split factors reversed to `(2, 8)`: the shape derivation and the query element test both fail.
- The malformed split replaced by the admitted one: the refusal test fails on `unwrap_err` of an `Ok`.
- The permutation dropped from the output merge: the inverse test fails with `[16, 1280]` against `[10, 2048]`.

**Not done, deliberately.** No documentation edit: `docs/` is outside this ticket's scopes, and the normative sentence the delivery required — the row-major major-factor-first direction — already exists in the registered `tiler::reindex-f32@1` definition and in `ReindexForm::split_axis`. Restating it in a second authority would have created two places to drift. The `[1, …]` batch axis the pinned reference carries is omitted throughout: it is extent one, contributes no element, and is named by none of the maps, so the compared element counts are the probe's exactly.
