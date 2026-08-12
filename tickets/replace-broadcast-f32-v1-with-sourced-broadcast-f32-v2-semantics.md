---
id: replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics
title: Replace broadcast-f32 v1 with sourced broadcast-f32 v2 semantics
status: todo
priority: p1
dependencies: [seal-and-validate-sourced-shapes-at-semantic-inference-boundaries, define-the-widening-relation-over-a-symbolic-broadcast-extent, resolve-semantic-shape-inference-over-symbolic-extents]
related: [resolve-semantic-shape-inference-over-symbolic-extents]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, semantics, broadcast, identity]
---
# Replace broadcast-f32 v1 with sourced broadcast-f32 v2 semantics

## User-visible outcome

One governed broadcast occurrence can declare a positive symbolic result extent, including a symbol that later binds to one, without changing semantic operation count or silently becoming a different operation.

## Work

- Complete-replace `tiler::broadcast-f32@1` with `@2`; do not retain a compatibility registration or two built-in implementations.
- Make the mapping's declared extents sourced and give the changed grammar a new mapping domain. Reuse the shape vocabulary's canonical symbol authority rather than independently encoding a second symbol language.
- Preserve the current literal canonicality rules exactly. For symbolic mappings, require exact-environment admission and positivity; prove `FromOperand` equality and the unit source of `StretchUnit` through the same environment.
- Split context-free syntax validation from environment-dependent application. Only the semantic builder that owns the program environment may mint an occurrence and result.
- Batch validation over one transient environment solve plus an `O(rank)` walk. Do not retain a derived cache or call a fresh solver once per axis.
- Replace the governed definition, facade, conformance population, registry row, law/provider references, documentation, identity ledgers, and pins that own the operation-key change.
- Keep zero, undeclared/foreign symbols, unproved equality, and literal-one many-to-one mappings as typed named refusals.

## Identity

The v2 key and mapping domain deliberately move every governed broadcast semantic identity and every downstream selected subject that reaches one. Unrelated operations and the admitted v1-free population must remain byte-stable. No artifact-schema step follows merely from receiving changed nested semantic identity bytes.

## Acceptance

- Literal extent one still fails under the existing canonicality rule while its reindex neighbour succeeds.
- Symbolic extents proven over `[1, upper]`, `[2, upper]`, and exactly one are admitted; a symbol whose lower bound is zero is refused before graph mutation.
- `FromOperand` and `StretchUnit` equality/unit proofs fail independently under subject perturbations.
- One mapping validation performs one environment solve, with a counted high-rank population that would expose per-axis solving.
- The old governed key is absent from the standard registry and source census; the new key/domain/pins are coherent.

## Stop conditions

Stop if sourced-shape sealing changes admitted canonical bytes, if semantic construction would require a second environment authority, or if one shared sourced-extent encoding cannot be used without an unreviewed public-boundary expansion.
