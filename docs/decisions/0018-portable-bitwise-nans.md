---
schema: "tiler-doc/v1"
id: "ADR-0018"
kind: "decision"
title: "Canonicalize arithmetic NaNs for portable bitwise results"
topics: ["numerics","floating-point","nan"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.operation-conformance-matrix"]
ticket: "numerical-policy-contract"
---

# 0018: Canonicalize arithmetic NaNs for portable bitwise results

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [operation conformance matrix](../research/numerics/operation-conformance-matrix.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Floating-point NaNs contain sign and payload bits. An arithmetic operation can
produce a quiet NaN while different valid implementations propagate an operand
payload, choose another payload, or canonicalize it. Permitting any quiet NaN
is incompatible with a portable bitwise result guarantee. Requiring exact
payload propagation instead can constrain native instructions and rewrites and
may require emulation.

Bit-preserving movement is different from arithmetic production: copying or
viewing a tensor should not silently rewrite its stored NaN payloads.

## Decision

Portable-bitwise arithmetic canonicalizes every NaN result to one
dtype-specific, versioned quiet-NaN bit pattern. The canonical pattern is part
of semantic and artifact identity.

Canonicalization follows operation semantics rather than globally rewriting
tensor storage. Operations whose contract preserves or selects existing bits,
including views and bit-preserving copies, preserve the selected source bits.
Numeric conversions follow their resolved conversion contract, and constants
retain their declared bits until a value-producing operation defines another
result.

Other conformance modes may explicitly require operand-payload propagation or
permit any quiet NaN. No operation inherits NaN payload behavior from the
backend.

## Consequences

- Portable-bitwise tests have one expected arithmetic NaN representation.
- Backends may need explicit canonicalization after otherwise valid native
  arithmetic.
- Copies, views, and bit reinterpretation do not destroy payload bits.
- Payload-sensitive frontends can request a stronger explicit contract when a
  backend can implement it.
- NaN policy participates in reference, plan, artifact, and explain identity.

## Alternatives considered

Allowing any quiet NaN maximizes native flexibility but cannot support portable
bitwise equality. Requiring payload propagation everywhere gives a strong
provenance rule but conflates arithmetic with bit-preserving movement and can
make ordinary portable execution unnecessarily expensive or infeasible.
