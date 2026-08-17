---
id: assemble-the-embedding-and-vocabulary-projection-programs
title: Assemble the embedding and vocabulary-projection programs
status: in-progress
priority: p1
dependencies: [admit-a-storage-carrier-for-integer-program-inputs, admit-the-rms-normalization-family, reclassify-language-model-work-as-a-conformance-track, admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-the-contraction-semantic-profile]
related: [design-model-ingestion-and-complete-execution, project-only-the-final-position-logits]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, gather, logits, language-model, class-conformance-fixture]
claimed_from: todo
assignee: worker-lm-boundaries
lease_expires_at: 1787004728
---
## User-visible outcome

The two ends of the model exist as programs: token IDs become a residual stream, and a residual stream becomes logits — so what the consumer hands over is token IDs and what it receives is logits.

## Required content

The two programs [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) calls P1 and P3.

**Graph correction — 2026-08-09.** The ticket originally left the gather and contraction semantic families under `related` even though P1 and P3 cannot be constructed without them. Both have landed and are now dependencies: [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owns P1's operation and bounds refusal, and [`admit-the-contraction-semantic-profile`](admit-the-contraction-semantic-profile.md) owns P3's projection operation. The outstanding integer-storage decision remains the only non-terminal family/carrier dependency.

**Graph repair — 2026-08-10.** Gather and contraction remain in `dependencies` only; they were dropped from `related` so the same ids are not double-listed once required.

- **P1 — embedding gather.** Inputs `token_ids [T]` at the admitted integer identity and `W_embed [151936, 1024]` F32; output `x0 [T, 1024]`; one operation.
- **P3 — final norm and vocabulary projection.** Inputs `h [T, 1024]`, `w_norm [1024]`, and the same `W_embed`; output `logits [T, 151936]` F32. L6's abstract cut is two operations, `model.norm` then `Contract td,od->to`. A constructible program must count the explicit `w_norm` widening (broadcast or reindex by `T`) before `rms_norm`, because `tiler::rms-norm-f32@1` refuses a `[1024]` weight against `[T, 1024]` with no implicit broadcast — so occurrence counts at C1 prefill and decode will be greater than two and must be recorded as measured.
- **The tied matrix is one bound value read by both.** The checkpoint carries no `lm_head.weight`, so the same `[151936, 1024]` tensor is the gather's table and the projection's weight. Both bindings are read-only, so no aliasing question arises; what must be recorded is that binding two copies costs 622,329,856 bytes and nothing below the consumer can detect it.
- **The token-ID bounds obligation has a named enforcement boundary.** An index outside `0..151936` refuses or is validated where [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) fixes it. It never clamps, never wraps, and never reads out of bounds.
- **The logits contract is what the output means.** Unnormalized, every position, no softmax, no scale, no vocabulary mask, no argmax, no token. The final-position mode is a different program shape and belongs to [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md).

## Both programs fit today's budgets, and that is worth checking rather than assuming

P1 declares two inputs and P3 three. Governed `buffers` is **30**, and `check_program_budgets` derives the buffers actual as `input_count + output_count * 4`, refusing only when the actual **exceeds** the limit (so P1's actual is 6 and P3's is 7 if each has one output). Value and operation budgets are similarly far inside today's governed limits. Verify the measured `input_count` / `operation_count` / `value_count` (and buffers actual if `verify_program` is exercised) rather than inherit them — if either program fails a real budget, the budget widening ticket's sizing evidence changes.

## Fact audit — 2026-08-10 at base `c99ac54950f2`

- The prior buffers claim `check_budget("buffers", 4, 4.max(input_count + 1))` never matched source (same defect class L6 already corrected under output-derived budgets). Implemented path: `check_program_budgets` / `DeterministicBudgets::governed` in `crates/tiler-compiler/src/request.rs`.
- P3 "two operations" is the L6 abstract cut only; constructible programs insert the explicit weight widening before RMS (see decoder-layer `widen_hidden_weight` pattern).

## Closes when

Both programs verify and reference-evaluate against the pinned reference at the C1 prefill shape and at one decode shape, the tied matrix is demonstrably one value in both, an out-of-range token ID reaches its named refusal, and the exact budget-relevant counts are recorded.
