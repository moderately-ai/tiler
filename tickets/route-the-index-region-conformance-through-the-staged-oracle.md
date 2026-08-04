---
id: route-the-index-region-conformance-through-the-staged-oracle
title: Route the index-region conformance through the staged oracle
status: todo
priority: p2
dependencies: [reclassify-language-model-work-as-a-conformance-track]
related: [decide-the-index-region-oracle-route-past-its-step-budget, route-the-contraction-conformance-through-the-staged-oracle]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, contraction, language-model]
---
## User-visible outcome

`crates/tiler-compiler/src/governed/contraction_conformance.rs` states the index-region oracle's reach as a settled decision it cites, and either compares the *emitted* region at `w_vocab_slice` or says why that crate declines the cost — so "the emitted region reproduces a measured device result" stops resting on the one cell that happened to fit.

## Why this is now open

**Fact — `decide-the-index-region-oracle-route-past-its-step-budget` (2026-08-01).** `MAX_EVALUATION_STEPS` did not move. What changed is that `tiler_reference::IndexRegionEvaluator::stage` now walks a verified region in caller-sized spans of its parallel domain, under an unchanged per-span budget, with the independence argument in `StagedIndexRegionEvaluation`'s rustdoc. A hand-mirrored `w_vocab_slice` region reproduces the retained `direct` digest through that walk.

So this file's section "The index-region oracle reaches the smaller of the two" is stale in one respect. Its statements remain true — the budget still refuses one span over that region, and `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` still passes unchanged — but it presents the boundary as an open question owned elsewhere, and it is now a decided one.

## What this must decide

1. Whether `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` keeps asserting the whole-region refusal (it should: it is the standing watcher for the budget, and costs 8.3 s) and simply cites the decision instead of naming an owner.
2. Whether the emitted region is *also* compared at `w_vocab_slice` through the staged walk. **Measurement — Apple M4 Max, 2026-08-01, dev profile:** that walk costs ~55 s at 516 ns a step. An `#[ignore]`d test on the precedent of `tiler-reference`'s own profile-cell tests is the obvious shape; declining the cost outright is also a legitimate answer if stated.
3. Whether the file's "four cells nothing here compares" paragraph — already flagged stale by `route-the-contraction-conformance-through-the-staged-oracle` — is settled by that sibling ticket or by this one. Do not settle it twice.

## Closes when

The file's reach statements cite accepted decisions rather than open ownership, and every cell it does not compare has a stated reason with its measured cost.
