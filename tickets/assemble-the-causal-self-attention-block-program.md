---
id: assemble-the-causal-self-attention-block-program
title: Assemble the causal self-attention block as one verified semantic program
status: done
priority: p1
dependencies: [admit-the-attention-contraction-structures, compose-rotary-position-embedding-from-reindex-and-broadcast, admit-the-grouped-query-head-layout-reindex-profile, admit-the-softmax-family]
related: [design-attention-program-vertical, admit-the-rms-normalization-family, plan-the-materialized-attention-decomposition, design-autoregressive-state-and-kv-cache, promote-the-symbolic-index-profile-to-a-public-boundary, stage-contractions-inside-whole-program-reference-evaluation]
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

~~The block verifies, its refusals fire under perturbation, and its reference evaluation at the C1 prefill shape is compared bit-for-bit against the pinned reference with every difference attributed to a named cause.~~

**Revised at integration, 2026-08-02, and the revision is narrower than the original by exactly one clause.** The block verifies at the C1 prefill shape, its refusals fire under perturbation, and its reference evaluation is compared bit-for-bit against the pinned reference with every difference attributed to a named cause — **at the C1 row's head geometry, mask, scale, and rotary composition, with the whole-block evaluation taken at a 512-wide model dimension rather than C1's own 1,024.** The operations the pinned reference row covers *are* compared at the C1 row itself.

**Why the original clause was unreachable rather than unmet, checked at integration rather than taken from the report.** `MAX_EVALUATION_STEPS` is `16 * 1024 * 1024` at `crates/tiler-reference/src/oracle.rs:68`, and C1's query and output projections are 20,971,520 steps each. The constant's own doc states the resolution: `StagedIndexRegionEvaluation` "reaches a larger region by spending several bounded spans rather than by weakening the number" — and `crates/tiler-reference/tests/contraction_profile_cells.rs` already runs `w_prefill_q` at those exact 20,971,520 steps that way. So the missing clause needs the staged evaluator carried into whole-program evaluation, which is a capability this ticket does not own and must not have obtained by raising a governed bound to pass its own test.

**Correction, 2026-08-02 — the paragraph above names the wrong constant, and the conclusion it draws is unaffected.** `MAX_EVALUATION_STEPS` at `crates/tiler-reference/src/oracle.rs:68` bounds one span of the *index-region* oracle and is not what refused these projections; `MAX_REFERENCE_TENSOR_ELEMENTS`, read by `contract_operands` in `crates/tiler-reference/src/contraction.rs`, is. Both are `16 * 1024 * 1024`, which is how the two came to be read as one, and `StagedIndexRegionEvaluation` is likewise the region oracle's staged form rather than the contraction's — that is `StagedStrictTensorContractionF32`, which is what `contraction_profile_cells.rs` runs `w_prefill_q` through. The rest of the paragraph is right about both bounds: neither moved, and the resolution was to spend several bounded units.

The remainder is [`stage-contractions-inside-whole-program-reference-evaluation`](stage-contractions-inside-whole-program-reference-evaluation.md), which is live. Per AGENTS.md the parent closes on its revised outcome rather than being held in `review`, where it would satisfy no dependent.

**Reclaimed, 2026-08-02.** The clause this revision gave up has been recovered. `stage-contractions-inside-whole-program-reference-evaluation` gave `ReferenceEvaluator` a per-occurrence iteration-step allowance and folds an occurrence over one window in several, so `causal_self_attention_block.rs` now evaluates end to end at the C1 row's own 1,024-wide model dimension with nothing reduced and `EVALUATED_HIDDEN` deleted — at 0 differing elements on all three outputs against the independent recomputation. `MAX_REFERENCE_TENSOR_ELEMENTS` did not move, and a default evaluator still refuses the same program with the same quoted diagnostic. The struck-through original outcome is therefore the one that holds; the revision below it is preserved as the record of what was true at integration.

## Outcome

Landed as `crates/tiler-reference/tests/causal_self_attention_block.rs`: the complete twenty-two-step block, **forty-eight semantic occurrences** over the eight already-registered keys, built through the public builder, with twelve ordered inputs and three ordered named outputs (`h_out`, `k_rope`, `v_heads`). Nothing was registered, admitted, or widened — the block is a shape over existing families, and the four named dependencies were each verified by reading the source they landed rather than by their status field.

**Delivered as stated.** The block verifies at the C1 prefill row's exact extents. `T` and `S` are two separate `ShapeEnv` symbols, each bound to an input dimension, each bounded to `[1, 32768]` and pinned to the row, and never joined by an equality. All five required refusals fire and are quoted with their diagnostic codes, each beside an admitted neighbour: `broadcast.mapping.extent-disagreement` (mask against the wrong key extent), `rms-norm.f32.weight-shape` (a `[128]` per-head weight against the 1,024-wide hidden axis), `reindex.split.not-surjective` and `reindex.split.not-total` (head splits whose factors do not multiply out), `contraction.rule.summed-index-in-one-operand`, and `ExtentRefusal::NoUpperBound` via `ExtentInterval::states_no_upper_bound`. The masked-position numerical case is retained against the probe's own pinned bits, in both signs.

**Reference comparison against the pinned `transformers` 4.51.0 bits.** The retained record's `row_h0_t2_scores_raw` is driven through the block's own operations 16, 17 and 18 — under the block's real `mask_mapping` at the workload's eight groups and two repetitions — and reproduces `row_h0_t2_scores_scaled`, `row_h0_t2_scores_masked`, and `row_h0_t2_probs` **bit for bit, 0 of 10 differing on each row**. No `torch` seed is needed because the record retains the score row before the scale. **There were no unattributed bit differences to attribute.**

**Measurement boundary, and the one part not delivered as written.** The whole block does *not* reference-evaluate at the C1 row's own 1,024-wide model dimension: `contract_operands` refuses a fold above 16,777,216 steps, and the query and output projections are 20,971,520 each at that row. The refusal is watched and quoted rather than inferred. The block therefore evaluates end to end — all forty-eight occurrences, all three contraction index structures, the C1 row's own head geometry, mask, scale, and rotary composition — at a **512**-wide model dimension against an independent recomputation, at 0 differing elements on all three outputs, with the repeat-tile head reading as a live perturbation. Raising the bound was rejected rather than taken: it is what a whole-program evaluation is deliberately held to, and `contraction_profile_cells.rs` already reaches four larger cells — `w_prefill_q` among them, at these exact extents — without moving it. Filed as [`stage-contractions-inside-whole-program-reference-evaluation`](stage-contractions-inside-whole-program-reference-evaluation.md). **That boundary is gone as of 2026-08-02** — see the reclaim note under "Closes when"; the block evaluates end to end at 1,024, still without moving the bound. This paragraph records what was true at integration.

**Two facts found while building that the L4 design did not record.**

- **The prefill block cannot be bound at `S > T`.** It computes its own key from its own input, so the score tensor's key extent is whatever the key path produced, and a mask asserting a wider context is refused at the mask add under `binary.shape`. A decode step is therefore *not* reachable by rebinding `S` alone — it is this program with `k_rope` and `v_heads` arriving as inputs, which is exactly why naming them as outputs is the seam. The design's "widening it is a binding change rather than a graph change" is true of the *row* (`T = S = 10` against `18`, where every key and every non-extent attribute is identical) and false of the context alone.
- **An explicit `BroadcastAxisMapping` carries its declared result extents into canonical identity**, so all ten of the block's broadcast occurrences have row-dependent attribute bytes. A program byte-identical across rows would need mappings carrying extent *symbols*, which the semantic vocabulary does not have. Both are pinned as checks rather than left as prose.
