---
id: stage-contractions-inside-whole-program-reference-evaluation
title: Reach a whole-program reference evaluation whose contractions exceed the per-call work bound
status: todo
priority: p2
dependencies: []
related: [assemble-the-causal-self-attention-block-program, admit-the-attention-contraction-structures, retain-the-c1-attention-block-conformance-evidence, integrate-the-attention-block-into-the-runtime]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, evaluation, contraction, language-model]
---
## User-visible outcome

`ReferenceEvaluator::evaluate` reaches a semantic program containing a contraction whose fold exceeds one call's work bound, so a whole-block language-model program reference-evaluates at the pinned checkpoint's own model dimension rather than only at a reduced one.

## Why this exists

**Fact — the bound, and the two occurrences that meet it.** `contract_operands` in `crates/tiler-reference/src/contraction.rs` refuses a fold of more than `MAX_REFERENCE_TENSOR_ELEMENTS` (16,777,216) multiply-accumulate steps, before the loop and under its own `IterationStepsExceeded` variant. At the C1 conformance row's prefill shape the causal self-attention block's query projection is `10 * 2048 * 1024` and its output projection `10 * 1024 * 2048` — **20,971,520 steps each** — so both are refused.

**Measurement — watched, not derived.** `the_reference_work_bound_refuses_the_c1_projections` in `crates/tiler-reference/tests/causal_self_attention_block.rs` builds the block at the C1 extents and evaluates it, and the refusal reads:

```text
reference capability revision 7 from tiler::standard-reference@7 for
tiler::strict-tensor-contraction-f32@1 failed: reference operation iteration
space has 20971520 steps, exceeding 16777216
```

**Inference — this is a bound rather than a defect, and it must not be moved casually.** The bound is what a whole-program evaluation is deliberately held to; `contraction.rs` says so at the site and `contraction_profile_cells.rs` says so in its module documentation, which reaches four L3 profile cells — `w_prefill_q` among them, at these exact extents — through `StagedStrictTensorContractionF32` **without moving it**, each slab passing the same test the unstaged path applies. Raising the constant would be the cheap option that silently widens what every other whole-program evaluation may do.

**Fact — what is already evidenced, so this ticket is not blocking correctness.** The two refused occurrences are ordinary structure-1 contractions. `w_prefill_q` is exactly this block's operation 2 at this row and is reproduced through the staged evaluator against a digest an Apple M4 Max produced. The attention-specific steps — the score contraction, the scale, the mask add, the softmax, and the value contraction — are all far under the bound and evaluate at the exact C1 shape today. What is missing is only the *end-to-end* evaluation at the C1 row's own 1,024-wide model dimension; the block currently evaluates end to end at 512.

## What this needs to decide

The candidates, none eliminated here, and the elimination is part of the work:

- **Stage inside the registered operation.** The evaluator's contraction capability walks output slabs internally, each under the existing per-call test, so no bound moves and no caller changes. Cost: the registered operation stops being a single bounded call, which is the property the current bound expresses, and a whole-program evaluation gains an unbounded total — so the *program* would need a budget where the *operation* used to carry one.
- **A budget on the evaluation rather than on the call.** `EvaluationRetention` already accumulates bytes, elements, and components across a whole program; a fourth accumulated resource — fold steps — would put the limit where the total actually is, and let one large contraction through while still refusing an unbounded program. This is the shape most consistent with what the existing retention does.
- **An explicit opt-in on the evaluator.** A caller that has decided to pay the time asks for it, which is what the staged API already expresses at the single-contraction level. Cost: two evaluation paths whose agreement then has to be evidenced.

Whichever survives, the refusal must stay reachable and explainable: a program that would run for an unbounded time must still be declined with the exact step count and the exact budget, never accepted silently.

## Closes when

A semantic program whose contraction fold exceeds one call's bound reference-evaluates, the C1-shape whole-block evaluation in `causal_self_attention_block.rs` runs at the checkpoint's own 1,024-wide model dimension with its measurement-boundary note removed, and a program whose total exceeds whatever budget survives is still refused with a watched, quoted diagnostic.
