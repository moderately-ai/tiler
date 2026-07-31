---
id: assemble-the-causal-self-attention-block-program
title: Assemble the causal self-attention block as one verified semantic program
status: todo
priority: p1
dependencies: [admit-the-attention-contraction-structures, compose-rotary-position-embedding-from-reindex-and-broadcast, admit-the-grouped-query-head-layout-reindex-profile, admit-the-softmax-family]
related: [design-attention-program-vertical, admit-the-rms-normalization-family, plan-the-materialized-attention-decomposition, design-autoregressive-state-and-kv-cache, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, attention, transformer, vertical-slice, language-model]
---
## User-visible outcome

One complete causal self-attention block — twenty-two typed steps from the residual stream in to the residual stream out — verifies as a semantic program and reference-evaluates to the pinned reference's answer at the C1 conformance row's prefill shape. This is the first program in the corpus with more than one output and the first that exercises all three contraction index structures.

## The program

**Proposal — from the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md)**, which holds the complete operation table, the shapes at C1 and B1-d, and the byte arithmetic. In summary: RMS normalization of the residual stream; Q, K, V projections under structure 1; head splits; per-head Q and K normalization over the 128-wide axis; rotary embedding; the grouped-query head layout; the score contraction under structure 2; the scale by `0x3db504f3`; the broadcast mask add; softmax over the key axis; the value contraction under structure 3; the head merge; the output projection under structure 1; and the residual add.

**Twelve ordered inputs and three ordered named outputs.** The outputs are the residual stream `h_out`, plus `k_rope` and `v_heads` — the values a KV cache would retain. **Inference — naming those two is the entire seam L5 attaches to.** A single-output framing would force [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md) either to recompute them or to reach inside the block, and both are the collapse the multi-result rule exists to prevent. Nothing here implements a cache.

## Evidence prerequisite

**Fact — the block is the batch-1 prefill shape, where `S = T`.** The block computes its own `K` and `V` from its own input, so neither contraction has an operand whose production is undefined — which is precisely the condition [the L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) said was missing when it deferred structures 2 and 3. `T` and `S` stay separate extent symbols so that a decode step is a binding change rather than a graph change.

**Fact — the scale multiplies the score, not an operand**, from `eager_attention_forward` line 157. **Measurement — the difference is not marginal:** pre-scaling the query changes 1,404 of the 1,600 score elements at the C1 prefill shape. So the scale's graph position is semantics; a rewrite that pushed it onto an operand would be a value change with no permission behind it.

**Measurement — the recomputation is the reference's own composition.** The [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) reproduces `modeling_qwen3.eager_attention_forward`'s weights and output at 0 differing elements, so the intermediates the design's worked example exposes describe the reference rather than a lookalike.

**Fact — the mask is an F32 program input.** `[T, S]`, broadcast over the two head axes and added. 400 bytes at the C1 prefill row and 268,435,456 at B1-d. The derived-predicate alternative needs a boolean dtype the registry does not admit and an index-domain comparison [ADR 0084](../docs/decisions/0084-reference-canonical-index-expressions-from-domain-predicates.md)'s vocabulary excludes by construction; its activation trigger is a row where the mask outgrows the program and this is not it.

## Required delivery

- **The complete program, constructed through the public builder**, with every `Broadcast` axis mapping explicit and no implicit rank padding or extent-one stretching anywhere.
- **Three ordered named outputs**, with `k_rope` and `v_heads` retained as results rather than as internal values, and the shape environment binding `T` and `S` as separate bounded symbolic extents.
- **Reference evaluation of the whole block** against the pinned `transformers` 4.51.0 reference at the C1 prefill shape, over synthetic operands at a recorded seed, comparing exact F32 bit patterns. Where bits differ, attribute the difference to a named reduction-order divergence rather than reporting a tolerance — the probe already measures that the score contraction's two spellings differ at 943 of 1,600 elements in F32 and 0 of 1,600 in float64.
- **Construction-time validation that actually fires.** Perturb each of these and watch it refuse: a `[T, S]` mask against the wrong key extent; a `[128]` per-head norm weight against the 1,024-wide hidden axis; a head split whose factors do not multiply out; a contraction structure whose contracted index appears in one operand; an unbounded extent symbol.
- **The masked-position numerical case**, which is reachable from ordinary data rather than adversarial: at query position 0 the probability row is `0x3f800000` followed by nine exact `0x00000000`, and with a negative `v` at the attended key the value contraction's seed is `0x80000000` while the completed fold is `0x00000000`. Retain it, because a schedule that skipped masked contributors would return the other sign and nothing else in the corpus would notice.
- **The `[2, 1]` rotary sign input**, which exists because `tiler::constant-f32@1` produces rank zero only.

## Non-goals

Every physical question: schedules, covers, fusion, materialization, cost, and any Metal work. Also out of scope are the KV cache and the append, the MLP half of the decoder layer, the embedding gather, the vocabulary projection, BF16 ingestion, the rotary table's construction, and any block-level or model-level numeric tolerance — which is L8's under [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) and which L1 already fixes cannot be composed from per-operation tolerances.

## Closes when

The block verifies, its refusals fire under perturbation, and its reference evaluation at the C1 prefill shape is compared bit-for-bit against the pinned reference with every difference attributed to a named cause.
