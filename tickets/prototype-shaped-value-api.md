---
id: prototype-shaped-value-api
title: Implement checked shaped values and shape witnesses
status: review
priority: p0
dependencies: [prototype-shape-evidence-spike, research-the-public-static-shape-evidence-spelling, spike-nightly-arbitrary-rank-shape-evidence]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/shapes, contracts/navigation]
shared_scopes: [project/tickets, contracts/foundation, contracts/decisions]
paths: [Cargo.lock]
tags: [implementation, prototype, semantics, shapes, rust-api]
claimed_from: todo
assignee: codex
lease_expires_at: 1784601838
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

## Outcome

- Added sealed `Rank<RANK>` and the accepted dependent-array
  `StaticShape<RANK, EXTENTS>` evidence under the shape vocabulary.
- Added privately constructed, zero-overhead `ShapedValue<T, E>` refinements
  with explicit weakening and typed mismatch/ownership failures on both draft
  builders and completed programs.
- Added graph- and ordered-subject-bound `ShapeWitness<SameShape>` capabilities
  with typed proof and validation failures.
- Added evidence-preserving scalar constant, equal-evidence pointwise, and
  scalar-broadcast facades plus exact-to-rank weakening. They delegate to the
  existing builder admission path and validate typed result arity, value type,
  and shape evidence before committing any graph mutation.
  Shape-changing runtime-attribute calls return unrefined values while callers
  and frontends retain the general checked-refinement path for concrete output
  evidence.
- Added runtime, canonical-identity, and downstream compile-pass/fail coverage
  for refinement, evidence loss, foreign graphs and witnesses, subject
  mismatch, forgery, sealed evidence, unequal static shapes, and invalid
  rank/array length.

The implementation is ready for consequential public-interface review. It is
not a claim that these experimental names or call-site details are stabilized.
