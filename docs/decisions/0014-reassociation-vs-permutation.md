---
schema: "tiler-doc/v1"
id: "ADR-0014"
kind: "decision"
title: "Separate reassociation from operand permutation"
topics: ["numerics","reductions","optimization"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "numerical-policy-contract"
---

# 0014: Separate reassociation from operand permutation

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Changing `(a + b) + c` to `a + (b + c)` regroups operands without changing
their logical order. Changing it to `(a + c) + b` also permutes them. Both can
change floating-point results, but they are different freedoms. Some reduction
combiners support regrouping but are not commutative, and some numerical
contracts may intentionally permit one transformation without the other.

A single unordered-reduction permission grants both freedoms even when only
one is necessary.

## Decision

Reduction order contracts represent reassociation and operand permutation as
independent dimensions. Reassociation permission never implies permutation
permission, and permutation permission never implies reassociation permission.

Each transformation requires two independent facts:

1. the operation declares the applicable algebraic capability; and
2. the operation's resolved numerical permissions authorize consuming it.

In particular, permutation requires a commutative operation capability as well
as permission to reorder. A physical topology proves its regrouping and
permutation behavior separately against the semantic contract.

## Consequences

- An ordered parallel tree can regroup operands without silently permitting
  arbitrary permutation.
- Associative but noncommutative combiners remain optimizable within their
  actual capabilities.
- Scheduler alternatives and rejection explanations name the precise order
  freedom they require.
- Operation capability does not itself authorize a numerical relaxation; the
  program ceiling and resolved per-operation permissions still govern it.
- Reduction legality has one additional explicit dimension.

## Alternatives considered

A single unordered-reduction flag is easier to propagate but over-authorizes
many schedules. Inferring permutation permission from reassociation conflates
associativity with commutativity. Encoding only the chosen physical tree makes
the distinction visible too late for logical rewrite and candidate legality.
