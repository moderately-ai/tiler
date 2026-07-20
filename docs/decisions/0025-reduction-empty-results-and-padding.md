---
schema: "tiler-doc/v1"
id: "ADR-0025"
kind: "decision"
title: "Separate reduction empty results from physical padding"
topics: ["numerics","reductions","semantics"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "reduction-semantics-contract"
---

# 0025: Separate reduction empty results from physical padding

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [reduction-semantics-contract](../../tickets/reduction-semantics-contract.md).


## Context

Reduction APIs need an empty-domain result, while parallel schedules often need
values for inactive lanes or empty partials. These values are easily called an
“identity,” but they are not always interchangeable under observable machine
semantics.

For strict floating addition with round-to-nearest, an empty sum may be
`+0.0`, yet `Add(-0.0, +0.0)` produces `+0.0`. Injecting that empty result into
a singleton `[-0.0]` reduction changes the result bits. Algebraic neutrality is
therefore insufficient when signed zero is observable.

## Decision

Reduction contracts distinguish:

- the typed result of an empty domain, or an empty-domain error;
- an optional logical `initial`, which contributes exactly once;
- algebraic identity properties used by rewrite reasoning; and
- physical padding values proven observably neutral under a named numerical
  and conformance contract.

A scheduler may inject or replicate padding only with the last proof. It cannot
infer padding legality from the empty result or an algebraic monoid claim. If no
neutral padding exists, physical plans track nonempty partials, mask inactive
lanes, or use another verified construction.

## Consequences

- Strict reductions preserve signed-zero and other bit-level distinctions.
- Relaxed signed-zero contracts may expose cheaper padding-capable alternatives
  without changing strict semantics.
- Reduction schedules and explain output record their padding strategy and the
  capability/permission that justifies it.
- Empty-domain behavior remains independent of SIMD width and topology.

## Alternatives considered

Treating every empty result as replicable padding is algebraically attractive
but unsound for observable signed zero and potentially other specialized
combiners. Forbidding padding globally is safe but rejects efficient schedules
where neutrality is actually proven. Hiding the distinction inside backend
lowering makes plan legality unavailable to the scheduler and verifier.
