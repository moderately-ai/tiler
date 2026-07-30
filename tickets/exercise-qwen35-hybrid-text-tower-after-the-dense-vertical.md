---
id: exercise-qwen35-hybrid-text-tower-after-the-dense-vertical
title: Exercise the Qwen3.5 hybrid text tower after the dense model vertical
status: todo
priority: p2
dependencies: [design-model-level-qualification-and-optimization]
related: [define-first-metal-lm-workload, derive-transformer-operation-and-shape-surface]
scopes: [research/program-planning, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [language-model, qwen, metal, architecture-stress]
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
- Define semantic state transitions, physical cache/state placement, artifact identity, runtime lifetime and rollback rules, and numerical contracts before lowering.
- Reuse the `lmbrrr` and Candle-fork fixtures and measurements as evidence inputs while re-deriving every Tiler contract; implementation in another stack is not proof that Tiler represents the same semantics.
- Keep vision, quantization, speculative decoding and MTP independently gated. The first language-only proof must not acquire those surfaces by checkpoint accident.
- Compare prefill and every decode-step logits under a tolerance derived before observation, require the declared greedy tie policy, and retain state-equivalence fixtures across chunking and rollback.

## Graph maintenance

Connect every newly required semantic operation, physical state carrier, runtime contract and Metal lowering to its existing owner where one exists. File coherent verticals for missing capabilities rather than one ticket per crate. Revisit priority if hybrid Qwen becomes the selected production workload or if a dense-only design would otherwise freeze a cache or state boundary that cannot represent this tower.

## Closes when

The exact hybrid text tower has a dependency-ordered delivery graph grounded in the retained local implementation and official model artifacts; its two state families and recurrent transitions are explicit and identity-bearing; unrelated multimodal, quantized and speculative surfaces remain separately gated; and its model-level correctness and performance qualification is reproducible.
