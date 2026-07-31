---
id: define-first-metal-lm-workload
title: Define the first representative Metal language-model workload
status: done
priority: p1
dependencies: [scope-optimized-metal-lm-inference]
related: [derive-transformer-operation-and-shape-surface, design-model-level-qualification-and-optimization, exercise-qwen35-hybrid-text-tower-after-the-dense-vertical, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [research/program-planning, contracts/integrations, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, planning, language-model, workload, metal, inference]
---
## User-visible outcome

The first model-level delivery target is a reproducible, bounded F32 execution of pinned `Qwen/Qwen3-0.6B-Base` whose prefill and decode logits, greedy tokens, latency envelope, artifacts, and exclusions are explicit enough to derive the implementation ladder without silently importing unrelated architectures or features.

## Outcome (2026-07-30)

**Selected: Option B, `Qwen/Qwen3-0.6B-Base` at immutable revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`.** The first complete language-model workload is the pinned dense Qwen checkpoint widened to F32, batch 1, on one Apple GPU, with bounded prompt, context, and decode lengths. This selection makes the first delivery graph prove the reusable modern dense-decoder surface—RMSNorm, Q/K RMSNorm, SwiGLU, RoPE, GQA, ordinary KV cache, and head dimensions independent of `hidden_size / heads`—without importing MoE, vision, hybrid recurrence, quantization, chat templates, sampling, or thinking-mode semantics.

GPT-2-shaped blocks remain useful as narrow diagnostic fixtures because their smaller operation surface makes failures easier to localize, but they are not the representative model workload and cannot weaken the Qwen-derived delivery graph. `exercise-qwen35-hybrid-text-tower-after-the-dense-vertical` owns the subsequent hybrid architecture stress after dense model-level qualification.

The decision releases this ticket to finish its bounded workload profile and durable success envelope; it does not by itself satisfy the closing conditions.

## Accepted decision record (2026-07-28)

**The question, atomic:** which exact model is the first workload — a GPT-2-class checkpoint or `Qwen/Qwen3-0.6B-Base` at immutable revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd`? Both survivors are small decoder-only transformers executed in `f32`, batch 1, on one Apple GPU; they differ only in operation and shape surface. The earlier generic small-Llama option is superseded by the exact dense Qwen checkpoint because Qwen covers the same RMSNorm, SwiGLU, RoPE, GQA and ordinary KV-cache families while adding Q/K normalization and an independent 128-wide head dimension, and it has direct local Candle evidence.

| | Option A — GPT-2-class (LayerNorm, GELU, learned absolute positions, multi-head attention) | Option B — pinned Qwen3-0.6B-Base (RMSNorm, Q/K RMSNorm, SwiGLU, RoPE, GQA) |
| --- | --- | --- |
| **Enables** | The smallest operation surface that still reaches every rung, so L2 derives the fewest families and L3–L6 are proven against the minimum. | A reproducible current dense decoder whose exact config stresses per-head normalization, GQA projection asymmetry and head dimensions independent of `hidden_size / heads`, while reusing the Qwen/Candle lineage already measured locally. |
| **Prevents** | Nothing architecturally, but it proves the ladder against an architecture largely superseded in practice, so a later move to a current model reopens L2 for RoPE, RMSNorm, gated feed-forward, GQA and Q/K normalization. | Nothing architecturally, but it adds those current operation and shape families before the minimum GPT-2 contraction/attention path is proven. |

**Recommendation: Option B, the pinned Qwen3-0.6B base checkpoint.** L2's derived surface is inherited by every rung above it, so re-deriving RMSNorm, RoPE, SwiGLU, GQA and Q/K normalization later is the expensive mistake. The checkpoint is still a conventional dense transformer: it does not force MoE, vision, hybrid recurrence, sliding-window attention, quantization, chat templates, sampling or thinking-mode semantics. The counterpoint is real: GPT-2 makes L3 and L4 strictly easier to specify and failures easier to attribute.

**This was a product choice, not a correctness one** — both options were executable and neither was blocked by evidence. Tom delegated the final selection on 2026-07-30, and the outcome above records it.

**The activation trigger has fired.** `scope-optimized-metal-lm-inference` is `done`, so the "do not start this before its trigger" instruction recorded below is satisfied and no longer a stop sign; this ticket is waiting on the answer above and on nothing else.

**One thing the decision did not settle by itself.** The exact checkpoint's configuration — layer count, head count, head dimension, context length, vocabulary — had to be read from the actual model config before L2 could derive shapes from it. The Qwen assessment below records those facts from the pinned revision; they must not be taken from memory.

## Qwen assessment added 2026-07-30

The original elimination did not assess Qwen explicitly, so its claim that only GPT-2 and a generic small Llama survived was incomplete.

**Fact — exact dense candidate.** `Qwen/Qwen3-0.6B-Base` revision `da87bfb608c14b7cf20ba1ce41287e8de496c0cd` declares 28 layers, hidden size 1024, intermediate size 3072, 16 query heads, 8 KV heads, head dimension 128, pre-RMSNorm and per-head Q/K RMSNorm, SwiGLU, RoPE theta 1,000,000 with no scaling, tied embeddings, no attention bias, no sliding window, and greedy base generation. Its 596,049,920 BF16 checkpoint parameters widen exactly to 2,384,199,680 bytes of F32 weights before runtime storage. The checkpoint is Apache-2.0. The model artifact, config, tokenizer, tensor inventory and reference implementation revision must be pinned by digest before L2 derives the graph.

**Inference — why dense Qwen subsumes generic Llama here.** Qwen3-0.6B exercises the same semantic families as the former small-Llama candidate and adds Q/K RMSNorm plus a projection/head shape that cannot be inferred from hidden size divided by head count. Those additions strengthen the shape and operation proof without adding a new state subsystem or architecture rung.

**Fact — exact local lineage.** `../lmbrrr` at `75ec511c` exercised the 0.8B Qwen3.5 text tower inside `openbmb/MiniCPM-V-4.6` snapshot `8169864629825dc1d755a5aa1cd8b5935dcbc83f` and pins the Candle fork at `cd2499cceae27a2b1192d7a89c123597479adf3a`. That work produced retained correctness and performance evidence for RMSNorm, SwiGLU, Q/K normalization, GQA, partial RoPE, Gated DeltaNet, dual state/cache families, chunk verification and rollback. The reusable dense operations strengthen Qwen3-0.6B as the first workload; the hybrid-only work does not justify silently adding recurrence to the dense ladder.

**Eliminated as the first workload — Qwen3.5-0.8B.** The official sub-1B Qwen3.5 model is a vision-conditioned hybrid with 18 Gated DeltaNet and 6 gated full-attention layers, partial RoPE, recurrent and convolution state, KV state and MTP. Even a language-only route would require new recurrent-state semantics and physical contracts before the ordinary dense decoder path exists. It remains valuable precisely because `lmbrrr` makes those requirements concrete, so `exercise-qwen35-hybrid-text-tower-after-the-dense-vertical` preserves it as the second architecture-stress workload after dense model qualification.

**Correctness consequence.** Token-sequence equality alone is insufficient because materially wrong logits can retain the same argmax. The workload must preserve fixed prompt token IDs; compare prefill and every decode-step logits under tolerances derived from the effective F32 numerical contract before results are observed; and additionally require greedy-token equality, an explicit tie policy, and EOS-or-fixed-budget termination.

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

## Completion profile (2026-07-30 audit)

Close this ticket on two bounded rows rather than one arbitrary universal sequence cap. The conformance row uses a small fixed token-ID prompt and short fixed decode budget so every prefill and decode-step logit can be retained and reproduced. The representative benchmark row uses a separately stated prompt/context/decode matrix within the checkpoint's declared `max_position_embeddings = 32768`; broader performance qualification remains L8.

Retain a manifest for the exact model config, tokenizer, weight shards/index, tensor inventory, license, and reference implementation at immutable revisions with SHA-256 digests. State the 596,049,920-parameter F32 weight budget of 2,384,199,680 bytes separately from KV-cache and peak workspace arithmetic. Do not download a roughly 1.19 GB checkpoint implicitly under this documentation ticket without first recording its repository storage policy and reproducible acquisition route.

The correctness oracle compares logits after prefill and every decode step under a predeclared effective F32 numerical policy; greedy-token equality, tie handling, and EOS-or-fixed-budget termination are additional observables rather than substitutes. Tolerances must come from the selected numerical realization/reference comparison contract rather than an ad hoc model-level constant.

Replace the stale MSL 3.1 target row in the older research section with the current qualified `apple9-f32-unified-msl4-macos26` authority: Apple9/macOS/F32, MSL 4.0, offline `metalfe-32023.883`, and runtime/pipeline compiler `metalfe-32023.921`. Live execution and delivered-numerics claims remain host-qualified.

## Delivery (2026-07-31)

**The durable profile is [`docs/research/program-planning/first-metal-lm-workload.md`](../docs/research/program-planning/first-metal-lm-workload.md).** It records both bounded rows, the pinned identity manifest with per-file SHA-256 digests and the acquisition policy, the configuration and tensor facts, the F32 memory arithmetic, the correctness oracle, the target qualification row, the traced operation and shape surface, and the exclusion table. The sections below stay as the selection evidence and elimination record they are; the profile does not restate them.

**Fact — every architecture number in this ticket was recomputed from the pinned revision rather than carried forward.** `config.json` at `da87bfb608c14b7cf20ba1ce41287e8de496c0cd` (SHA-256 `504a6b58c4271583724e66584b6b7698aea18450209df6b2f7582df0e89cee59`) confirms 28 layers, hidden 1024, intermediate 3072, 16 query heads, 8 KV heads, head dimension 128, `rms_norm_eps` 1e-06, `rope_theta` 1000000 with `rope_scaling: null`, tied embeddings, no attention bias, no sliding window, vocabulary 151936, and `max_position_embeddings` 32768. The checkpoint's own safetensors header, read by two HTTP range requests totalling about 34 KiB, declares 310 BF16 tensors summing to 596,049,920 parameters — 2,384,199,680 bytes widened to F32 — and its offsets account for the file exactly, so the parameter and byte figures this ticket asserted are now reproduced from the artifact instead of restated.

**Fact — the checkpoint was not downloaded.** Its SHA-256 `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba` is the repository API's Git-LFS object id, recorded as an identity to verify on acquisition rather than a digest computed here; every other manifest row was hashed from bytes actually fetched, with each size cross-checked against the API-reported size. `retain-the-qwen-conformance-reference-logit-fixture` owns converting the one API-sourced digest into a locally reproduced fact, and no digest anywhere in the profile is invented.

**Fact — the stale MSL 3.1 target row was already corrected before this delivery.** `git log -L 154,154:tickets/define-first-metal-lm-workload.md` shows commit `53c702d` replacing the old "qualified row in the Apple GPU numerical behaviour record" wording with the `apple9-f32-unified-msl4-macos26` authority, so the completion profile's instruction was satisfied by that commit rather than by this one. The identifiers were re-verified against the retained records before the profile repeated them: [`spikes/apple-targets/results/2026-07-30-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`](../spikes/apple-targets/results/2026-07-30-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv) and its `exhaustive` sibling, described by the unified MSL 4 replay section of [`docs/research/apple-targets/numerical-behaviour.md`](../docs/research/apple-targets/numerical-behaviour.md), which names schema `tiler.apple-numerical-behaviour/v7`, profile identity `apple9-f32-unified-msl4-macos26`, MSL 4.0 on both paths, offline `metalfe-32023.883`, and source-JIT `metalfe-32023.921`.

**One thing the profile deliberately does not supply: the comparison tolerance.** The conformance level is `bounded error`, and the bound cannot be composed from per-operation tolerances — the region-accuracy record establishes why a graph budget is not a sum. Writing a model-level constant now would be exactly the ad hoc threshold `design-model-level-qualification-and-optimization` forbids, so the profile states the oracle's five observables, marks the numeric bound `Unknown`, and hands the derivation to that rung with a measurable procedure attached rather than filing a competing ticket for a threshold it already owns.

## Implementation keys

Record the two bounded workload rows, immutable model/tokenizer/weight/reference identities and acquisition policy, exact operation and shape surface, F32 memory arithmetic, per-step correctness oracle, target qualification, exclusions, and dependency-ordered follow-ups. Keep diagnostic GPT-2 fixtures subordinate to the representative Qwen workload and leave Qwen3.5 hybrid recurrence to its named downstream stress ticket.

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

## Graph maintenance

- Update the L2 operation-and-shape derivation from the pinned Qwen manifest rather than from remembered architecture defaults.
- Link every newly exposed subsystem to an existing owner or file a bounded ticket with evidence and a reconsideration trigger; do not add implementation work to this research ticket.
- Keep the Qwen3.5 hybrid text-tower ticket downstream of dense model qualification and preserve GPT-2 only as a diagnostic fixture family.

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
- **Target profile:** the qualified `apple9-f32-unified-msl4-macos26` authority—Apple9/macOS/F32, MSL 4.0, offline `metalfe-32023.883`, and runtime/pipeline compiler `metalfe-32023.921`. **Which claims need a live device:** every execution and numerical-delivery claim. Compile-side claims—emitted operations, module options, artifact identity—do not.
- **Success measures at the model boundary:** predeclared comparison of reference logits at prefill and every decode step under the effective F32 numerical contract, greedy token-sequence equality with an explicit tie policy for a fixed prompt, and decode latency per token plus prefill latency, both as min-of-N on a quiet host. Not a throughput figure, which would need batching.
- **Excluded, per the ladder's deferral table:** training, distributed execution, speculative decoding, unsupported architectures, unbounded dynamic shapes.

### The one genuine choice, now resolved

Two exact candidates survived the corrected elimination and encoded different valid priorities: a GPT-2-class checkpoint and pinned Qwen3-0.6B-Base. Both are small decoder-only transformers in `f32`; they differ in operation and shape surface. The accepted Qwen selection, its enables and prevents, the recommendation, and the counterpoint are stated at the top of this file.
