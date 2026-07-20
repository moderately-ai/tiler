---
schema: "tiler-doc/v1"
id: "ADR-0022"
kind: "decision"
title: "Define reduction identities and initial values"
topics: ["numerics","reductions","semantics"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "reduction-semantics-contract"
---

# 0022: Define reduction identities and initial values

**Status:** accepted

The identity terminology in this decision is refined by ADR 0025: an empty
result is not automatically a bitwise-neutral, replicable padding value.

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [reduction-semantics-contract](../../tickets/reduction-semantics-contract.md).


## Context

Empty reductions need defined behavior. Some operations have a typed
mathematical identity, such as zero for addition, while others such as the
initial minimum/maximum contracts do not. An explicit initial value could mean
either a real accumulator seed or a value used only when the domain is empty;
overloading both meanings would make logical and parallel execution ambiguous.

NumPy defines `initial` as the value that starts a reduction and requires one
for an empty reduction whose operation has no identity. Rust similarly
distinguishes seeded `fold` from unseeded `reduce`. StableHLO represents init
values explicitly in its reduction operation. Parallel lowering adds a further
constraint: an arbitrary seed is not necessarily neutral and cannot be
duplicated across partial reductions.

## Decision

Each reduction operation declares a typed empty-domain result or explicitly
rejects an empty domain. It separately declares any algebraic-identity and
replicable-padding capabilities under its resolved conformance contract.

An optional explicit `initial` is a true reduction seed. It contributes exactly
once to each logical output reduction domain whether that domain is empty or
non-empty. It is not an empty-only fallback.

A physical schedule may inject or replicate a padding value only when the
resolved operation contract proves it observably neutral. It may not infer that
property from the empty result, nor replicate an arbitrary initial value per
lane, partition, threadgroup, or reduction pass.

An identity-less reduction requires either an explicit initial value or a
proven/runtime-validated non-empty domain. Static violations fail graph
verification; a failed dynamic non-empty semantic precondition produces a
precise invalid-input error before dependent execution.

## Consequences

- Common monoidal reductions have natural empty results.
- `minimum([], initial=10)` and `minimum([20], initial=10)` both produce `10`.
- Parallel reduction topology must track the seed as one logical contributor.
- Identity, initial-value conversion, empty behavior, order permissions, and
  accumulator dtype all participate in semantic and artifact identity.
- Empty-only fallback remains expressible later as a separately named operation
  or explicit conditional without changing `initial` semantics.

## Alternatives considered

Requiring an explicit seed for every reduction is uniform but needlessly
verbose for operations with valid intrinsic identities. Treating `initial` as
an empty-only default is convenient but is not a reduction seed and obscures
its behavior on non-empty input. Letting backends choose empty behavior would
make fusion and cross-target conformance unsound.

## Primary precedents

- [StableHLO `reduce`](https://openxla.org/stablehlo/spec#reduce)
- [NumPy `ufunc.reduce`](https://numpy.org/doc/stable/reference/generated/numpy.ufunc.reduce.html)
- [Rust `Iterator::fold` and `Iterator::reduce`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
