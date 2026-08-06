---
id: implement-stage-level-cover-atoms-for-multi-region-occurrences
title: Implement stage-level cover atoms for multi-region occurrences
status: todo
priority: p1
dependencies: []
related: [resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage, admit-the-registered-elementary-families-as-recognizable-program-stages, widen-the-staged-realization-law-to-the-registered-elementary-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, cover, identity-domain, p1-spine]
---

## The decision this executes

**Tom decided on 2026-08-06, at the live session, relayed and executed by the coordinator:** the planner's attribution atom becomes a *(member, stage)* pair — Option A of [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md), whose derivation is the authority for what follows. The grounds, in the priority order Tom stated (correctness, performance, long-term maintainability, code quality): correctness is equal between the options; stage-level atoms are what let a family's internal pass fuse into a neighbouring region (the flash-shaped plan the project exists to reach); one identity migration instead of a guaranteed two; and stages are real domain objects the model should say exist. Option B (a multi-stage region spelling) is *rejected*, not deferred — its internal-boundary opacity would have to be undone by exactly this change later.

## The surface (from the fork ticket's derivation, verified at its cited sites)

`SemanticMemberId` (`crates/tiler-compiler/src/region.rs:123`, `pub(crate)`) is the attribution key throughout: `NormalizedOutput::owns_region_members` (`request.rs`), `physical::spell_output` (first-match by `members ==`), `cover::derive_duplication` (repeated member = duplication, never a split), the region graph, program assembly's stage coverage, and the identity encodings — cover identity, region-occurrence identity, and explain records all encode member positions. The change threads a stage ordinal beside the member everywhere attribution is decided, with single-stage occurrences carrying stage zero so every existing program's semantics are unchanged.

## The identity step, executed whole

Cover identity, region-occurrence identity, and any explain record encoding member positions move together: the encoding change lands with its version step at the owning layer (or per-tag injectivity reasoning if genuinely appends-only — derive which honestly, do not assume appends-only because it is cheaper), the ledger comments move in the same commit, and every pinned identity is recomputed on the tree from observed failing values with each moved pin enumerated in the report. The explain request qualifier (currently `ce6f9106c1c5933b`) will move if the subject bytes reach any changed encoding — verify rather than assume in either direction.

## What this ticket does and does not deliver

It delivers the attribution model: multi-region occurrences become representable, `derive_duplication` distinguishes a split from a duplication, and `spell_output` resolves per stage. It does NOT register any elementary family's law (that is [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md), which supplies the producers of multi-region realizations) — the two compose in the parent keystone. A test proving the new atom is load-bearing must exist without the law widening: the split-reduction's existing partial/final pair, or a hand-built two-stage subject, exercised so the stage ordinal is observed distinguishing what member sets could not.

## Closes when

The atom is `(member, stage)` at every attribution site, single-stage behaviour is byte-identical (existing pins unmoved or recomputed with ledgers per the step), the distinguishing test exists and was watched failing under the old key shape, and the parent keystone's wall 2 derivation is updated to point here as discharged.
