---
schema: "tiler-doc/v1"
id: "tiler.research.shapes.stable-rust-shape-evidence"
kind: "research"
title: "Stable-Rust shape-evidence feasibility"
topics: ["shapes", "rust", "semantics", "diagnostics"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
informs: ["tiler.contract.ir"]
adopted_by: ["ADR-0061"]
ticket: "prototype-shape-evidence-spike"
---

# Stable-Rust shape-evidence feasibility

## Question

Can Tiler provide useful static shape diagnostics on stable Rust without
duplicating the canonical graph solver or making every value statically
shaped?

## Findings

**Fact:** Rust 1.89 supports a zero-cost `ShapedValue<T, E>` wrapper with
private construction, explicit weakening to `Value<T>`, sealed evidence
classes, and an open static-shape descriptor whose implementation grants no
proof. Successful refinement can always compare the descriptor with the
owning graph's canonical metadata.

**Fact:** pointwise methods can require equal evidence types and reject
different fixed ranks or exact-shape descriptors at compilation. They must
still call canonical admission: equal `Rank<2>` evidence does not prove equal
extents, and an externally implemented static descriptor is only a claim until
checked.

**Fact:** inline const assertions reject an out-of-range `Axis<A>` and
duplicate `Axes2<A, B>` during generic instantiation on Rust 1.89. The retained
diagnostic names the invariant and the call site. General result types such as
`Rank<{R - 1}>` still require unstable generic const expressions, so a
reduction must initially return weaker evidence or a separately checked result
refinement.

**Fact:** graph-owned multi-value witnesses can bind a sealed predicate, graph
identity, and ordered subject handles. A foreign witness or a witness presented
for different subjects rejects without treating an open Rust trait as solver
authority.

**Measurement:** on an Apple M4 Max host, the generated 1/10/100/1,000 exact-
shape workloads took 0.07/0.07/0.08/0.17 seconds for a package-clean check and
0.03/0.03/0.04/0.06 seconds for a touched-source incremental check. Optimized
builds took 0.07/0.08/0.09/0.19 seconds. Peak RSS at 1,000 shapes was 122.9 MiB
for clean check, 92.6 MiB for incremental check, and 136.8 MiB for optimized
build. Every optimized binary was 404,016 bytes, showing no binary growth in
this zero-sized evidence workload. These are single-run feasibility bounds,
not stable performance guarantees. Exact provenance and method are in the
[measurement summary](../../../spikes/shapes/shape-evidence/measurements/summary.json).

## Recommendation

Use this bounded initial vocabulary:

```rust,ignore
Value<T>
ShapedValue<T, Rank<R>>
ShapedValue<T, Exact<S>> where S: StaticShapeSpec
Axis<A>
Axes2<A, B>
ShapeWitness<SameShape>
```

`ShapeEvidence` and witness predicate classes remain host-controlled. It is
safe for `StaticShapeSpec` to be downstream-implementable because a descriptor
cannot construct `ShapedValue`; every refinement is checked. Tiler may later
add ergonomic built-in rank tuples or declaration macros without changing the
authority boundary.

Reduction output evidence weakens initially. Do not introduce nightly
features, recursive type-level extent lists, generated cross-product trait
implementations, specialization, or a parallel symbolic solver to preserve it.
Operation-specific result evidence may be added when the builder can establish
and recheck the exact result directly.

## Consequences and limits

- Static authored mismatches can become ordinary Rust errors, while parsed and
  dynamic frontends continue using `Value<T>` and the same semantic graph.
- The open descriptor permits arbitrary static ranks without making downstream
  code an authority. Misspelled descriptors fail refinement with Tiler's typed
  diagnostic rather than corrupting the graph.
- Exact-shape evidence creates one Rust monomorph per distinct descriptor. The
  1,000-shape result is acceptable for the prototype but must not be generalized
  into an unbounded promise.
- Relationships depending on symbols, runtime bindings, broadcasting case
  splits, or solver derivations remain graph proof capabilities.
- Evidence types and witnesses remain absent from semantic and artifact
  identity and cannot select a physical specialization.

