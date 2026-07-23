---
id: close-semantic-identity-test-gaps
title: Close semantic identity and registry hardening residuals
status: todo
priority: p1
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, hardening]
---
Four adversarially verified residuals from the 2026-07-23 audit that the completed hardening tickets did not cover:

- add negative tests for the `sum.axes.range` (axis at or beyond operand rank) and `constant.bits` (u64 payload exceeding u32) rejection paths of the governed inferencers; only `sum.axes.empty`/`sum.axes.canonical` are currently tested;
- add a canonical-identity construction-order regression test that builds the same live DAG in two different valid topological insertion orders and asserts identical identity bytes; the existing test varies only dead-operation insertion position;
- `CanonicalValue::value_type`, `TypeDefinitionFacts::new`, `OperationDefinitionFacts::new`, and `OperationConformance::new` remain unvalidated `const` constructors that can carry over-bound values past the structural depth/node limits into facts wrappers and durable registry identity; route them through validation or bound them at registration, with tests; and
- `ValueTypeDefinitionKey`'s derived `Ord` makes durable registry-identity byte order depend on Rust enum declaration order; replace it with an explicit stable ordering (or encode a stable discriminant) plus a regression test before any durable serialization consumes the ordering.

All changes stay inside tiler-ir and its tests; no public-boundary changes are expected, but any that emerge remain drafts until Tom reviews them.
