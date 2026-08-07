---
id: relocate-the-sourced-extent-vocabulary-to-the-shape-module
title: Relocate the sourced-extent vocabulary from the index module to the shape module
status: in-progress
priority: p1
dependencies: []
related: [carry-symbolic-extents-into-the-semantic-program, promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, extents, api]
claimed_from: todo
assignee: agent-relocate
lease_expires_at: 1786132480
---
## User-visible outcome

The crate's one constant-or-symbol extent vocabulary lives where its components live, so the semantic layer can consume it without minting a second copy. The index layer's accepted behaviour, identity bytes, and diagnostics are unchanged.

## Why this exists

**Fact.** `SourcedExtent`'s own documentation calls it "the crate's *one* constant-or-symbol vocabulary for an index-layer magnitude", and states the ground: "a second divisor enum would give a frontend two ways to spell the same fact, two encodings to fold into identity, and two places to extend when a third source kind arrives." Reproduce with `grep -n "constant-or-symbol vocabulary" crates/tiler-ir/src/index/sourced.rs`, which returns line 144.

**Inference.** The scoping phrase "index-layer" is a fact about where the type currently sits, not about what it is: `SourcedExtent` is `Extent | ShapeSymbol` and both live in `crate::shape`. [The symbolic-semantic-extents record](../docs/research/shapes/symbolic-semantic-extents.md) eliminates a semantic-layer mirror on exactly this ground, which leaves relocation as the only way for two layers to share one vocabulary.

## Implementation keys

- Move `SourcedExtent`, `SourcedShape`, `ExtentSources`, `ExtentSourceError`, `SymbolicExtentError`, and `EXTENT_PHASE_CEILING` into `crate::shape`, re-exported flat as the `ShapeEnv` vocabulary already is.
- Decide and state whether `tiler_ir::index` keeps re-exports at the accepted paths or whether callers move. Either is defensible; leaving both live without saying which is canonical is not.
- Change no encoding. `SourcedExtent::encode`, `SourcedShape::encode`, and their `encoded_len` companions must produce identical bytes, so no identity domain moves. A domain that advanced for a relocation alone would make two identical subjects carry different domains, which the index promotion already refused for a visibility change.
- `EXTENT_PHASE_CEILING`'s documentation currently derives the boundary case by inference from a rule the accepted contract states for *semantic* extents. Once the type is shared, that inference becomes the quoted case for one of its two consumers; correct the comment rather than leaving it describing the narrower situation.

## Evidence

- Every existing `tiler-ir`, `tiler-compiler`, and `tiler-reference` test passes unchanged, including the index-region identity assertions.
- A canonical-bytes comparison over a symbolic and a static region before and after the move, showing equality.
- `git diff --check`, `tkt lint`, per-package Clippy with warnings denied, `make full`.

## Public boundary

The exported paths change, which is ADR 0075's always-ask category. Tom accepted the current paths on 2026-07-31; this is a re-acceptance and the packet must say so, listing the before and after path for each of the six items.
