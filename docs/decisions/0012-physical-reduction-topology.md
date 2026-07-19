# 0012: Keep reduction topology in physical plans

**Status:** accepted

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

The stability scope of a deterministic order is a separate unresolved
contract. This decision does not use `deterministic` as an unqualified promise.

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
