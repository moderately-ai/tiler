---
schema: "tiler-doc/v1"
id: "ADR-0021"
kind: "decision"
title: "Require proof or runtime validation for value assumptions"
topics: ["numerics","validation","optimization"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics", "tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.numerics.region-accuracy-contract"]
ticket: "numerical-policy-contract"
---

# 0021: Require proof or runtime validation for value assumptions

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [region accuracy contract](../research/numerics/region-accuracy-contract.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Numerical rewrites may require facts about tensor contents. For example,
`x / x -> 1` requires restrictions excluding zero, NaN, and infinity. A
frontend or caller can cheaply state those restrictions, but validating them
may require scanning the tensor. Trusting an unchecked statement can silently
produce an incorrect result when it is violated.

Optimization applicability must also remain distinct from semantic program
validity. A failed vector-alignment or finite-values guard should not turn an
otherwise valid computation into an error merely because one optimized plan is
inapplicable.

## Decision

Correctness-sensitive optimization may initially consume a value-domain fact
only when it is compiler-proven or explicitly runtime-validated before the
dependent plan executes. Caller-declared but unvalidated value assumptions are
preserved with provenance and explanation, but do not establish rewrite or
schedule legality.

Runtime validation is a costed computation. If validation is unavailable or
not worthwhile, Tiler ignores the assumption for optimization and selects a
general valid plan.

Semantic input preconditions and physical variant guards are distinct:

- failure of a semantic precondition produces a precise invalid-input error;
- failure of an optimization guard selects another valid plan or fallback
  before dependent work begins.

## Consequences

- Unchecked caller assertions cannot silently widen legal transformations.
- Proven and validated facts remain useful to logical rewrites and physical
  schedules.
- Tensor scans compete in the cost model with the optimization they enable.
- Explain output identifies each assumption's provenance, validation, consumer,
  and failure behavior.
- A future explicitly trusted-contract mode remains possible as a new policy;
  it does not require reinterpreting initial programs.

## Alternatives considered

Always trusting caller declarations avoids validation cost but permits silent
wrong answers on contract violations. Always scanning preserves correctness but
can cost more than the optimization. Ignoring all value-domain information is
safe but unnecessarily rejects facts the compiler can prove or runtime can
validate.
