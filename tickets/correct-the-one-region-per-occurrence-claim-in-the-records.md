---
id: correct-the-one-region-per-occurrence-claim-in-the-records
title: Correct the one-region-per-occurrence claim where four records still state it
status: todo
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [contracts/navigation, research/numerics, research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## What is stale

`GovernedRootMeanSquareScaleF32` landed in `crates/tiler-compiler/src/governed.rs` on 2026-08-06, so the governed profile no longer emits exactly one region per occurrence and `tiler::rms-norm-f32@1` has refinement evidence through the ordinary path. Four records outside the implementation scopes still state the superseded claim, and each states it as a *blocker* rather than as history:

- `docs/roadmap.md`, the normalization row (`tiler::rms-norm-f32@1`): its R6 cell says R6 "additionally needs an index-access lowering capability … while `GovernedIndexAccess` emits exactly one region per occurrence", and its Fact column says the family "realizes as two regions where the boundary assembles one per occurrence". The first is now supplied. **Corrected 2026-08-06 by [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md): the row's *other* named blocker has moved too.** `select_supported_strategy` no longer refuses this key under `operation-set` — recognition is law-derived and the family carries `StagedRootMeanSquareScaleF32` — so a normalization program now reaches region formation, the cover search, and its own index-access refinement through `compile()`. The remaining blocker is one layer further down and belongs to [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md): no `ScalarProgram` spells a stage of the law's realization, so every region declines under `region-staged-family-unspellable`. The row's R6 cell must name *that*, and both of the recognizer sentences in the Fact column are now history rather than a wall.
- `docs/roadmap.md`, the softmax row: the same clause, and it stays a genuine blocker for that family only because `tiler::softmax-f32@1` has no registered law at all (which needs the governed maximum key). The reason should name the law, not the one-region limit.
- `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md:413` — "where `GovernedIndexAccess` emits one region per occurrence, which [`lower-a-two-region-occurrence-through-one-index-access-capability`] owns".
- `docs/research/semantic-graph/operation-family-delivery-graph.md:58` — the M5/M6 derivation attributes the normalization's and the softmax's R5 rungs to the same one-region-per-occurrence limit, which is now true of neither.

## Why it is a separate ticket

`docs/**` is `contracts/navigation`, `research/numerics`, and `research/semantic-graph`; the landing that made these claims stale was scoped to `implementation/ir` and `implementation/compiler` and could not edit them. The in-crate documentation *was* corrected in that landing — `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` and `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs` — so the divergence is between the crates and the records rather than inside either.

## What this must do

Rewrite each claim in place to current truth, in tense where the superseded reading was a stated blocker for a period (the roadmap's own convention). Name the *actual* remaining blocker per family: for the normalization, request-boundary recognition alone; for the softmax, the missing realization law and the governed maximum key beneath it, and recognition above it. Advance the normalization row's evidence with the refinement it now has, and say which rung that supports — the row moves only if the rung's own definition is met, which is a judgement to make against the ladder rather than to assume.

## Closes when

No record outside `crates/**` states that `GovernedIndexAccess` emits exactly one region per occurrence, every family whose rung cites that limit cites its real blocker instead, and the normalization row records the refinement evidence with the rung it does or does not support.
