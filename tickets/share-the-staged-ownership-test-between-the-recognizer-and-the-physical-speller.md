---
id: share-the-staged-ownership-test-between-the-recognizer-and-the-physical-speller
title: Share the staged ownership test between the recognizer and the physical speller
status: todo
priority: p3
dependencies: []
related: [admit-a-scheduled-region-for-a-staged-elementary-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, maintenance]
---
## User-visible outcome

The predicate "this member set is a region of this staged occurrence" has one authority, so a widening of the staged shape (a three-stage law, or atoms spanning members) changes one site instead of silently diverging two.

## The duplication, found by the 2026-08-06 session audit

**Fact.** `spell_output`'s `Staged` arm in `crates/tiler-compiler/src/physical.rs` (near :873 at the audit's base) re-implements, verbatim, the private `NormalizedOutput::owns_region_members` `Staged` arm in `crates/tiler-compiler/src/request.rs` (near :1820): `!members.is_empty() && members.iter().all(|atom| atom.member() == normalized.member)`. The comment above the physical copy reads "the ownership test is the recognized partition's own", which reads as delegation but is a copy — `owns_region_members` is a private `fn` unreachable from `physical.rs`, and nothing asserts the two agree. This is the second-account-of-one-fact drift AGENTS.md warns about, at exactly the predicate the next staged widening must change in both places.

## The work

Either make `owns_region_members` `pub(crate)` and have `spell_output` call it (the comment then becomes true as written), or add a test asserting the two predicates agree over the staged fixture's member sets including the refusing cases (empty set, straddling atoms). Prefer the first — one authority beats an agreement test — unless reading exposes a reason the physical layer must not depend on the recognizer's method. Verify the two sites' line positions on your base; the audit's are facts about e4ccc6d9.

## Closes when

One site owns the predicate (or an agreement test pins the pair with a watched-failing perturbation), and the physical arm's comment states what is actually true.
