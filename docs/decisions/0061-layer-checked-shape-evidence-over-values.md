---
schema: "tiler-doc/v1"
id: "ADR-0061"
kind: "decision"
title: "Layer checked shape evidence over canonical typed values"
topics: ["rust", "semantics", "shapes", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.shape-environment-contract", "tiler.research.shapes.constraint-prover-boundary", "tiler.research.semantic-graph.rust-construction-lifecycle"]
ticket: "prototype-semantic-reference-slice"
---

# 0061: Layer checked shape evidence over canonical typed values

**Status:** accepted

## Context

ADR 0059 selects exact nominal `Value<T>` handles while leaving shape
verification to the canonical graph. Rust can nevertheless reject useful
classes of shape mistakes before graph admission: fixed-rank mismatches, exact
static-extent mismatches, and invalid statically selected axes. Omitting this
capability would postpone errors for which an ordinary Rust call site already
has sufficient evidence.

Making an exact shape parameter mandatory on every canonical value would solve
only the static subset. Tiler also admits graph-scoped symbolic extents,
runtime-bound root dimensions, solver-derived equalities, parsed frontends, and
operation results whose shape depends on attributes or constraints. Stable Rust
does not provide a general const-generic shape algebra for these relationships.
A parallel trait-level solver would duplicate the authoritative `ShapeEnv`,
produce weaker diagnostics, and risk disagreement with graph verification.

## Decision

`Value<T>` remains the canonical Rust capability for a graph-owned value with
an exact resolved shape-independent value type. Absence of Rust shape evidence
means only that the caller lacks such evidence. It never means that the graph
has unknown rank: every admitted semantic value retains its authoritative
ranked shape-expression vector and scoped `ShapeEnv` metadata.

Tiler also provides an optional checked refinement capability of the conceptual
form `ShapedValue<T, E>`, where `E` is non-authoritative Rust-side shape
evidence. The initial bounded vocabulary covers at least fixed rank and exact
static shapes. More expressive symbolic evidence is reserved until a stable-
Rust feasibility spike demonstrates sound marker binding, useful diagnostics,
and acceptable compile-time cost.

Only the owning builder or immutable program may construct a refined handle,
after verifying `E` against the authoritative graph metadata. Callers cannot
forge evidence by implementing a trait or construct a refined handle from a
`ValueId`. Weakening to `Value<T>` is explicit and zero-cost; refinement is
checked and fallible unless the producing builder operation established the
evidence directly.

Graph and solver proofs of relationships between several values use separately
typed witness capabilities, conceptually `ShapeWitness<P>`. A witness records
the exact graph-owned predicate and subjects it proves. It is not a general
boolean, a durable replacement for the predicate, or permission to skip graph
ownership checks.

There is one semantic admission and verification path. Refined-handle methods
or future frontend facades delegate to the same host-owned operation admission
used by unrefined values; they do not implement a second shape inference or
mutation system. Operation-specific evidence propagation is permitted only
when the result relationship is unambiguous and revalidated against the graph.
Solver-derived or runtime-dependent results may conservatively return weaker
evidence without weakening their authoritative graph shape.

Rust marker types, const parameters, marker names, and `TypeId` values never
enter semantic, compilation, artifact, or cache identity. Canonical shape
expressions, root bindings, constraints, and operation semantics continue to
determine identity. Shape evidence must not direct physical specialization;
the optimizer and scheduler make that decision from canonical graph facts.

The initial public operation surface remains builder-centered. Refined values
may improve arguments, results, and diagnostics, but do not introduce an
independent fluent operation API until measurements demonstrate that it can
remain complete and nonduplicative.

## Consequences

- Rust-authored programs may fail at compilation for supported static shape
  mistakes while dynamic and parsed frontends retain the same semantic IR.
- Complex symbolic compatibility remains a typed graph admission or solver
  result with Tiler-authored diagnostics rather than a Rust trait error.
- Static evidence can be retained and propagated without becoming graph
  authority or durable identity.
- The prototype must test explicit weakening, checked refinement, forgery
  resistance, foreign-graph rejection, and preservation of the single
  admission path.
- The exact public spelling, sealed evidence vocabulary, and compile-time cost
  require a bounded stable-Rust spike before stabilization.

## Alternatives considered

Mandatory `Value<T, S>` maximizes static propagation for wholly static Rust
programs but makes every dynamic frontend and solver-derived relationship pass
through an incomplete parallel type algebra. Rank-only `Value<T, const R:
usize>` is simpler but cannot retain exact static extents and permanently
privileges one evidence class. Keeping all shapes out of Rust types preserves
the canonical model but misses inexpensive, high-value call-site diagnostics.
An open authoritative `ShapeEvidence` trait is unsound because downstream code
could claim facts that the graph never established.

## Traceability

The [IR contract](../ir.md) owns canonical value shape metadata, refined
authoring capabilities, witnesses, and verification boundaries. The [shape
environment research](../research/shapes/shape-environment-contract.md) owns the
ranked semantic-graph and scoped constraint model.
