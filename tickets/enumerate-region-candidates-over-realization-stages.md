---
id: enumerate-region-candidates-over-realization-stages
title: Enumerate region candidates over realization stages
status: in-progress
priority: p1
dependencies: []
related: [resolve-which-authority-mints-a-multi-stage-region-candidate, fold-the-attribution-stage-into-region-and-request-subject-identity, admit-the-registered-elementary-families-as-recognizable-program-stages, implement-stage-level-cover-atoms-for-multi-region-occurrences]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, identity-domain]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1786044344
---
## User-visible outcome

A program containing a staged family enumerates one region candidate per realization stage, so the cover search sees the family's internal boundary and a registered elementary middle stage becomes coverable — the outcome [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) is blocked on. This executes Tom's Option A′ decision on [`resolve-which-authority-mints-a-multi-stage-region-candidate`](resolve-which-authority-mints-a-multi-stage-region-candidate.md), whose derivation is the specification and is not re-litigated here.

## The surface, enumerated from the decision node

- **`region::assemble` (`crates/tiler-compiler/src/region.rs`)** takes the registered `IndexRealizationLaw` authority (and therefore the scalar authority) as an input and mints one candidate per stage for an occurrence whose law realizes a region sequence, with intra-occurrence producer/consumer edges. Every other occurrence's enumeration stays first-stage.
- **Synthetic values.** A staged occurrence's published intermediate is no program value; the region graph, `RetainedOutput`, `MaterializationEdge::value` (`cover.rs`), program assembly's internal values, and the assembled program's ABI must carry it explicitly. Fail closed anywhere the value cannot yet be represented.
- **The cover obligations.** `verify_cover`'s per-operation counting becomes per-stage counting (every stage covered exactly once — the mask obligation `cover::member_index`'s doc names); `Partitioner::refused_duplication` and the completeness test move with it, each watched failing.
- **Identity, whole in this change.** [`fold-the-attribution-stage-into-region-and-request-subject-identity`](fold-the-attribution-stage-into-region-and-request-subject-identity.md) fires with this ticket and must land in the same change: region content, region occurrence, and request-subject identity encode the stage atom injectively with the reasoning at the encoding sites, the `unencoded-member-stage` premise guard is replaced by the encoding rather than widened around, and every single-stage program's bytes are proved unchanged or every moved pin is recomputed on the landing tree and enumerated in the commit.

## Non-goals

The softmax's law (still wants the maximum key and the multi-reader sequence vocabulary); any recognizer widening beyond what stage-enumerated candidates make reachable; reference bit-agreement for a compiled middle-stage program if a wall outside these scopes appears — name it with an owner instead.

## Closes when

A program with a normalization middle stage enumerates staged candidates, the cover search covers every stage exactly once, the identity encoding lands whole with its blast radius enumerated, and the recognizer parent's blocked outcome is re-evaluated against what remains.
