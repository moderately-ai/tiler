---
schema: "tiler-doc/v1"
id: "ADR-0024"
kind: "decision"
title: "Use round-to-nearest ties-to-even for initial arithmetic"
topics: ["numerics","floating-point","rounding"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.operation-conformance-matrix"]
ticket: "numerical-policy-contract"
---

# 0024: Use round-to-nearest ties-to-even for initial arithmetic

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [operation conformance matrix](../research/numerics/operation-conformance-matrix.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Resolved dtypes do not completely define a floating-point operation; its
rounding direction is observable. Tiler had already separated arithmetic,
conversion, FMA, and transcendental contracts, but had not explicitly selected
the initial rounding direction for ordinary arithmetic.

Round-to-nearest, ties-to-even is the standard default IEEE arithmetic contract
and the practical baseline for GPU tensor computation. Leaving it ambient would
allow a backend mode to change semantic meaning.

## Decision

Initial floating-point `Add`, `Subtract`, `Multiply`, and `Divide` operations
use round-to-nearest, ties-to-even at each semantic operation boundary.
Semantic `Fma` requires the correctly rounded fused result under the same
rounding direction.

Conversions retain their specialized typed rounding contracts, and
transcendentals retain their operation-specific accuracy contracts. A future
directed-rounding arithmetic family is additive and versioned; it does not
reinterpret existing operation keys.

## Consequences

- Reference evaluation and adversarial halfway tests have one initial oracle.
- Separate multiply/add and required FMA remain observably distinct.
- Backends must prove, emulate, relax explicitly, or reject this rounding
  contract rather than inheriting an ambient mode.
- Other rounding directions remain possible through new typed contracts.

## Alternatives considered

Inheriting a target default makes semantics absent from graph identity.
Supporting every directed rounding mode initially expands backend and reference
scope without an identified tensor-kernel requirement. Making rounding a
graph-wide switch is too coarse for conversions and specialized operations.

## Primary precedents

- [IEEE 754-2019](https://standards.ieee.org/ieee/754/6210/)
- [StableHLO floating-point operations](https://openxla.org/stablehlo/spec#floating-point-operations)
- [LLVM constrained floating-point intrinsics](https://llvm.org/docs/LangRef.html#constrained-floating-point-intrinsics)
