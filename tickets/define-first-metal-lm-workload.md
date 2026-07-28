---
id: define-first-metal-lm-workload
title: Define the first representative Metal language-model workload
status: awaiting-decision
priority: p1
dependencies: [scope-optimized-metal-lm-inference]
related: [derive-transformer-operation-and-shape-surface, design-model-level-qualification-and-optimization]
scopes: [research/program-planning, contracts/integrations, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, planning, language-model, workload, metal, inference]
---
## Decision needed (2026-07-28)

**The question, atomic:** which model family is the first workload — a GPT-2-class model or a small Llama-class model? Both survivors are small decoder-only transformers executed in `f32`, batch 1, on one Apple GPU; they differ only in operation surface. The elimination run below removed every other candidate, so this is what remains.

| | Option A — GPT-2-class (LayerNorm, GELU, learned absolute positions, multi-head attention) | Option B — small Llama-class (RMSNorm, SwiGLU, rotary positions, grouped-query attention) |
| --- | --- | --- |
| **Enables** | The smallest operation surface that still reaches every rung, so L2 derives the fewest families and L3–L6 are proven against the minimum. | Proving the ladder against the architecture actually deployed today, so L2's derived surface is the one that will still be wanted at L6 and L7. |
| **Prevents** | Nothing architecturally, but it proves the ladder against an architecture largely superseded in practice, so a later move to a current model reopens L2 for RoPE, RMSNorm, and gated feed-forward. | Nothing, but it adds rotary embedding, a gated feed-forward, and grouped-query attention to L2's surface before the unbatched contraction of L3 is proven. |

**Recommendation: Option B**, on the ground that L2's derived surface is inherited by every rung above it and re-deriving it later is the expensive mistake, whereas carrying three extra operation families through L2 is a one-time cost paid in a research ticket. The counterpoint is real and should be weighed: Option A makes L3 and L4 strictly easier to specify and to attribute failures in, and the ladder's purpose is to prove the *architecture* rather than to ship a model.

**This is a product choice, not a correctness one** — both options are executable and neither is blocked by evidence. It is put to Tom rather than decided here for that reason.

**The activation trigger has fired.** `scope-optimized-metal-lm-inference` is `done`, so the "do not start this before its trigger" instruction recorded below is satisfied and no longer a stop sign; this ticket is waiting on the answer above and on nothing else.

**One thing the decision does not settle, whichever way it goes.** The exact checkpoint's configuration — layer count, head count, head dimension, context length, vocabulary — must be read from the actual model config before L2 derives shapes from it. Nothing here asserts those numbers, and they should not be taken from memory.

## Scope

Select and bound the first language-model inference workload that will drive
Tiler's capability growth. Do not use an unspecified "transformer" or
"LLM-compatible" claim as a substitute for an executable workload.

## User-visible question

What is the smallest representative language-model workload whose successful
execution would demonstrate that Tiler's compiler architecture can grow into a
useful Metal inference library?

## Required evidence and decisions

- Compare candidate model classes using their actual operation, dtype, shape,
  state, weight, and execution requirements.
- State the supported batch, prompt, sequence, and decode bounds.
- State the initial dtype and numerical requirements without preselecting
  quantization merely because it is cheaper.
- Name the initial Apple target profile and which claims require a live device.
- Define correctness and performance success measures at user-observable model
  boundaries.
- Explicitly exclude or defer training, distributed execution, speculative
  decoding, unsupported model architectures, and unbounded dynamic shapes.

Eliminate candidates that cannot test the intended architecture or that require
unrelated capabilities before presenting any genuine product choice to Tom.

## Ticket-producing outcome

File dependency-ordered follow-up tickets only for workload requirements not
already owned by the graph. Each new ticket must name the model-visible outcome
it enables, its evidence prerequisite, and its reconsideration trigger if
deferred.

## Closes when

One bounded workload profile and its success envelope are durably recorded;
the selection evidence and rejected candidates are reproducible; and every
newly exposed subsystem requirement is either linked to an existing owner,
filed as a scoped ticket, or explicitly deferred with a trigger.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L1** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** `scope-optimized-metal-lm-inference` is accepted. **Fired:** that ticket is `done` as of 2026-07-28, so this gate is open and the instruction below is history rather than a hold.

**Rests on:** nothing — it is the ladder's first rung.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Research 2026-07-27 — elimination run, one genuine choice remains

### What the workload has to be *for*

**Fact — the current supported profile.** `tiler-ir`'s standard semantic registry admits four governed operations: `constant_f32_op`, `multiply_f32_op`, `add_f32_op`, and `strict_serial_sum_f32_op` (`crates/tiler-ir/src/semantic/operation.rs:95-113`). No contraction, no softmax, no normalization family exists above R2.

**Inference.** The workload is therefore not something Tiler can nearly run. Its job is to *order* the ladder's rungs — L2 derives its operation and shape surface from this choice, and everything above L2 inherits that. Choosing wrongly costs the derivation, not a failed execution.

### Candidates eliminated, with the derivation

**Encoder-only (BERT-shaped).** Eliminated: it has no autoregressive decode, so rung L5 — stateful prefill and token decoding, the KV-cache design — would have nothing to exercise. A workload that cannot reach a named rung of the ladder it exists to drive is not representative of the thing being built, whatever else it demonstrates.

**A synthetic single attention block.** Eliminated as a *workload*, though it remains correct as a *rung*: it is what L3 and L4 already are. It has no user-observable model boundary, so this ticket's requirement to "define correctness and performance success measures at user-observable model boundaries" has nothing to attach to, and L6 and L8 would be unreachable.

**A 7B-class model.** Eliminated on two independent grounds, either sufficient. It does not fit in `f32` on the qualified target, so it forces quantization — which this ticket explicitly forbids preselecting "merely because it is cheaper", and which is L7's decision rather than L1's. And it needs weight streaming or multi-device memory management, which the ladder records as deferred with no reserved seam.

**Mixture-of-experts, state-space (Mamba-shaped), and encoder-decoder architectures.** Eliminated: each requires a capability family the ladder does not name — expert routing, selective scan, cross-attention — so choosing one would silently add a rung nobody scheduled.

### What survives, and the bounds that apply to either survivor

A **small decoder-only transformer, executed in `f32`, single sequence, on one Apple GPU**. Concretely:

- **Batch** 1. Batching is a throughput concern and adds a batched-contraction shape class before the unbatched one is proven.
- **Prompt and sequence** bounded and stated per run, not unbounded. The sourced-extent profile bounds symbolic extents deliberately, and the ladder records unconstrained dynamic shapes as deferred.
- **Decode** to a fixed token budget, so a run terminates and a measurement has a denominator.
- **Dtype `f32` throughout**, not chosen for cost but because it is the only width with a governed numerical contract and a measured Apple realization today. Quantization is L7 and depends on milestone 2Q.
- **Target profile:** the qualified row in the Apple GPU numerical behaviour record — Apple M4 Max, macOS 27.0, Xcode 26.6, offline `metalfe-32023.883`. **Which claims need a live device:** every execution and numerical-delivery claim. Compile-side claims — emitted operations, module options, artifact identity — do not.
- **Success measures at the model boundary:** token-sequence equality against a reference implementation for a fixed prompt and greedy decode, and decode latency per token plus prefill latency, both as min-of-N on a quiet host. Not a throughput figure, which would need batching.
- **Excluded, per the ladder's deferral table:** training, distributed execution, speculative decoding, unsupported architectures, unbounded dynamic shapes.

### The one genuine choice, which is Tom's

Two model families survive the elimination and they encode different valid priorities. Both are small decoder-only transformers in `f32`; they differ in operation surface. The choice, its enables and prevents, the recommendation, and the counterpoint are stated at the top of this file under **Decision needed**.
