---
id: produce-typed-strict-affine-assemble-semantic-precondition
title: Produce the strict-affine Assemble scale precondition
status: in-progress
priority: p2
dependencies: [produce-typed-strict-affine-quantize-semantic-preconditions]
related: [enforce-resolved-encoded-value-binding-conformance, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/reference, contracts/foundation, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantic-ir, validation, quantization]
claimed_from: todo
assignee: worker-assemble-pre
lease_expires_at: 1785567212
---

## User-visible outcome

`AssembleStrictAffine` cannot construct a governed encoded value from a non-positive or non-finite scale. Its exact scale restriction is a typed occurrence-owned semantic precondition with the same proof, disproof, residual, identity, and invalid-input behavior established for `Quantize`, but it remains a distinct declaration on the Assemble occurrence.

## Implementation keys

- Declare `PositiveFiniteScalar` on Assemble operand 1 over the whole rank-zero f32 logical value with an Assemble-specific stable invalid-input code.
- Reuse the reviewed predicate declaration, static exact-constant proof, assessment, obligation identity, and program-view vocabulary from `produce-typed-strict-affine-quantize-semantic-preconditions`. Do not copy another evaluator or editable string authority.
- Keep codes and zero-point payload domains in resolved-value conformance. Keep packed-tail, padding, alignment, bit order, and storage canonicality in physical representation validation.
- A directly governed positive finite f32 constant proves the declaration; zero, negative, infinite, or NaN constants disprove transactionally; every other producer remains residual.
- Preserve the exact Assemble occurrence, subject value/view/type, declaration ordinal, and invalid-input code in obligation identity. Never reuse a Quantize obligation merely because predicate and subject shape resemble one another.
- Reuse the shared `PositiveFiniteScalar` evaluator through one Assemble-owned `assemble_preconditions()` declaration and an Assemble-specific stable diagnostic code. The reference path already calls `read_scale`; do not add a second validator.

## Closes when

Valid constants prove; positive and negative zero, positive and negative finite values, subnormals, positive and negative infinity, quiet NaN, and signaling NaN take their exact proved/refused class transactionally; runtime-unknown scale leaves one exact residual; dead Assemble occurrences compact their assessment away; the normative reference and typed declaration agree for U4 and U8; Assemble and Quantize retain distinct declaration and obligation identities; every new check has been observed failing under perturbation; targeted `tiler-ir` and `tiler-reference` tests and Clippy pass; and the batch gate passes.

## Graph maintenance

- Update ADR 0033 and numerical/IR maturity text only for the newly implemented Assemble producer.
- Relate the residual to `enforce-resolved-encoded-value-binding-conformance`, which separately owns direct encoded program inputs and does not replace this operation precondition.
- Update the dtype maturity matrix only for the exact Assemble semantic-validation cell.
- Advance the semantic definition projection, standard registry provenance/identity, and owning provider revision exactly once, then recompute every pinned identity on the merged tree rather than copying Quantize fixtures.
