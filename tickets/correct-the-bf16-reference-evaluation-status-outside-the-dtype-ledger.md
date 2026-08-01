---
id: correct-the-bf16-reference-evaluation-status-outside-the-dtype-ledger
title: Correct the BF16 reference-evaluation status outside the dtype ledger
status: todo
priority: p1
dependencies: []
related: [evaluate-bf16-reference-semantics, register-the-bf16-semantic-operation-signatures]
scopes: [contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, documentation]
---
## User-visible outcome

Two documents outside `evaluate-bf16-reference-semantics`'s scope still assert that `tiler-reference` refuses every BF16 key. A reader planning the next BF16 rung would conclude the oracle does not exist, and would either duplicate it or gate work on it that is no longer gated.

## What is stale, exactly

**Fact**, at `evaluate-bf16-reference-semantics`'s landing commit:

- `docs/roadmap.md`, the reduced-precision-float row of the R-rung table (search `R4 through R7 are unmoved`): "the standard reference provider registers no BF16 evaluator and refuses each key with `MissingCapability`". R4 moved; R5 through R7 did not, and the sentence has to separate them. The same row's closing sentence gates BF16's remaining rungs on `evaluate-bf16-reference-semantics` (R4), which is now satisfied.
- `docs/research/numerics/bf16-computation-accumulator-and-conversion.md`, the maturity table row `Defined but unimplemented`: "`MissingCapability`, which is exactly what `ReferenceEvaluator::standard()` returns for the three landed BF16 keys today". The example is no longer that; the *class* is still a real one and needs a different instance, not deletion.

`docs/dtype-support.md`'s BF16 `Reference evaluation` cell and its family note were moved by the landing ticket and are not part of this one.

## Why this is a separate ticket

`docs/roadmap.md` was held under `contracts/navigation` by a concurrently live ticket when the reference landed, and `docs/research/numerics/**` is a scope the landing ticket did not hold. Both edits are one paragraph each and neither is urgent enough to justify a cross-scope edit under someone else's claim.

## Closes when

Both sentences state the layer that actually moved and the layers that did not, the roadmap row's R-rung claim separates R4 from R5 through R7, and the research document's `Defined but unimplemented` row names a class instance that is still true.
