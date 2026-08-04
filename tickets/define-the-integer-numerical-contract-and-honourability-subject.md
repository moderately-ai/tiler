---
id: define-the-integer-numerical-contract-and-honourability-subject
title: Define the integer numerical contract and its honourability subject
status: deferred
priority: p2
dependencies: []
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, admit-a-storage-carrier-for-integer-program-inputs, measure-code-domain-integer-arithmetic-on-the-qualified-apple-row, generalize-the-sub-byte-storage-encoding-contract, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, integers, numerical-contract]
---
## User-visible outcome

A target can be asked whether it honours a wrapping, saturating, checked, or widening integer operation, and can answer `Unsupported` or `Unknown` by name — the same way it already answers for floating-point scalar arithmetic.

## Why this exists, stated so it can be refuted

**Fact — the operation contracts already exist.** [Numerical semantics](../docs/numerical-semantics.md) `## Integer and index arithmetic` names the wrapping, saturating, checked, and widening data-arithmetic families and the signed truncating, floor, Euclidean, ceiling, canonical unsigned, and exact division families, under [ADR 0039](../docs/decisions/0039-explicit-integer-overflow-operations.md) and [ADR 0040](../docs/decisions/0040-specialize-integer-division-families.md). This ticket does not redesign them.

**Fact — what does not exist is a subject those families could be declared honourable at.** The same contract's `### Per-dimension honourability` closes with: "The subject vocabulary covers floating-point scalar arithmetic. Integer overflow families, boolean semantics, quantized compound schemes, and any future policy family have their own contracts elsewhere in this document and acquire no honourability declaration by standing beside this one." [The dtype support ledger](../docs/dtype-support.md)'s dry run states the same thing as a rung-6 failure for the integer column.

**Fact — measured evidence already sits on the far side of that seam.** [`measure-code-domain-integer-arithmetic-on-the-qualified-apple-row`](measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md) measured a `u8` buffer read, an `int` subtraction, an `int`-to-`float` conversion, and the following multiply on the qualified Apple row. There is nowhere on a target profile to record what that measurement establishes.

**Inference.** This is the same shape of seam [`admit-a-bf16-scalar-arithmetic-subject`](admit-a-bf16-scalar-arithmetic-subject.md) opened for floats: a validated construction route proving the association from a governed registry authority, rather than a widened check. Naming the analogy is not proposing the solution — an integer family's dimensions are not the float dimensions, because overflow is a semantic choice rather than a rounding mode.

## Activation trigger

A named tensor workload selects an exact width, an operation family, an overflow, division, or conversion behaviour, a storage, a target, and a corpus. **Quantized codes do not fire it** — the ledger's `### Signed and unsigned integers` trigger says so by name, and the code-domain measurement above is exactly that case.

## Explicit non-goals

- The storage carrier for a `[T]` token-ID program input. That is [`admit-a-storage-carrier-for-integer-program-inputs`](admit-a-storage-carrier-for-integer-program-inputs.md)'s, it is a public boundary for Tom, and it selects no arithmetic family.
- Sub-byte packing, which is [`generalize-the-sub-byte-storage-encoding-contract`](generalize-the-sub-byte-storage-encoding-contract.md)'s.
- `RQ-OP-01`'s arity question for a checked-overflow operation, which the operation axis owns and which this work must consume rather than re-decide.

## Closes when

The trigger has fired, an integer honourability subject exists with a validated construction route, every pair it must refuse refuses by name with the refusal watched firing, and the measured Apple integer evidence is declarable through it.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-2 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).
- `contracts/numerics` is declared because the subject vocabulary sentence quoted above lives in `docs/numerical-semantics.md` and must move in the same change as the mechanism.
