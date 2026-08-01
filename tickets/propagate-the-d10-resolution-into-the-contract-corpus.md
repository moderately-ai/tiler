---
id: propagate-the-d10-resolution-into-the-contract-corpus
title: Propagate the D-10 resolution into the contract corpus
status: in-progress
priority: p2
dependencies: [admit-the-reindex-and-broadcast-operation-families]
related: [design-attention-program-vertical, scope-the-sequence-extending-tensor-family, compose-rotary-position-embedding-from-reindex-and-broadcast, own-operation-family-support-matrix]
scopes: [contracts/foundation, research/program-planning, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, semantics, structural]
claimed_from: todo
assignee: worker-propagate-th
lease_expires_at: 1785564765
---
## User-visible outcome

A reader who opens [IR](../docs/ir.md) to learn what a `Reindex` admits gets the same answer the registered definition gives, instead of a sentence that predates the decision.

## Why this is a separate ticket

**Fact — the resolution landed outside these files' scopes.** [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) settled decision D-10 in `tiler::reindex-f32@1`'s registered `NormativeDefinitionRef`, which is `implementation/ir`. The three documents that still state the question, or the pre-decision reading, are `contracts/foundation` and `research/*`. A ticket does not edit outside its declared scopes, and this remainder is narrow enough to carry its own.

**Inference — nothing checks the disagreement.** Nothing validates the documentation corpus, so a `Reindex` sentence that omits an admitted form costs a reader rather than a gate, and the cost is exactly the kind [AGENTS.md](../AGENTS.md) names: a doc comment is a claim, and an understated one makes reachable work look unreachable.

## Evidence prerequisite

**Fact — the resolution.** A within-axis coordinate permutation is admitted in exactly one named form, `reverse-axis`, the map `i -> extent − 1 − i`; no other is admitted, and one presented under any other name is refused as `reindex.form.unadmitted-kind`. The derivation and its four steps are recorded in the admission ticket's outcome and, in short form, in the registered normative reference itself.

**Fact — the three stale sites.**

- [`docs/ir.md`](../docs/ir.md) spells the initial forms as "bijective permutations/split/merge mappings or legal removal/insertion of unit axes". That sentence is now incomplete, and the same section's `Reindex` paragraph is where a reader looks first.
- [The L4 attention design](../docs/research/program-planning/first-attention-program-vertical.md) carries D-10 in its unresolved-decisions list and states it as open at two further points.
- [The sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) carries the qualification that its "no slice and no concatenate inside a layer" result is conditional on D-10.

## Required delivery

- **`docs/ir.md`'s `Reindex` paragraph states the complete admitted set**, including the reversal, and states that a general within-axis permutation is a tensor-data-derived index the index-expression vocabulary rejects — which is the reason the admission is one named form rather than a class.
- **L4's D-10 entry moves out of the unresolved list** and records the answer and where it lives, rather than being deleted: an open question is removed only when its answer lives in a durable contract, and the derivation is worth keeping beside the measurement that motivated it.
- **The sequence-extending record's qualification is discharged**, so its result reads as unconditional with the resolution cited rather than as conditional on a decision that has been made.
- **No new claim.** This ticket propagates a decision already taken; it does not widen the family, revisit the rotation question, or restate the derivation as though it were being made here.

## Non-goals

Reopening D-10, admitting a within-axis rotation, and any change to `crates/`.

## Closes when

All three documents agree with the registered normative reference, and `grep -rn 'D-10' docs/` returns only settled statements.
