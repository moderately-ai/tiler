---
id: correct-the-extrema-familys-identity-ground-and-name-its-padding-identity
title: Correct the extrema family's identity ground and name its padding identity
status: todo
priority: p2
dependencies: []
related: [derive-the-multi-round-two-level-reduction-composition]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, numerics, reductions, doc-claim]
---
## User-visible outcome

`ScalarProgram::StrictSerialMaximum`'s doc states a ground its own family refutes, and the next reader of it stops concluding that the extrema fold can never be padded.

## The defect, stated so it can be refuted in one command

**Fact.** `ScalarProgram::StrictSerialMaximum` (`crates/tiler-ir/src/schedule/model.rs`) reads: "**There is deliberately no empty-domain identity, and the omission is the contract rather than an oversight.** … the extrema families have no identity: no binary32 value `i` satisfies `Maximum(i, x) == x` for every `x`, because any candidate is itself a possible contributor."

**Fact.** `BinaryOp::F32Maximum` (`crates/tiler-ir/src/kernel/model.rs`) is "The IEEE 754-2019 `maximum` of two binary32 values … **The NaN-propagating extrema family, with `-0.0` ordered below `+0.0`**", which [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md) fixes.

**Inference — the conclusion stands and the stated ground is false.** `maximum(-inf, x) = x` for every binary32 `x`: for finite and infinite `x` because `-inf` is the order's minimum and `maximum(-inf, -inf) = -inf`; for `±0.0` because the family orders both above `-inf`; and for a NaN because the family propagates, with the fold's per-combine `CanonicalizeF32Nan` making the committed bits the same ones the unpadded fold commits. `0xff80_0000` is therefore a two-sided identity, and the sentence's reason — "any candidate is itself a possible contributor" — is an argument about an *empty-domain result* being indistinguishable from a real fold, which is a different claim from algebraic identity. [Numerical semantics](../docs/numerical-semantics.md) and [ADR 0022](../docs/decisions/0022-reduction-identities-and-initial-values.md) already separate empty result, algebraic identity, and replicable padding; the comment collapses them.

**Why it is load-bearing rather than a wording nit.** [The multi-round two-level reduction composition](../docs/research/scheduling/multi-round-two-level-reduction-composition.md) needs a padding identity for the extrema family, because a two-level composition pads at an imposed subgroup width and the family's non-emptiness argument — "a product of nonzero factors equalling a nonzero total forces every factor nonzero" — is stated for exactly covered splits and does not reach a padded one. A reader who carries the comment forward concludes the composition can never serve the softmax row maximum it exists for.

## Work

- Correct the ground in `ScalarProgram::StrictSerialMaximum`'s doc: keep the empty-domain refusal and its ADR 0023 and ADR 0025 basis, drop or repair the algebraic claim, and state the padding question separately. Describe what the code does now — no field is being added by this ticket.
- Check the sibling comment on `empty_domain_is_satisfied` in `crates/tiler-ir/src/schedule/builder.rs`, whose non-emptiness derivation is correct *for exact coverage* and should say so rather than read as unconditional.
- Add no field and no behaviour. A stated padding identity on a schedule is [ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) decision 7's public boundary and is Tom's, not this ticket's; this ticket only stops the doc from denying that such a value exists.
- Size the test to the change: a doc correction needs none, and one asserting a comment's wording would be worse than none.

## Closes when

The comment describes the family the code implements, the empty-domain refusal keeps its authority, and no reader can derive "the extrema family cannot be padded" from either site.
