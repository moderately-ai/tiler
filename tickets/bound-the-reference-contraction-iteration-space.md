---
id: bound-the-reference-contraction-iteration-space
title: Name the reference contraction's iteration-space bound in its own diagnostic
status: todo
priority: p3
dependencies: []
related: [admit-the-contraction-normative-reference]
scopes: [implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, reference, numerics, contraction]
---
The contraction reference bounds its multiply-accumulate work — `output_count * contracted_count`, which is larger than either operand and bounded by neither tensor limit the operands already passed — and reports the refusal as `ReferenceOperationError::ShapeTooLarge`, whose documented meaning is that *shape arithmetic* exceeded host limits.

The refusal is correct and fails closed. The diagnostic is not: a caller reading `ShapeTooLarge` learns that a shape was too large, when what happened is that a well-formed pair of in-bounds operands named an iteration space this host oracle will not walk. In a crate whose contract is explainable refusal, that gap is worth closing.

## Required delivery

A typed variant naming the bounded resource and carrying its limit and first rejected size, on the pattern `OutputElementsExceeded` already sets, reached by the contraction fold and by anything else that later bounds iteration work rather than storage. `ReferenceOperationError` is `#[non_exhaustive]`, so the variant is additive; it is still a public boundary and goes to Tom with the rest of that batch.

## Closes when

A contraction whose iteration space exceeds the bound is refused under a variant that names the iteration space, with a regression that watches the old and new bounds discriminate — and the existing `ShapeTooLarge` sites keep their meaning rather than being widened to absorb it.
