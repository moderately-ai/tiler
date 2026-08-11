---
id: requalify-the-compiler-statement-of-the-reference-work-bound
title: Requalify the compiler's statement of the reference work bound
status: in-progress
priority: p3
dependencies: []
related: [stage-contractions-inside-whole-program-reference-evaluation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, reference, contraction]
claimed_from: todo
assignee: terra-reference-bound
lease_expires_at: 1786418733
---
## User-visible outcome

`crates/tiler-compiler/src/governed/contraction_conformance.rs`'s module documentation describes the reference work bound as it now is, so a reader reasoning from it does not conclude that a fold above 16,777,216 steps is unreachable through `ReferenceEvaluator`.

## Why this exists

**Fact — corrected 2026-08-10 after re-reading the source at claim
`d8f4cfca07c9bc9f8a71a3fa172eb9501bac349f`.** The module still says
`MAX_REFERENCE_TENSOR_ELEMENTS` bounds a *single* fold at 16,777,216 steps,
but that is now stale: the constant bounds one window. `contract_operands`
refuses `output_count * contracted_count` above the evaluator-carried
per-occurrence allowance under `IterationStepsExceeded`; the default allowance
is the constant.

**Fact — what `stage-contractions-inside-whole-program-reference-evaluation` changed.** That constant now bounds a single *window* — the steps one uninterrupted walk of a contraction's iteration space may cost — and `contract_operands` refuses above the iteration-step allowance its evaluator carries, which is that constant unless a caller states another. A fold over one window is spent as several windows rather than refused.

**Fact — corrected 2026-08-09.** `governed::contraction_conformance` does not call `ReferenceEvaluator::standard()`; both relevant sites construct `ReferenceEvaluator::new(FrozenReferenceRegistry::standard(...))`. That constructor still uses the default per-occurrence allowance, and neither site calls `with_iteration_step_allowance`, so every refusal this module observes is unchanged and every assertion in the file still holds. What has stopped being true is the general claim about the constant: a reader deriving "the four prefill cells can never come through the evaluator" would now be wrong.

The out-of-scope status is why this is filed rather than fixed: the change that made the sentence stale held `implementation/reference` alone.

## Closes when

The passage distinguishes the per-window bound from the per-occurrence allowance, states that this module constructs the default allowance and why that keeps its four refusals, and no assertion changes — the refusals it watches are the default evaluator's and must stay watched.
