---
id: admit-symbolic-extents-through-compiler-region-formation
title: Admit symbolic extents through compiler region formation
status: in-progress
priority: p1
dependencies: [admit-symbolic-extents-at-the-compiler-request-boundary]
related: [admit-live-extent-operands-to-payload-indexing, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, extents]
claimed_from: todo
assignee: worker-admit-symbolic-region-formation
lease_expires_at: 1786640549
---
## User-visible outcome

`compile()` of a symbolic semantic program is no longer stopped at the first strategy-selection refuse. A recognized symbolic program reaches region formation and either produces a scheduled region that still names its symbols or declines with a typed reason for the unsupported case.

## Exact gap

**Fact at `209e0f9fd5a18486039d859a5f47ccf260f0f8cf`, re-read this session.** [`admit-symbolic-extents-at-the-compiler-request-boundary`](admit-symbolic-extents-at-the-compiler-request-boundary.md) made a symbolic program reach strategy selection. Recognition still declines the first non-static extent as `RequestError::UnsupportedSymbolicExtent { phase: "strategy", rule: "symbolic-extent", extent }` from `static_shape` in `crates/tiler-compiler/src/request.rs`. Durable anchor: `A symbolic extent is refused here rather than resolved through the environment`.

**Fact, same base.** Later fail-closed gates still refuse a symbolic shape if reached: normalization `NormalizeError::Structure { rule: "symbolic-extent" }` in `crates/tiler-compiler/src/normalize.rs`, and region-graph construction `RegionError::Structure { rule: "symbolic-extent" }` from `value.shape().as_static()` in `crates/tiler-compiler/src/region.rs`. The normalization helper documents that it is not the compile path's first refusal.

**Imprecise, repaired from the live-extent review comment.** The comment on [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) said `compile()` declines at region formation. That is the later gate, not the first. The first refuse on today's compile path is strategy selection. Region formation would refuse if a recognizer ever returned a symbolic region. The working live-extent draft path is `ScheduledRegionBuilder` + `lower_scheduled_region`, which bypasses `compile()`.

This ticket owns the compile path, not the labelled kernel operand.

## Required work

- Re-audit `static_shape` / `static_shape_ref` in `request.rs`, every strategy recognizer that requires a fixed `Shape`, `normalize.rs` `static_shape`, and `RegionGraph::from_program` at the exact base before editing.
- Let a named, bounded population of recognizers accept a symbolic input or result shape without folding `ExtentSources::determined` into the logical plan. Unrecognized or unlowerable symbolic cases still decline as `UnsupportedSymbolicExtent` naming the extent, never as a mis-attributed handle or signature rule.
- Carry the program's own environment through any rewrite so a rebuilt symbolic value cannot lose or swap the environment its identity folds.
- Teach region-graph construction to record a symbolic value instead of requiring `as_static()`. A hole in the graph is not an answer.
- Keep later payload compilation allowed to decline until the live-extent operand and envelope exist. This ticket does not invent those surfaces and does not claim a symbolic payload is executable.
- If a recognizer, region-graph record, or compile facade needs a new public type, produce the labelled draft and stop for Tom.

## Required evidence

- A symbolic elementwise neighbour that today's strategy refuses now reaches region formation, and its literal neighbour still compiles.
- A still-unsupported symbolic case declines with `UnsupportedSymbolicExtent` naming the extent. Remove the new path and watch the old strategy refuse return.
- A rewrite perturbation that would mint a symbolic value without the program's environment fails as invalid compiler output.
- `RegionGraph::from_program` no longer dies at `as_static()` for the admitted population; a value whose shape is still unrepresentable fails with a named rule.
- Targeted compiler tests, rustdoc, Clippy, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Live-extent kernel operands, Metal `eN` ABI, artifact envelope rows, `N = 14` / `N = 15` pipeline evidence, and lifting `AotRefusal::SymbolicExtent`. Those belong to the sibling remainders and to [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md).

## Closes when

`compile()` of the admitted symbolic population reaches a scheduled region or a typed decline past strategy selection, specialization remains forbidden at the request boundary, and every new check is fail-capable.
