---
id: assemble-the-decoder-layer-program
title: Assemble the complete decoder-layer program
status: todo
priority: p1
dependencies: [assemble-the-causal-self-attention-block-program, admit-the-silu-activation-family, admit-the-sequence-extension-concatenate-family, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, design-attention-program-vertical, design-autoregressive-state-and-kv-cache, widen-the-deterministic-budgets-to-the-decoder-layer-program]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, transformer, program, language-model]
---
## User-visible outcome

One complete decoder layer of the pinned checkpoint — attention, MLP, both residuals, and the two cache extensions — is a single verified semantic program that reference-evaluates against the pinned reference.

## Required content

The program [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) calls P2:

- **Eighteen ordered inputs.** [L4's twelve](../docs/research/program-planning/first-attention-program-vertical.md) — `x`, `w_input_layernorm`, `W_q`, `W_k`, `W_v`, `w_q_norm`, `w_k_norm`, `cos`, `sin`, `rope_sign`, `mask`, `W_o` — plus [L5's two](../docs/research/runtime/autoregressive-state-and-kv-cache.md) `k_cache` and `v_cache`, plus the MLP's `w_post_attention_layernorm`, `W_gate`, `W_up`, and `W_down`.
- **Three ordered named outputs.** `h_out [T, 1024]`, `k_rope [8, S, 128]`, `v_heads [8, S, 128]`.
- **The MLP half is `down(silu(gate(x)) * up(x))`** over `[T, 3072]` intermediates, which L2 derived introduces no family its constituents do not, plus the second residual add.
- **The two concatenations are at the block boundary**, exactly where L5 places them: L4's steps 13 and 14 each feed one, and the concatenation's result is the retained output the score and value contractions read.

## The counts this ticket must produce

They are an input to [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md) and must be measured rather than estimated: the program's exact `value_count()`, `operation_count()`, and `input_count()`. The L6 record's derived floors are at least fifty-one occurrences and at least twenty-one boundary values; a smaller number is a real result and a larger one is too, and either replaces the floor.

## Closes when

The program verifies, reference-evaluates against the pinned reference at the C1 prefill shape and at one decode shape, the three outputs are ordered and named, and the exact counts above are recorded in this ticket's outcome.

## Do not

Do not compile it. This ticket assembles and reference-evaluates; the deterministic budgets refuse the compilation and widening them is a separate, identity-moving decision that belongs to Tom.
