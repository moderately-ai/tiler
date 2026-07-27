---
id: reduce-the-reference-crate-root-to-a-facade
title: Reduce the reference crate root to a public facade
status: todo
priority: p2
dependencies: []
related: [reconcile-index-oracle-ownership-prose]
scopes: [implementation/reference]
shared_scopes: []
paths: []
tags: [refactor, reference, progressive-disclosure]
---
Make `tiler-reference` disclose its public contract before its evaluation
mechanisms.

The crate root currently owns tensor representation, semantic registration and
evaluation, scalar dispatch, index evaluation, arithmetic, errors, and a large
test suite.

## Outcome

Keep a small `lib.rs` that introduces and re-exports the reference boundary.
Separate tensor representation, semantic registry/evaluation, scalar
registry/evaluation, index evaluation, arithmetic, errors, and tests into
shallow files.

Do not merge the semantic and scalar registries merely because both dispatch
behavior; they govern different identities and extension obligations. Preserve
the reference implementation's independence from compiler verification.

## Closes when

The crate root is a legible facade, each evaluation authority has one owner,
public paths and reference results are unchanged, and the full gate passes.
