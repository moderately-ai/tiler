---
id: stage-contractions-inside-whole-program-reference-evaluation
title: Reach a whole-program reference evaluation whose contractions exceed the per-call work bound
status: in-progress
priority: p2
dependencies: []
related: [assemble-the-causal-self-attention-block-program, admit-the-attention-contraction-structures, retain-the-c1-attention-block-conformance-evidence, integrate-the-attention-block-into-the-runtime]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, evaluation, contraction, language-model]
claimed_from: todo
assignee: agent-staged-eval
lease_expires_at: 1785694007
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

## Outcome, 2026-08-02

**The third candidate survived, and the first two were eliminated together rather than separately, because they are one design.** Staging inside the registered operation (candidate one) is *necessary* whichever way this goes — a 20,971,520-step fold has to be spent somehow, and only windowing spends it without moving a number. What candidates one and two actually disagree about with candidate three is not the mechanism but **what authorizes the extra work**.

- **A program-total accumulated resource (candidate two) was eliminated on two counts.** It refuses programs that pass today: whatever constant is chosen, a program whose contractions total more than it stops evaluating, and today the total is unbounded while each occurrence is bounded — so this widens nothing and narrows something nobody asked to narrow. And the constant would have no derivation. `EvaluationRetention`'s three existing accumulators reuse the *same* constants as the per-tensor bounds, so they introduce no new number; a fold-step total must be larger than the per-call bound to serve any purpose at all, and "larger" has no principled value. That is precisely the "constant nobody re-derives" the staged contraction's own documentation was written against.
- **Candidate one alone was eliminated because it deletes a check.** Staging unconditionally with no replacement bound leaves an occurrence able to walk `output_count * contracted_count` steps with `output_count` and `contracted_count` each bounded at 16,777,216 — about 2.8e14 steps, roughly a month of host time at the measured 9 ns per step. The ticket says as much: the program would need a budget where the operation used to carry one.
- **Candidate three survived, and its stated cost turned out not to apply.** The ticket priced it as "two evaluation paths whose agreement then has to be evidenced". There is one path: `contract_operands` and `StagedStrictTensorContractionF32` already share `ContractionFold::evaluate_outputs`, and both now take their window width from one new private `ContractionFold::window_output_count`. A fourth candidate — a `StagedProgramEvaluation` the caller loops, mirroring `StagedIndexRegionEvaluation` — was eliminated on cost: suspending inside an occurrence would require *every* `ReferenceOperation` to become resumable, for one family's benefit.

**What landed.** `ReferenceEvaluator` carries a per-occurrence iteration-step allowance, defaulting to `MAX_REFERENCE_TENSOR_ELEMENTS`, delivered to each registered occurrence through `ReferenceEvaluationRequest` — the same shape `ScalarReferenceRequest::conformance` already uses to carry an evaluator-level term to a callback. `contract_operands` refuses above the allowance and otherwise folds in windows sized by the unchanged per-window bound. **`MAX_REFERENCE_TENSOR_ELEMENTS` did not move**, and neither did the index-region oracle's separate `MAX_EVALUATION_STEPS`, which is a different bound on a different oracle that happens to carry the same number.

**The refusal still fires, watched on both sides of the number.** A default evaluator declines the C1 block with the byte-identical `reference operation iteration space has 20971520 steps, exceeding 16777216`, and an evaluator told `C1_LARGEST_FOLD - 1` declines it naming that stated number instead. Both are asserted at the C1 extents in `the_reference_work_bound_refuses_the_c1_projections`, and the same pair is asserted on `w_prefill_q`'s operands in `contraction_profile_cells.rs`.

**The C1 block evaluates at 1,024.** `the_block_evaluates_end_to_end_against_an_independent_recomputation` runs at the row's own extents with nothing reduced, at zero differing elements on all three outputs against the independent recomputation, with the repeat-tile perturbation still live. `EVALUATED_HIDDEN` is gone. **Measurement — Apple M4 Max, 2026-08-02, dev profile:** that test is 1.81 s (from about 0.9 s at 512), and `the_reference_work_bound_refuses_the_c1_projections` 0.90 s.

**Bit-identity, where it can be stated.** There is no unstaged path for a 20,971,520-step fold, so the comparison is against a *different partition* and against a device. `a_whole_program_evaluation_reaches_the_cell_its_default_evaluator_refuses` drives `w_prefill_q` through `ReferenceEvaluator::evaluate` in the evaluator's two windows of 16,384 outputs, reproduces the retained Apple M4 Max `result_sha256`, and requires element-for-element equality with a seven-slab partition of the identical fold — 0.49 s. At smaller folds where both a single window and several partitions run, `slab_boundaries_do_not_change_any_folded_value` is unchanged and still compares five widths against the whole-program evaluator.

**Watched failing.** `evaluate_outputs(contract, first_output, outputs)` was perturbed to `first_output.min(1)` and `a_fold_over_one_window_is_walked_in_several_when_the_allowance_admits_it` failed at the first element of the second window; the perturbation was reverted and the test passes. That test's expectation is the linear index itself, so a window that starts at the wrong offset, runs short, or repeats disagrees at the element where it went wrong.

**New public items, for Tom.** Three additive methods on existing public types in `tiler-reference`, no new type, trait, namespace, or error variant, and no changed existing signature:

- `ReferenceEvaluator::with_iteration_step_allowance(self, usize) -> Self`
- `ReferenceEvaluator::iteration_step_allowance(&self) -> usize`
- `ReferenceEvaluationRequest::iteration_step_allowance(self) -> usize`

None matches an ADR 0075 always-ask category (no new publicly reachable namespace, no new public trait, no breaking change to an existing public signature, no `pub(crate)` promotion), and none matches a no-approval category verbatim either — the record does not name "a new method on an existing public type". Recorded for Tom rather than self-accepted.

**No identity moved.** The registered contraction capability, its revision 7, its signature, and the frozen registry's canonical identity bytes are untouched: the allowance changes which programs are *accepted*, never a value, so nothing output-affecting changed and no pin was rebaselined.

**Filed.** [`requalify-the-compiler-statement-of-the-reference-work-bound`](requalify-the-compiler-statement-of-the-reference-work-bound.md) — `crates/tiler-compiler/src/governed/contraction_conformance.rs` describes the constant as bounding "a single fold", which is now the wrong description even though every assertion in that file still holds. Out of this ticket's scope.
