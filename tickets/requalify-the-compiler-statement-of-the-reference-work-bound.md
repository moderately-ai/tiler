---
id: requalify-the-compiler-statement-of-the-reference-work-bound
title: Requalify the compiler's statement of the reference work bound
status: todo
priority: p3
dependencies: []
related: [stage-contractions-inside-whole-program-reference-evaluation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, reference, contraction]
---
## User-visible outcome

`crates/tiler-compiler/src/governed/contraction_conformance.rs`'s module documentation describes the reference work bound as it now is, so a reader reasoning from it does not conclude that a fold above 16,777,216 steps is unreachable through `ReferenceEvaluator`.

## Why this exists

**Fact — the sentence, at `crates/tiler-compiler/src/governed/contraction_conformance.rs:45-48` as of `5d35f46`.** "`MAX_REFERENCE_TENSOR_ELEMENTS` bounds a *single* fold at 16,777,216 steps, and `contract_operands` still refuses `output_count * contracted_count` above it under `IterationStepsExceeded`."

**Fact — what `stage-contractions-inside-whole-program-reference-evaluation` changed.** That constant now bounds a single *window* — the steps one uninterrupted walk of a contraction's iteration space may cost — and `contract_operands` refuses above the iteration-step allowance its evaluator carries, which is that constant unless a caller states another. A fold over one window is spent as several windows rather than refused.

**Inference — the sentence is true of this crate and misleading about the reference.** `governed::contraction_conformance` builds its evaluator with `ReferenceEvaluator::standard()` and states no allowance, so every refusal it observes is unchanged and every assertion in the file still holds. What has stopped being true is the general claim the sentence makes about the constant: a reader deriving "the four prefill cells can never come through the evaluator" from it would now be wrong, and the file's neighbouring paragraphs invite exactly that derivation because they are about which cells are reachable.

The out-of-scope status is why this is filed rather than fixed: the change that made the sentence stale held `implementation/reference` alone.

## Closes when

The passage distinguishes the per-window bound from the per-occurrence allowance, states which one this crate is held to and why (it states no allowance), and no assertion in the file changed — the refusals it watches are the default evaluator's and must stay watched.
