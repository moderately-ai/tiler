---
id: produce-typed-strict-affine-assemble-semantic-precondition
title: Produce the strict-affine Assemble scale precondition
status: todo
priority: p2
dependencies: [produce-typed-strict-affine-quantize-semantic-preconditions]
related: [enforce-resolved-encoded-value-binding-conformance]
scopes: [implementation/ir, implementation/reference, contracts/foundation, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantic-ir, validation, quantization]
---

## User-visible outcome

`AssembleStrictAffine` cannot construct a governed encoded value from a non-positive or non-finite scale. Its exact scale restriction is a typed occurrence-owned semantic precondition with the same proof, disproof, residual, identity, and invalid-input behavior established for `Quantize`, but it remains a distinct declaration on the Assemble occurrence.

## Implementation keys

- Declare `PositiveFiniteScalar` on Assemble operand 1 over the whole rank-zero f32 logical value with an Assemble-specific stable invalid-input code.
- Reuse the reviewed predicate declaration, static exact-constant proof, assessment, obligation identity, and program-view vocabulary from `produce-typed-strict-affine-quantize-semantic-preconditions`. Do not copy another evaluator or editable string authority.
- Keep codes and zero-point payload domains in resolved-value conformance. Keep packed-tail, padding, alignment, bit order, and storage canonicality in physical representation validation.
- A directly governed positive finite f32 constant proves the declaration; zero, negative, infinite, or NaN constants disprove transactionally; every other producer remains residual.
- Preserve the exact Assemble occurrence, subject value/view/type, declaration ordinal, and invalid-input code in obligation identity. Never reuse a Quantize obligation merely because predicate and subject shape resemble one another.

## Closes when

Valid constants prove, every invalid scale class rejects transactionally, runtime-unknown scale leaves one exact residual, dead Assemble occurrences compact their assessment away, the normative reference and typed declaration agree for U4 and U8, every new check has been observed failing under perturbation, targeted `tiler-ir` and `tiler-reference` tests and Clippy pass, and the batch gate passes.

## Graph maintenance

- Update ADR 0033 and numerical/IR maturity text only for the newly implemented Assemble producer.
- Relate the residual to `enforce-resolved-encoded-value-binding-conformance`, which separately owns direct encoded program inputs and does not replace this operation precondition.
- Update the dtype maturity matrix only for the exact Assemble semantic-validation cell.
