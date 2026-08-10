---
id: refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause
title: Refresh the roadmap softmax cell's remaining-prerequisite clause
status: done
priority: p2
dependencies: []
related: [refresh-the-l2-derivation-operation-family-standing, register-the-softmax-realization-law]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

The roadmap's `tiler::softmax-f32@1` family row states the family's actual remaining walls, so a reader of the family-state table reaches the current standing rather than three landed prerequisites.

## The finding

**Fact.** The L2 family-standing refresh found the roadmap's softmax cell naming two remaining prerequisites — a governed maximum scalar key and a multi-reader handed value — whose tickets were already `done`, and flagged the stale clause as outside its scopes. Between that read and this ticket the third wall also fell: `register-the-softmax-realization-law` registered `StagedSoftmaxF32` (tag 11) as a labelled draft, and the measured boundary refusal moved from `operation-set` to `missing-capability` — not to the `region-staged-family-unspellable` wall the cell predicted.

## Closes when

The cell's ceiling narrative and its trailing R6-needs clause state the three landings, the corrected wall (with the wrong prediction recorded rather than dropped), and what R6 still needs; the adjacent extrema-family row notes the scalar key without moving its rung.

## Outcome — 2026-08-06

Three edits to `docs/roadmap.md`, each read in the full cell before writing:

- The softmax row's "the reason has narrowed twice" Fact is now "narrowed three times": the law registration (tag 11, labelled draft, acceptance node parked), the staged arm answering `true`, and the measured `missing-capability` refusal with the class-and-rule assertion rationale.
- The trailing R6-needs clause carries a dated full supersession: all three walls down (maximum key accepted, multi-reader retention accepted with the region-formation carry noted, law registered), the wrong `region-staged-family-unspellable` prediction recorded with the measured wall in its place, and the two things R6 still needs (a shipped four-stage lowering provider; a physical staged-plan arm).
- The `Minimum`/`Maximum` families row gains a dated Fact that `tiler.scalar::maximum-f32@1` exists as scalar realization vocabulary and is deliberately not a semantic family, naming `the_maximum_has_no_semantic_counterpart` as the check, so the row's R2 stands with the coincidence explained rather than discoverable.

The rung cell stays R5: no program compiles through the key, which is the R6 line the cell already draws.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The Outcome's "labelled draft" / `accept-the-softmax-realization-law` parked language is historical as of the 2026-08-06 close. That acceptance node is `status: done` with **Accepted — 2026-08-07** (Tom, no exclusion). The live `docs/roadmap.md` Softmax cell and `crates/tiler-ir/src/index/law.rs` Draft-boundary comments still speak present-tense of a labelled draft awaiting decision; that drift is residual product debt on those paths, not a reopen of this navigation ticket. Separately, the roadmap R6-needs clause still says the four-stage lowering provider and the physical `staged_plan` arm are each "its own ticket," but at audit time no discoverable owner ticket existed under those subjects — the phrase asserted tickets without filing. Filing (or connecting) those two R6 walls and flipping the live matrix draft/parked prose remain remainders outside this node's delivered close condition.
