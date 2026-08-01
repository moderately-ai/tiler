---
id: admit-a-fusion-role-for-the-tensor-contraction
title: Admit a fusion role for the tensor contraction
status: todo
priority: p2
dependencies: [realize-the-contraction-through-the-appendable-direct-path]
related: [realize-the-strict-contraction-on-metal, admit-reassociated-contraction-schedule-alternatives]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, contraction]
---
## User-visible outcome

A cover region holding a tensor contraction beside another operation derives its fusion legality from a declared role instead of failing closed to `Unknown`, so the support matrix's R5 criterion is met for the family rather than skipped.

## Why this ticket exists

**Fact — the direct path reached R6 without R5's own criterion, and said so.** `realize-the-contraction-through-the-appendable-direct-path` carried a whole-program contraction from `compile()` to an emitted Metal entry point on 2026-08-01. It registered no `FusionOperationRole`: `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs` maps `constant-f32`, `multiply-f32`, `add-f32`, `silu-f32`, `strict-serial-sum-f32`, `rms-norm-f32`, `reindex-f32`, and `broadcast-f32`, and no contraction key.

**Fact — the omission is invisible today, and that is the reason it is filed rather than fixed there.** `crates/tiler-compiler/src/pipeline/planning.rs` skips `derive_fusion_legality` for any cover region with fewer than two members, and the recognized contraction shape is exactly one operation, so no reachable program consults a contraction's role. Registering one during that ticket would have been a declaration with no consumer and no evidence — the mirror image of the K-multiple refusal that ticket explicitly refused to ship.

**Inference — what makes it load-bearing is a second operation in the region, and two tickets bring one.** A pointwise epilogue fused into a contraction, or a prologue fused in, is a region the recognizer does not yet build; when it does, `classify` returning `None` is what stops the plan. `admit-reassociated-contraction-schedule-alternatives` and the epilogue work Milestone 6 frames are the consumers.

## Required delivery

- One declared role for `tiler::strict-tensor-contraction-f32@1`, derived rather than borrowed. `OrderedReduction` and `PrologueCarryingOrderedReduction` both describe a fold over *one* contributor domain; a contraction folds a product of two operands read at different coordinates, so state whether that is a third role or a widening of the second, and say which obligations move with the choice.
- The legality obligations the role carries, stated per obligation rather than inherited: what ADR 0015 contraction permission means for a region that fuses a multiply into this fold, what reassociation and permutation mean for it, and which of these the existing `ArithmeticContraction` obligation already decides.
- A fixture whose cover region actually holds two members including the contraction, so the role has a reachable consumer and the `Unknown` it replaces was observed first.

## Non-goals

The `tiled` realization; contraction-order exploration; GEMM recognition; layout-conversion costing; the multi-operand structure reserved behind ADR 0087's fifth rule.

## Closes when

A cover region containing a contraction and at least one other operation derives `FusionLegality::Legal` or a typed rejection instead of `Unknown`, the `Unknown` it replaces was watched firing first, and the support matrix's contraction row records R5 as met rather than skipped.
