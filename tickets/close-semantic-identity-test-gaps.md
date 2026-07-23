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

- add negative tests for the governed inferencers' untested rejection paths: `sum.axes.range` (axis at or beyond operand rank) and the f32 constant payload rejection (non-`FloatBits` format or wrong byte width); only `binary.shape`, `sum.axes.empty`, and `sum.axes.canonical` are currently tested;
- add a canonical-identity construction-order regression test that builds the same live DAG in two different valid topological insertion orders and asserts identical identity bytes; the existing test varies only dead-operation insertion position;
- `CanonicalValue::value_type`, `TypeDefinitionFacts::new`, `OperationDefinitionFacts::new`, and `OperationConformance::new` remain unvalidated `const` constructors that can carry over-bound values past the structural depth/node limits into facts wrappers and durable registry identity. Close this by **bounding at registration**, not by making the constructors fallible: they have live cross-crate call sites in `crates/tiler-compiler/src/request.rs` and `crates/tiler-reference/src/lib.rs`, so a fallible signature would change a public boundary and require scopes this ticket does not declare; and
- `ValueTypeDefinitionKey`'s derived `Ord` makes durable registry-identity byte order depend on Rust enum declaration order; replace it with an explicit `Ord` implementation. Note `ValueTypeDefinitionKey::encode` already emits stable discriminants 1/2/3, so the encoding is not at risk — only the BTreeMap iteration order is. The explicit order **must preserve the current derived order** (Nominal < Parameterized < EncodedNumeric) so existing registry-identity bytes do not change; add a regression fixture asserting byte stability.

Under the bound-at-registration option all changes stay inside tiler-ir and its tests; no public-boundary changes are expected, but any that emerge remain drafts until Tom reviews them. Sequence after the typed-explain branch merges — both hold `implementation/ir`, and the explain work touches the same compiler-side call sites this hardening ripples through.
