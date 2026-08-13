---
schema: "tiler-doc/v1"
id: "ADR-0025"
kind: "decision"
title: "Separate reduction empty results from physical padding"
topics: ["numerics","reductions","semantics"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
refines: ["ADR-0022"]
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

## Implementation boundary

Added 2026-08-13 by [`admit-shared-contributor-coverage-and-reduction-padding-identity`](../../tickets/admit-shared-contributor-coverage-and-reduction-padding-identity.md), which moved `implementation_status` from `not-started` to `partial`. This section states which clauses that value rests on and which it does not. It is a status record and adds no decision.

**Realized — empty result and physical padding are separately stated.** A sum family's `empty_identity_bits` remains the empty-domain result and is still pinned to `+0.0`. An identity-padded split states a `ReductionPaddingIdentity` on `ContributorCoverage`; the verifier derives two-sided neutrality against the region's combiner, arithmetic type, signed-zero contract, and family canonicalization. Exact coverage carries no identity. There is no fallback from a pad to `empty_identity_bits`.

**Realized — padding is suffix-only and derived, not declared.** The verifier computes `capacity − contributors` by checked subtraction, requires a canonical suffix (`T·k·(R−1) < C ≤ T·k·R` on a cooperative tile), and names exact-coverage and padded-coverage failures separately. `ContributorPartition::covers` keeps its exact meaning. `KernelSchedule::tail` remains iteration-domain launch coverage.

**Unrealized — no schedule is lowered with padding.** Identity-padded coverage is representable and intrinsically verified; kernel lowering refuses it under `padded-contributor-coverage`. Vector-lane and subgroup consumers are not admitted here. No algebraic-identity capability is declared on a family conformance contract; the proof is a schedule-verification derivation.
