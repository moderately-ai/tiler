---
schema: "tiler-doc/v1"
id: "ADR-0017"
kind: "decision"
title: "Separate local semantics from region accuracy goals"
topics: ["numerics","accuracy","optimization"]
catalog_group: "numerical-operations"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics", "tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.numerics.region-accuracy-contract"]
ticket: "research-region-accuracy-contracts-and-analyzable-error-budgets"
---

# 0017: Separate local semantics from region accuracy goals

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [region accuracy contract](../research/numerics/region-accuracy-contract.md).
- **Work record:** [research-region-accuracy-contracts-and-analyzable-error-budgets](../../tickets/research-region-accuracy-contracts-and-analyzable-error-budgets.md).


## Context

Local operation tolerances do not safely add into a whole-graph error bound.
Cancellation, sensitivity, correlation, exceptional values, casts, reductions,
and path divergence can amplify or mask local error. Whole-expression systems
therefore require an explicit reference, bounded input domain, error metric,
and global analysis.

Current compiler IR precedents primarily expose per-operation permissions and
accuracy. Research systems such as FPTuner and Daisy demonstrate useful
whole-expression contracts, but over deliberately bounded program classes and
with substantially stronger analysis requirements. Sampling-based optimization
provides different evidence from a sound worst-case proof.

## Decision

Precise local numerical contracts are mandatory and authoritative. The initial
Tiler optimizer does not redistribute a shared output-error budget across
operations.

A future optional region/output accuracy layer is additive. A region goal must
name:

- the observable output or region;
- explicit reference semantics or oracle;
- input value/range assumptions and shape bounds;
- error metric, tolerance, and exceptional-value behavior; and
- evidence class, distinguishing sound proof, empirical validation with a
  named test definition, and unknown.

The goal is a hard feasibility constraint, not an optimizer cost. Cost is
optimized only among plans demonstrated or explicitly accepted to satisfy it.
A region goal does not silently override local operation semantics. Any future
delegation of internal accuracy to a region goal must be explicit and scoped.

Tiler preserves semantic casts and materialization boundaries, reduction
topology, input/shape assumptions, reference provenance, and resolved local
permissions so the future layer can be added without reconstructing erased
meaning.

## Consequences

- Initial legality and reference evaluation remain compositional at operation
  boundaries.
- Some globally acceptable faster plans remain unavailable initially.
- Empirical testing cannot be presented as a sound proof.
- Graph-level analysis can be introduced incrementally for bounded analyzable
  program classes.
- The future research spike is tracked separately and does not block the local
  numerical-policy contract.

## Alternatives considered

Naively summing local error bounds is unsound or uselessly pessimistic. Making
one graph tolerance immediately authoritative would require solving range,
conditioning, path, reduction, and symbolic-extent analysis before the local
compiler contract is usable. Rejecting graph goals permanently would forfeit
valuable mixed-precision and stable-rewrite optimization demonstrated by prior
research.
