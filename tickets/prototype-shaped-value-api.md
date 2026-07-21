---
id: prototype-shaped-value-api
title: Implement checked shaped values and shape witnesses
status: todo
priority: p0
dependencies: [prototype-shape-evidence-spike, research-the-public-static-shape-evidence-spelling, spike-nightly-arbitrary-rank-shape-evidence]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/shapes]
shared_scopes: [project/tickets, contracts/foundation, contracts/decisions]
paths: [Cargo.lock]
tags: [implementation, prototype, semantics, shapes, rust-api]
---
ADR 0067 accepts the pinned-nightly dependent-array spelling. Its retained
conformance harness now passes and the repository toolchain policy uses the
governed exact pin. Implement the accepted `StaticShape<RANK, EXTENTS>` evidence
in `tiler-ir`.

- add the accepted sealed or host-controlled `ShapeEvidence` vocabulary;
- add privately constructed `ShapedValue<T, E>` refinements over `Value<T>`;
- add graph-owned typed witnesses for the selected multi-value predicates;
- make weakening explicit and zero-cost and refinement checked and fallible;
- propagate evidence only for unambiguous operation relationships and
  revalidate it against canonical result shapes; and
- cover forgery, foreign graph/witness, invalid refinement, evidence loss,
  compile-fail diagnostics, and identity exclusion.

Refined and unrefined calls must use one builder-centered semantic admission
path and produce identical canonical nodes. Rust markers, const parameters, and
proof objects never enter semantic identity or direct physical specialization.
Do not implement a second shape solver, authoritative open marker traits, or an
independent fluent operation API.
