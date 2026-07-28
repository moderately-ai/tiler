---
id: promote-the-symbolic-index-profile-to-a-public-boundary
title: Promote the sourced-extent index profile to a reviewed public boundary
status: awaiting-decision
priority: p2
dependencies: []
related: [implement-shapeenv-index-bindings, implement-shapeenv-core]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, api]
---
`implement-shapeenv-core`, `implement-shapeenv-constraints`, and `implement-shapeenv-index-bindings` each landed under ADR 0074 convention 7: implemented, tested, and `pub(crate)`, because each ticket states that "any consequential public or cross-crate boundary remains a draft until Tom reviews and accepts the exact implementation commit". Three drafts now exist and none is reachable outside `tiler-ir`.

**What is currently crate-internal.** `crate::shape::env` (scoped symbols, typed root bindings, the constraint environment, identity); `crate::index::sourced` (`SourcedExtent`, the phase ceiling, `ExtentSources`); `IndexRegionBuilder::with_shape_environment` and `symbolic_dimension`; `VerifiedIndexRegion::extent_sources`; and `DomainDimensionRef::sourced_extent`, which is the additive borrowed view `docs/ir.md` reserved beside `static_extent()`.

**Why this is a ticket and not a mechanical change.** Promotion is the point at which the boundary becomes a compatibility commitment, and several shapes in it were chosen to be cheap to revise while private. Named examples: whether `with_shape_environment` stays a consuming builder step or becomes a `new`-time argument; whether `SymbolicExtentError` stays a separate error type or folds into `IndexBuildError`; whether `DomainDimensionRef::sourced_extent` is the right additive view or a narrower `symbol()` accessor is; and whether `ShapeEnv` is exported from `tiler_ir::shape` or gets its own module. Each is an ADR 0075 always-ask category once public.

Every module involved carries a `dead_code` allow whose stated reason is exactly this draft status. Promotion removes those allows; it must not be done by adding a caller that exists only to satisfy the lint.

## Closes when

Tom has reviewed the exact boundary, the accepted subset is `pub` with its
documentation and `#[non_exhaustive]` decisions made, and draft `dead_code`
allows are removed or narrowed for every promoted item. A still-private
reservation may keep an item- or submodule-level allow whose reason names the
unavailable producer or consumer and reopening trigger; a whole-file draft
allow must not survive merely because some reserved work remains. `make full`
passes.

## Decision — Tom, 2026-07-25

**Approved: promote.** ADR 0075 reserves public-surface promotions to the owner; this one is granted.

Covers all three ShapeEnv drafts — `shape::env`, `env::constraint`, and `index::sourced`. The authority, its constraint environment, and its first consumer are complete and tested, and were unreachable outside `tiler-ir`. Note the fragment boundary is still being widened by `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients`, so promote the surface that is settled and say plainly which parts are still moving.

## Parked 2026-07-27 — awaiting Tom

**The question is four questions, and each is an ADR 0075 always-ask once public:**

1. Does `with_shape_environment` stay a consuming builder step or become a `new`-time argument?
2. Does `SymbolicExtentError` stay a separate error type or fold into `IndexBuildError`?
3. Is `DomainDimensionRef::sourced_extent` the right additive view, or a narrower `symbol()` accessor?
4. Is `ShapeEnv` exported from `tiler_ir::shape`, or does it get its own module?

Three implemented, tested `pub(crate)` drafts exist and none is reachable outside `tiler-ir`. Each shape above was chosen to be cheap to revise *while private*, so promoting without deciding them would spend that option for nothing.
