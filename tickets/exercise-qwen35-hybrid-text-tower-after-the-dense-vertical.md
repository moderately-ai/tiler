---
id: exercise-qwen35-hybrid-text-tower-after-the-dense-vertical
title: Exercise the Qwen3.5 hybrid text tower after the dense model vertical
status: deferred
priority: p2
dependencies: [design-model-level-qualification-and-optimization, reclassify-language-model-work-as-a-conformance-track]
related: [define-first-metal-lm-workload, derive-transformer-operation-and-shape-surface, prove-the-c1-complete-model-execution]
scopes: [research/program-planning, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [language-model, qwen, metal, architecture-stress, class-conformance-fixture]
---
## Activation trigger

Activate after the selected dense decoder workload reaches model-boundary correctness and performance qualification. Do not pull hybrid recurrent-state semantics into the first dense vertical merely because the backend evidence already exists.

## User-visible outcome

Tiler executes and qualifies the exact Qwen3.5 0.8B text tower as a second architecture-stress workload, proving that the compiler and runtime can represent a decoder that interleaves Gated DeltaNet recurrent state with ordinary attention KV state without collapsing either into a generic cache or silently reusing dense-transformer assumptions.

## Primary evidence

The local `../lmbrrr` repository at `75ec511c` exercised the 0.8B Qwen3.5 text tower inside `openbmb/MiniCPM-V-4.6` snapshot `8169864629825dc1d755a5aa1cd8b5935dcbc83f` and pins the Candle fork at `cd2499cceae27a2b1192d7a89c123597479adf3a`. Its retained config names 24 layers arranged as 18 Gated DeltaNet plus 6 gated full-attention layers, partial RoPE, per-head Q/K normalization, SwiGLU, tied embeddings, F32 recurrent state, and distinct recurrent/conv and KV cache families. Its history and research records preserve correctness fixtures, fused-kernel measurements, chunk verification, state rollback, and refuted shortcuts.

## Required analysis

- Pin the exact language-only checkpoint or extracted text tower, immutable revision, config and tensor digests, tokenizer artifacts, and reference implementation revision.
- Separate reusable dense operations from hybrid-only semantics: Gated DeltaNet recurrence, depthwise causal convolution, output-gated attention, partial RoPE, MTP, recurrent and convolution state, KV state, chunk verification, and rollback.
- Define semantic state transitions, physical placement of the retained values, artifact identity, and numerical contracts before lowering. **Bounded 2026-08-04 by [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md):** this bullet read "physical cache/state placement, artifact identity, runtime lifetime and rollback rules". The Tiler runtime owns no cross-invocation state — not a KV cache, and not a recurrent or convolution state either — so lifetime and rollback are the consumer's, and this analysis states them as consumer obligations. A hybrid tower is *more* reason for that boundary, not less: two state families with chunk verification and rollback are exactly the serving-session machinery a consumer-agnostic compiler must not acquire. Every retained value crosses as an ordinary program input and output.
- Reuse the `lmbrrr` and Candle-fork fixtures and measurements as evidence inputs while re-deriving every Tiler contract; implementation in another stack is not proof that Tiler represents the same semantics.
- Keep vision, quantization, speculative decoding and MTP independently gated. The first language-only proof must not acquire those surfaces by checkpoint accident.
- Compare prefill and every decode-step logits under a tolerance derived before observation, require the declared greedy tie policy, and retain state-equivalence fixtures across chunking and rollback.

## Graph maintenance

Connect every newly required semantic operation, physical carrier for a retained value, runtime contract and Metal lowering to its existing owner where one exists; a "physical state carrier" that would live inside the runtime across invocations is refused rather than owned, per the bound above. File coherent verticals for missing capabilities rather than one ticket per crate. Revisit priority if hybrid Qwen becomes the selected production workload or if a dense-only design would otherwise freeze a cache or state boundary that cannot represent this tower.

## Closes when

The exact hybrid text tower has a dependency-ordered delivery graph grounded in the retained local implementation and official model artifacts; its two state families and recurrent transitions are explicit and identity-bearing; unrelated multimodal, quantized and speculative surfaces remain separately gated; and its model-level correctness and performance qualification is reproducible.

## Trigger check log

- 2026-08-04 — **not fired.** The trigger is the selected dense decoder workload reaching model-boundary correctness and performance qualification. [`prove-the-c1-complete-model-execution`](prove-the-c1-complete-model-execution.md) and [`prove-the-c1-stateful-attention-vertical`](prove-the-c1-stateful-attention-vertical.md) are both `todo`, so no dense vertical has been qualified. Recheck: those two statuses.
- 2026-08-09 — **not fired.** Both named dense-vertical prerequisites remain `todo`; neither model-boundary correctness nor the stateful attention vertical has been qualified. The hybrid-only recurrence, convolution, and state surfaces therefore remain deliberately downstream rather than parallel work. Recheck both prerequisite statuses with `tkt show`.
