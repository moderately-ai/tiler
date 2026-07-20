---
schema: "tiler-doc/v1"
id: "ADR-0012"
kind: "decision"
title: "Keep reduction topology in physical plans"
topics: ["numerics","reductions","scheduling"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "reduction-semantics-contract"
---

# 0012: Keep reduction topology in physical plans

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [reduction-semantics-contract](../../tickets/reduction-semantics-contract.md).


## Context

Floating-point reduction results can change when the same inputs are combined
with a different parenthesization. A semantic reduction must therefore
constrain evaluation order. However, a concrete parallel reduction tree also
contains target-dependent scheduling choices such as SIMD width, threadgroup
partitioning, synchronization, and intermediate passes.

Putting that concrete tree in the semantic graph would make tensor meaning
depend on a GPU schedule and would suppress otherwise legal physical
alternatives.

## Decision

A semantic reduction carries an order contract that defines the allowed
evaluation-order or result class. It does not carry a concrete parallel
reduction topology. The contract must be expressive enough to distinguish an
ordered fold, a deterministically selected legal order, and a relaxed result
set when reassociation is permitted; the final public variants and names remain
to be specified.

Physical planning chooses and records the actual topology, including
partitioning, tree shape, synchronization, and multi-pass structure. The
schedule is legal only when its evaluation is contained by the semantic order
contract. The selected topology participates in physical-plan and artifact
identity.

Deterministic order uses a separately defined, explicit stability scope. This
decision does not use `deterministic` as an unqualified promise.

## Consequences

- Semantic reductions remain backend-neutral while still constraining
  floating-point results.
- The optimizer may cost several legal reduction topologies without changing
  the semantic graph.
- Ordered reductions can reject parallel trees that change their evaluation.
- Relaxed reductions can admit target-specific trees only through explicit
  permissions.
- Explain output can distinguish rejection by semantic order from rejection by
  target resources or cost.

## Alternatives considered

Storing the exact tree in semantic IR completely specifies evaluation but
mixes target scheduling into tensor meaning. Leaving reduction order entirely
to physical planning permits numerical changes absent from the semantic
contract. A boolean `deterministic` flag is insufficient because it does not
state which executions, artifacts, targets, or toolchains share the promise.
