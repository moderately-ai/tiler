---
id: prototype-semantic-operation-registry
title: Implement the canonical semantic operation registry
status: done
priority: p0
dependencies: [prototype-semantic-type-authority-v2]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/extensions]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions]
paths: []
tags: [implementation, semantics, registry, operations]
---
Implement ADR 0044's canonical semantic-operation path before compiler work.

- add validated `OpKey`, bounded operation schemas and canonical attributes,
  initial pure effects, immutable normative/conformance identity, and checked
  deterministic inference/validation;
- store operation keys and canonical attributes in the graph rather than a
  closed public operation enum;
- provide one transactional erased `apply` path that resolves the frozen
  semantic authority, validates operands, and exclusively derives result
  types/shapes;
- register constant, multiply, add, and strict serial Sum through the governed
  standard provider, with an external operation proof using the identical
  path;
- expose typed facades only as wrappers over `apply`, and reject missing
  operation/type support without ambient promotion or caller-declared results;
  and
- version semantic-program identity around `OpKey`, canonical attributes,
  resolved signatures, and numerical contracts while keeping provider
  implementations out of graph identity.

Add deterministic projection of reached semantic definitions and their
admission-provider provenance for later compilation. ADR 0072 subsequently
separates those two identity subjects. Keep decomposition, lowering,
rewriting, costing, and target capabilities reserved behind separate
registries; do not implement them here.

## Outcome

Implemented the canonical semantic-operation path in `tiler-ir`. Durable graph
nodes now store `OpKey`, bounded canonical attributes, ordered operands, and
registry-derived results. Host-owned schemas check arity and attribute kinds
before immutable provider inference, and the host revalidates all inferred
types, shapes, and result counts before a transactional commit.

The governed constant, multiply, add, and strict serial Sum definitions carry
explicit numerical facts and conformance identities. External operations use
the identical registration and graph-admission path. Program identity now
preserves multi-result sharing and includes only deterministically projected
reached semantic authority. Tests cover external authority, missing/invalid
applications, rollback, projection, and multi-result identity; strict Clippy,
tests, and doctests pass.

The later `correct-semantic-identity-layering` ticket owns the corrective
transitive projection across nested types, definition facts, schema defaults,
facts, conformance values, and occurrence attributes. The projection added by
this completed slice was an initial boundary, not the final proof that every
admission-time authority is retained downstream.
