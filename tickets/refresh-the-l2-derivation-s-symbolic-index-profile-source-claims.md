---
id: refresh-the-l2-derivation-s-symbolic-index-profile-source-claims
title: Refresh the L2 derivation's symbolic index profile source claims
status: todo
priority: p2
dependencies: []
related: []
scopes: [research/shapes]
shared_scopes: []
paths: []
tags: [documentation]
---
## User-visible outcome

[The L2 derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md)'s *Extent classes* section states the symbolic index profile's public standing from inspected source, and that statement matches `crates/tiler-ir/src/index/` today.

## The defect, found on 2026-08-06 while refreshing L2's operation-family standing

**Fact.** The paragraph beginning "**Fact — the semantic layer can already state these bounds, and the public index layer cannot yet carry them**" makes four source claims and all four are stale at `b913165b`:

- `DomainDimensionRef::sourced_extent` and `TensorRef::sourced_shape` are named as `pub(crate)`; neither symbol exists under that name anywhere in `crates/tiler-ir/src/`.
- `VerifiedIndexRegion::extent_sources` is named as `pub(crate)`; it is `pub` at `crates/tiler-ir/src/index/model.rs:285`.
- Each is said to carry an `#[allow(dead_code)]` whose `reason` records that the symbolic profile is a crate-internal draft; no such attribute is on any of them.
- The quoted doc comment "No public constructor produces a symbolic dimension yet, so every region a public caller can build still answers `Some` for every dimension" appears nowhere in `crates/`.

**Fact.** [`promote-the-symbolic-index-profile-to-a-public-boundary`](promote-the-symbolic-index-profile-to-a-public-boundary.md) is `done`, which is what moved the boundary the paragraph describes. The *Inference* that follows it — that the workload's extent requirement is a promotion rather than a new capability, and that without it every `T` and every `S` is a separate compiled artifact — is a derivation about the workload rather than a source reading, and needs re-reading against the delivered boundary rather than assuming it survived.

## Why this is a separate ticket

[`refresh-the-l2-derivation-operation-family-standing`](refresh-the-l2-derivation-operation-family-standing.md) owned that record's *operation-family standing* and named its nine stale sites; this paragraph is about the index layer's public surface, not about where a family sits on the support matrix, and settling it needs a full read of `crates/tiler-ir/src/index/` that the family refresh did not take. Correcting it inside that ticket would have stated a source fact on a grep rather than on a reading.

## The work

Read `crates/tiler-ir/src/index/{model,sourced}.rs` and `crates/tiler-ir/src/shape/env/constraint.rs` in full, and the promotion ticket's delivered surface. Restate the paragraph as a dated **Correction** in the record's own convention — quote each stale clause, state what is true now, state the bound — rather than rewriting it silently. Then re-derive the *Inference* beneath it against the delivered boundary: state whether a distinct `T` or `S` still forces a separate compiled artifact, and cite what decides it.

## Closes when

Every source claim in that section is verified against `crates/tiler-ir/src/index/` by a full read, with the stale clauses preserved as a dated correction and the extent-promotion inference re-derived or restated with its bound.
