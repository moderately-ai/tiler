---
id: unify-schedule-index-region-with-verified-index-region
title: Unify schedule bounded index region with tiler_ir::index::VerifiedIndexRegion
status: todo
priority: p2
dependencies: []
related: [prototype-scheduled-region-ir]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, refactor]
---
`tiler_ir::schedule` introduced its own bounded `IndexRegion` (iteration domain,
accesses, bounds/ownership proofs, scalar program, numerical realization) rather
than composing the existing `tiler_ir::index::VerifiedIndexRegion`. Two
"index region" concepts now coexist in one crate, with parallel bounds/ownership
proof descriptors and their own witness newtypes.

Unify them so a scheduled region references a `VerifiedIndexRegion` (or a shared
bounded projection of it) instead of duplicating the description. The intrinsic
schedule verifier must keep proving schedule-specific facts (launch/domain
coverage, tail legality, reduction topology agreement) but should not re-derive
the index-region invariants the `index` module already establishes.

Deferred deliberately by `prototype-scheduled-region-ir` because the bounded slice
did not need the full `VerifiedIndexRegion`; record whether the unified form must
preserve the schedule module's current canonical identity bytes or may re-baseline
them (identity is currently a pure function of the schedule's own descriptors).
