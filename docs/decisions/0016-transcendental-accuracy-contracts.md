---
schema: "tiler-doc/v1"
id: "ADR-0016"
kind: "decision"
title: "Resolve transcendental accuracy per operation"
topics: ["numerics","transcendentals","accuracy"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.transcendental-accuracy-precedents"]
ticket: "numerical-policy-contract"
---

# 0016: Resolve transcendental accuracy per operation

**Status:** accepted

The contract vocabulary that this decision leaves open is fixed by ADR 0042:
four discriminated accuracy forms, exact rational tolerances, versioned metric
keys, and separately classified conformance evidence. Only the initial
supported subset remains a product-profile choice.

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [transcendental accuracy precedents](../research/numerics/transcendental-accuracy-precedents.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Transcendental operations may have several useful implementation contracts,
from correctly rounded results to bounded approximations or a versioned target
elementary-function behavior. Treating every operation as correctly rounded
can require expensive emulation or reject useful targets. Allowing a global
fast-math switch instead makes accuracy implicit and can silently affect
unrelated operations.

## Decision

Every transcendental semantic operation carries a resolved,
operation-specific accuracy contract. Frontends may expose named presets, but
they resolve those presets before canonical semantic admission. Later compiler
phases do not consult ambient frontend settings, compiler flags, or backend
defaults to reinterpret accuracy.

The contract vocabulary must support testable forms such as correctly rounded
results, bounded-error models over stated domains, or explicitly versioned
backend-elementary behavior. The final metric types and initial supported
subset remain open.

Any authorized approximation relaxation resolves to a new canonical, testable
effective envelope before semantic optimization. An optimizer or backend may
choose an implementation only when its guarantee refines that resolved
contract. A physical phase never weakens admitted semantics by merely consuming
a permission. Otherwise it emulates or rejects the plan according to the
standard backend feasibility classification.

This decision governs local operation accuracy. It does not authorize moving
or redistributing an error budget between operations.

## Consequences

- Transcendental accuracy participates in semantic, plan, artifact, reference,
  and explain identity.
- Different operations in one graph may require different accuracy.
- Backends cannot silently substitute approximate native functions.
- Correctly rounded contracts may require emulation or be unsupported.
- Bounded contracts need explicit domains, metrics, special-value behavior,
  and conformance tests.

## Alternatives considered

Making every transcendental correctly rounded is simple but may unnecessarily
exclude efficient implementations. A global approximate-math boolean is too
broad. Deferring accuracy to backend lowering makes tensor meaning depend on
state absent from canonical IR.
