---
id: integrate-the-autoregressive-decode-loop
title: Integrate the autoregressive decode loop behind one cursor authority
status: todo
priority: p1
dependencies: [execute-the-decode-step-path, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, admit-a-position-selecting-slice-for-the-rotary-table, test-the-autoregressive-state-failure-cases]
scopes: [implementation/candle, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, decode, consumer-neutral, language-model, class-conformance-fixture]
---
## User-visible outcome

A consumer runs C1 end to end — one prefill and eight decode steps — and every position-dependent input is derived from **one** cursor, so the two ways a caller can get position wrong become one.

## Required behaviour

- One cursor authority derives, per execution: the cache extent `C`, the rotary table rows `C … C + T`, and the causal mask. A consumer that states position twice can state it inconsistently, and [the L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md) establishes that **no layer of Tiler can detect the inconsistency** — a wrong `cos`/`sin` row has the same shape, dtype, accessible range, and launch geometry as the right one, so the envelope decodes, the guard holds, the byte comparison passes, and the result is a plausible logit vector with a wrong argmax.
- Termination is EOS token 151643 or the row's fixed eight-step budget, and never an implicit stop.
- A failed invocation returns Tiler's stage-named typed failure with outputs withheld; the driver stops the loop attributing that failure to the step it already knows (routing is synchronous), does not continue from its pre-failure tensors, does not treat a post-commit failure as retriable in place, and never silently skips the step. This product rule stops the loop on any refusal, including pre-commit; routing-legal re-preflight of an uncommitted step is not used here. **Correction — 2026-08-10.** Prior wording "typed error naming the step" fused Tiler's stage-named failure with the driver's loop ordinal; step attribution is the driver's composition over a synchronous route result, not a Tiler-public ordinal field. *Corrected 2026-08-04 under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md):* this bullet read "A poisoned state stops the loop". Tiler holds no state to poison — the typed failure and the withheld outputs are Tiler's, and refusing to continue is the driver's, which is exactly the obligation this ticket's single cursor authority already carries.

## Non-goals

Sampling policy beyond greedy, batching, prefix sharing, and speculative decoding. The tie rule is already declared by L1 and is not reopened here.

## Closes when

The loop reproduces C1's eighteen-position run against one attention block, the cursor is the only place position is stated, and a deliberate perturbation of the cursor produces a *consistently* wrong run rather than an inconsistent one — which is the property this ticket buys and the limit of what it can buy.
