---
id: prototype-semantic-operation-registry
title: Implement the canonical semantic operation registry
status: todo
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

Add deterministic projection of only reached semantic authorities for later
compilation provenance. Keep decomposition, lowering, rewriting, costing, and
target capabilities reserved behind separate registries; do not implement
them here.
