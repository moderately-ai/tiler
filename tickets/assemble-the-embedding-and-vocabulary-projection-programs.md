---
id: assemble-the-embedding-and-vocabulary-projection-programs
title: Assemble the embedding and vocabulary-projection programs
status: todo
priority: p1
dependencies: [admit-a-storage-carrier-for-integer-program-inputs, admit-the-rms-normalization-family]
related: [design-model-ingestion-and-complete-execution, admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-the-contraction-semantic-profile, project-only-the-final-position-logits]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, gather, logits, language-model]
---
## User-visible outcome

The two ends of the model exist as programs: token IDs become a residual stream, and a residual stream becomes logits — so what the consumer hands over is token IDs and what it receives is logits.

## Required content

The two programs [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) calls P1 and P3.

- **P1 — embedding gather.** Inputs `token_ids [T]` at the admitted integer identity and `W_embed [151936, 1024]` F32; output `x0 [T, 1024]`; one operation.
- **P3 — final norm and vocabulary projection.** Inputs `h [T, 1024]`, `w_norm [1024]`, and the same `W_embed`; output `logits [T, 151936]` F32; two operations, `model.norm` then `Contract td,od->to`.
- **The tied matrix is one bound value read by both.** The checkpoint carries no `lm_head.weight`, so the same `[151936, 1024]` tensor is the gather's table and the projection's weight. Both bindings are read-only, so no aliasing question arises; what must be recorded is that binding two copies costs 622,329,856 bytes and nothing below the consumer can detect it.
- **The token-ID bounds obligation has a named enforcement boundary.** An index outside `0..151936` refuses or is validated where [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) fixes it. It never clamps, never wraps, and never reads out of bounds.
- **The logits contract is what the output means.** Unnormalized, every position, no softmax, no scale, no vocabulary mask, no argmax, no token. The final-position mode is a different program shape and belongs to [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md).

## Both programs fit today's budgets, and that is worth checking rather than assuming

P1 declares two inputs and P3 three, so `check_budget("buffers", 4, 4.max(input_count + 1))` passes for both, as do the value and operation budgets. Verify it rather than inherit it — if either fails, the budget widening ticket's sizing evidence changes.

## Closes when

Both programs verify and reference-evaluate against the pinned reference at the C1 prefill shape and at one decode shape, the tied matrix is demonstrably one value in both, an out-of-range token ID reaches its named refusal, and the exact budget-relevant counts are recorded.
