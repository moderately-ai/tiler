---
id: admit-symbolic-extents-through-schedule-formation
title: Admit symbolic extents through schedule formation
status: in-progress
priority: p1
dependencies: [admit-symbolic-extents-through-compiler-region-formation, accept-the-live-extent-operand-public-surface]
related: [deliver-an-artifact-family-from-a-symbolic-region, carry-live-extent-operands-through-the-artifact-envelope]
scopes: [implementation/ir, implementation/compiler, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, ir, shapes, public-boundary]
claimed_from: todo
assignee: worker-symbolic-schedule
lease_expires_at: 1786716570
---
## User-visible outcome

`compile()` of a recognized same-shape symbolic elementwise program produces a scheduled region that still names its symbols — typically a `LiveRowMajor` plan over the declared `[n]` — or declines with a typed reason that is not the current schedule-geometry refuse. Specializing the plan on a representative literal extent remains forbidden.

## Why this exists

[`admit-symbolic-extents-through-compiler-region-formation`](admit-symbolic-extents-through-compiler-region-formation.md) deliberately stopped at schedule. Same-shape symbolic elementwise now reaches region formation; `crates/tiler-compiler/src/pipeline.rs` then returns `RequestError::UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }` unless the program carries a parametric broadcast. Durable anchors: `A sourced broadcast must reach physical selection` and `IndexRegion requires a fixed geometry`.

[`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md) lifted the frontend-local `AotRefusal::SymbolicExtent` gate at `bd9c65dd` so that refuse is what a `deliver macos;` consumer now sees. That ticket cannot form the scheduled region: `IndexRegion.iteration_shape` is a fixed `Shape` (`crates/tiler-ir/src/schedule/model.rs`). Live-extent operands already exist on the hand-built `ScheduledRegion` / `LiveRowMajor` path, not on `session::compile`.

## Required work

- Re-audit `IndexRegion`, `ScheduledRegionBuilder`, `LiveRowMajor`, `pipeline.rs` `first_symbolic_extent` / `carries_parametric_broadcast`, and the frontend compile path at the exact base before editing.
- Form a scheduled region whose launch geometry names the program's symbols. Do not fold `ExtentSources::determined` into the logical plan and do not bake a bound value into plan or artifact identity.
- If `IndexRegion.iteration_shape: Shape` must become a sourced geometry, that is a public IR change. Produce the labelled draft, stop, and file or update a Tom packet rather than inventing the spelling. If an existing carrier (`LiveRowMajor` plus the already-accepted live-extent operand) can express the rank-1 `[n]` case without changing that field, implement only that slice.
- Keep reductions, contractions, staged families, and structural maps refused by name until each has its own admitted geometry. Do not silently reuse the elementwise path.
- Leave Metal emission and the `deliver` identity-across-extents hash to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Required evidence

- The existing `sym n` `(a * b) + c` fixture that today declines at schedule now yields a scheduled region that still names `n`, and its literal neighbour still compiles with unchanged identity bytes.
- Removing the new path restores `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }`. Quote that failure text.
- A rewrite or formation step that would mint a launch over a determined representative extent fails as invalid compiler output.
- Perturb the new geometry independently of the parametric-broadcast exception so a missing broadcast cannot be the only way a symbol reaches a plan.
- Targeted compiler and IR tests, rustdoc, Clippy with warnings denied, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Lifting the frontend refuse again (already gone). Artifact-envelope rows. `N = 14` / `N = 15` pipeline evidence. Teaching `deliver` to embed and hash one artifact across bound extents — that remains the parent ticket after this lands.

## Closes when

`compile()` of the admitted same-shape symbolic elementwise population returns a scheduled region that names its symbols, or a narrower typed decline than `symbolic-extent` at schedule, without specializing on a bound value.

## Dependency correction — 2026-08-13

The former dependency on [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) was too broad. This schedule ticket needs the already-accepted `LiveRowMajor` / kernel live-operand spelling, not the later artifact/backend proof that has now been reopened. It therefore depends directly on [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md). This avoids a false cycle: [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md) must consume the schedule carrier produced here before a symbolic artifact interface can be validated.
