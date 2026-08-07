---
id: refresh-the-l2-derivation-s-symbolic-index-profile-source-claims
title: Refresh the L2 derivation's symbolic index profile source claims
status: review
priority: p2
dependencies: []
related: []
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-l2-symbolic
lease_expires_at: 1786078751
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

## Outcome — 2026-08-07

**Scope, stated rather than assumed.** The ticket declared `research/shapes` alone and `project/tickets` was absent; it is added above, because closing this ticket edits its own file. No `crates/` path is touched — the change is `docs/research/shapes/transformer-operation-and-shape-surface.md` and this ticket — so the workspace gate is untouched and no carry reasoning is owed.

**The read the correction rests on.** `crates/tiler-ir/src/index/{mod,model,sourced,builder,predicate}.rs`, `crates/tiler-ir/src/shape.rs`, `crates/tiler-ir/src/shape/env.rs`, and `crates/tiler-ir/src/shape/env/constraint.rs` read in full or at every site the claims touch, plus `crates/tiler-ir/src/semantic/program.rs` and `crates/tiler-macros/src/{region,aot}.rs` for the re-derivation, and the delivered surface recorded by `promote-the-symbolic-index-profile-to-a-public-boundary` and `carry-symbolic-extents-into-the-semantic-program`.

**Old claims → new, one line each.** `DomainDimensionRef::sourced_extent` and `TensorRef::sourced_shape` `pub(crate)` → neither exists; the optional-accessor pair was replaced by the total views `DomainDimensionRef::extent` (`model.rs:722`) and `TensorRef::shape` (`model.rs:758`), which is the substance and not a rename. `VerifiedIndexRegion::extent_sources` `pub(crate)` → `pub` at `model.rs:285`. Three `#[allow(dead_code)]` draft markers → none; the only allowance left in the index layer is on `predicate.rs:416`'s `expression_class_is_stateable` and is unrelated. The quoted doc comment "No public constructor produces a symbolic dimension yet…" → absent from `crates/`, because `IndexRegionBuilder::new_with_shape_environment` (`builder.rs:654`), `symbolic_dimension` (`builder.rs:815`), and `sourced_tensor` (`builder.rs:724`) are public and one of them does. The surviving first sentence was re-verified rather than carried: `ExtentInterval` is at `constraint.rs:820` and `seed_domains` opens every class at `0..=MAX_EXTENT` at `constraint.rs:1167`.

**Three bounds added, because the visibility alone is not the claim.** A `ShapeSymbol` reaches an index *expression* only as a floor-division or modulo divisor — `linear_combination` takes `IndexInteger` coefficients and constant (`builder.rs:1019`) — so no coordinate can be offset or scaled by an extent symbol; `EXTENT_PHASE_CEILING` is `AvailabilityPhase::LiveDevicePreflight` (`sourced.rs:136`) and a later binding returns `SourceTooLate`; and an unbounded extent is retained as an `InsufficientFacts` obligation while a merely *guarded* divisor positivity refuses with `DivisorNotProvedPositive`.

**Re-derivation verdict: delivered at the index layer, and the consequence the *Inference* drew from it is refuted.** A distinct `T` or `S` still forces a separate compiled artifact, and the promotion did not change that, so the derivation named a necessary cause and treated it as sufficient. What decides it: `SemanticProgramBuilder::input`/`input_resolved` still take a fixed `Shape` (`crates/tiler-ir/src/semantic/program.rs:493`, `:516`); the inline frontend returns `ProgramEvidence::DeferredSymbolicExtent` without building a program (`crates/tiler-macros/src/region.rs:915`); `deliver` over a symbolic region refuses with `AotRefusal::SymbolicExtent` (`crates/tiler-macros/src/aot.rs:409`); and all seven delivery tickets `carry-symbolic-extents-into-the-semantic-program` filed are `todo`, the last being `deliver-an-artifact-family-from-a-symbolic-region`, whose own outcome is one artifact identity across every bound extent. The floor is worse than the record stated rather than better: a symbolic region does not compile per extent, it does not compile at all.

**The neighbouring KV-append *Inference* was stale in the same class, and is corrected.** It read a *contract* sentence as a source fact: `docs/ir.md`'s bounded initial vocabulary admits addition and multiplication by a parameter-only expression, and the implemented index layer admits a symbol only as a divisor, so `i + (S − T)` is not expressible today. Its deeper staleness is that the mechanism was eliminated — the KV-append row's own 2026-08-04 correction removed the windowed write, the append is a value-producing `tiler::concatenate-f32@1` at the program boundary, and its extent arithmetic is `ExtentRelation::AdditiveEquality` (`admit-an-additive-extent-relation`, `done`, public through `tiler_ir::shape`) rather than a coordinate offset. Nothing lowers that family, so no index region expresses the append at all; whether its eventual law needs a symbolic offset is left open rather than pre-answered. The clause the paragraph existed to make survives: `admit-semi-affine-index-expression-class` is not a prerequisite, now because it is `done` and delivered the divisor this workload never uses.

**One sibling corrected outside the section, deliberately.** The third bullet of *What this record hands to another rung rather than filing* repeated the same evidence and named `promote-the-symbolic-index-profile-to-a-public-boundary` and `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients` as its live recipients; both are `done`. Left alone it would have reproduced exactly the failure this record calls out two sections earlier — the uncorrected twin that made the record disagree with itself for two days — so it carries a dated note redirecting the evidence to the delivery chain. Nothing else in the record repeats these claims.

**Flagged, not filed.** `docs/ir.md:1037` states that the bounded initial index vocabulary admits "multiplication by a parameter-only expression" beside division and modulo by one; the implementation admits only the divisor form. No ticket owns that divergence — the nearest is `admit-live-extent-operands-to-payload-indexing` (`todo`), which owns the payload-consumable half. No ticket is filed because the contract sentence states an *admitted* vocabulary rather than an implemented one, which is the corpus's normal ordering; if it should instead carry an explicit maturity label, that is a `contracts/foundation` edit outside this ticket's scopes and the coordinator's to route.

**Checks.** `tkt lint` clean, `git diff --check` clean, `tkt guard` reports affected scopes equal to declared. No `crates/`, `Cargo.*`, `Makefile`, or config path is touched, so the workspace gate is untouched by construction and none was run.
