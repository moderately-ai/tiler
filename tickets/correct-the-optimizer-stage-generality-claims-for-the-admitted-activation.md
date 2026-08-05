---
id: correct-the-optimizer-stage-generality-claims-for-the-admitted-activation
title: Correct the optimizer stage-generality claims for the admitted activation
status: in-progress
priority: p2
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [docs, optimizer, correction]
claimed_from: todo
assignee: agent-opt-claims
lease_expires_at: 1785934519
---
## User-visible outcome

`docs/compiler/optimizer.md` stops telling a reader that `tiler::silu-f32@1` is refused at the request boundary, which it is not since 2026-08-04.

## The exact stale spans

**Fact — audited by reading the file at the commit that admitted the activation.** Two sections make claims the tree now refutes:

1. **"What each stage is general over today", the stage-1 paragraph.** It says the boundary "refuses a vocabulary, not a shape: an operation no region can spell, …". That sentence survives, but the enumeration of *which* operations no region can spell has moved: `tiler::silu-f32@1` is now projected into `PointwiseF32Node` from `crates/tiler-compiler/src/elementary.rs`'s single statement of its composition, and only `tiler::reindex-f32@1` and `tiler::broadcast-f32@1` remain refused — because what they lack is a `LogicalAccess` spelling of their access relation rather than a per-point body.

2. **"Lowering capability resolution and index-region refinement", maturity boundary.** Its claim that the resolution/refinement pair is unconditional is unaffected. What is newly true and unstated is the *composition* the activation's admission established: because the boundary's projection and the governed index-access lowering are driven from one statement, refinement's proof that the emitted region realizes the occurrence is also evidence about the projection. That was observed rather than argued — perturbing the shared statement was watched failing at `compile.lowering.refinement-refused` before any region was scheduled — and it belongs in this contract because it is the reason a projecting boundary does not create a second unchecked authority.

There is a third addition the file does not currently have anywhere: **a program carrying a registered elementary family now places a hard accuracy obligation on its target.** `request::require_elementary_accuracy` assesses each distinct obligation per target, before numerical-contract resolution, and refuses with the refusing authority's stable key. That is feasibility rather than cost and belongs beside the four surfaces the optimizer may consult.

## Why this is a separate ticket

`ticketsplease.toml` maps `docs/compiler/**` to `contracts/optimizer`. The ticket that admitted the activation declares `implementation/compiler` exclusively, and `contracts/optimizer` was held by a live sibling at the time, so the correction could neither be made from that branch nor scheduled against a scope another worker held.

## Closes when

The stage-1 enumeration names the two families that still refuse and the reason each does, the refinement section states the composition the shared statement establishes, and the elementary accuracy obligation is stated as a feasibility question with its own refusal — each with the exact code location a reader can check it against.
