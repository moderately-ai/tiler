---
id: decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode
title: Decide whether one decoder-layer graph can serve prefill and decode
status: in-progress
priority: p1
dependencies: []
related: [assemble-the-decoder-layer-program, design-autoregressive-state-and-kv-cache, design-model-ingestion-and-complete-execution]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, semantics, language-model, identity]
claimed_from: todo
assignee: agent-decoder-fork
lease_expires_at: 1785950113
---
## What was found, and where

**Measurement, 2026-08-05, at `crates/tiler-reference/tests/decoder_layer.rs`.** The assembled decoder layer (P2) carries **fifty-eight** occurrences at the C1 prefill row (`T = 10`) and **sixty-two** at the C1 decode row (`T = 1`). It is not one graph at the two rows, and the difference is not extents.

**Fact — the mechanism.** `BroadcastAxisMapping::result_shape` refuses a many-to-one relation onto a result axis of extent below two, under `broadcast.mapping.relation-does-not-widen`, and the refusal's own documentation names the replacement: "written as a replication it is a reindex's unit-axis insertion". At `T = 1` six of the layer's widenings pad the position axis onto extent one:

- the two `[1024]` normalization weights (`w_input_layernorm`, `w_post_attention_layernorm`) lose their broadcast entirely, because the mapping left after removing the position axis is the identity and states no widening either — `[1024] -> [1, 1024]` is a `Reindex::insert_unit_axis` and nothing else;
- the two `[128]` per-head weights and the two `[2, 1]` rotary signs keep a broadcast over the head axis and gain an insertion.

Four extra occurrences, and two broadcasts become reindexes. `a_rank_pad_onto_a_single_position_refuses` watches the refusal at all three widenings beside its admitted neighbour at ten positions; `a_single_new_position_changes_six_widenings` counts the delta.

**Fact — the cache half of the claim does hold.** At a fixed `T`, the layer built against `C = 0` and against `C = 8` has an identical occurrence signature: same families in the same order, same reindex forms, same contraction structures, same broadcast relations (`a_nonempty_cache_changes_no_occurrence`). So the sequence extension is exactly the binding change the autoregressive-state record says it is.

## What this bears on

Two proposals in merged records read as though one program serves both phases:

- [the autoregressive-state record](../docs/research/runtime/autoregressive-state-and-kv-cache.md) — "the decode step is not a second program design. It is L4's program with two inputs, two occurrences, and one changed extent binding", and candidate **P2**'s ground that P1 "packages the same twenty-two steps twice".
- [the complete-model record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) — "the complete C1 run is nine forward passes of thirty executions, **270 executions over exactly three artifact identities**, and a build that produced a fourth would have specialized something it must not."

Neither is refuted as a *design*; what is refuted is that the current semantic vocabulary can express it. The attention-block work already recorded the weaker half — an explicit `BroadcastAxisMapping` carries its declared result extents into canonical identity, so a prefill row and a decode row differ in attribute bytes. This ticket is the stronger half: they differ in occurrence count and family sequence, so no amount of extent-symbol plumbing inside a mapping's *extents* field fixes it on its own. A mapping would additionally have to state a widening whose legality is not decidable at construction.

## Candidates, none eliminated here

1. **Accept two identities per layer**, one for prefill and one for decode, and correct the three-identity claim to four (P1, P2-prefill, P2-decode, P3). Cheapest; costs the record's stated invariant and a test that can assert it.
2. **Admit a symbolic-extent broadcast mapping**, so a `replicate` onto a symbolic extent is legal and its widening is checked where the extent is bound. Reaches the records' claim; is a semantic-vocabulary change with identity, validation and lowering consequences, and is Tom's.
3. **Spell every position-axis rank pad as a unit-axis insertion followed by a stretch**, uniformly at both rows. Refused as written today — `StretchUnit` onto a result extent of one is the same `relation-does-not-widen` refusal — so this candidate needs the same relaxation as 2 and is not independent of it.
4. **Chunked decode** (`T >= 2` always, with a one-token chunk padded), which makes the question disappear at the cost of computing positions the consumer discards. Not evaluated.

## Closes when

The disagreement between the measured occurrence counts and the two records' one-program claim is resolved by an explicit decision: either the records are corrected to state two identities per layer with the ground above, or a semantic change is accepted that makes the two rows one graph. Whichever it is, the affected sentences in both records move in the same change, and `decoder_layer.rs`'s two count assertions are updated or kept as the evidence.
