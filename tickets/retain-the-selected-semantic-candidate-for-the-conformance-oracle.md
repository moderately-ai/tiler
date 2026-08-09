---
id: retain-the-selected-semantic-candidate-for-the-conformance-oracle
title: Retain the selected semantic candidate for the conformance oracle
status: awaiting-decision
priority: p2
dependencies: []
related: [decide-how-a-pinned-pointwise-grouping-becomes-evaluable, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation, accept-the-composed-realization-evaluation-surface]
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, reference, decision, needs-tom, public-boundary]
---
## User-visible outcome

The semantic candidate the portfolio selected is retained where a conformance oracle can evaluate it, so an expected value can be computed from the program the plan actually implements instead of from the caller's baseline or from the artifact under test.

## Why this exists

**Fact.** [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) closed on design 1 — retain the selected semantic candidate's program — and implemented nothing; its Outcome says so. **Fact.** [The composed-realization-evaluation derivation](../docs/research/reference/composed-realization-evaluation.md) rests on that retention: its refusal 1, `CandidateProgramNotRetained`, is currently the answer for its entire population, because `grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` returns nothing at 5cec07d0 — no accessor exposes any semantic program from a compilation. Until the retention lands, the derived composition oracle is a derivation without a mechanism.

## In scope, from the records that derived it

- Retain the selected candidate `P'` per retained plan alternative, exposed where the settled design's Outcome puts it (read that Outcome; the exact siting is its decision, not this ticket's). If the accessor is a new public surface, it lands as a labelled draft with an acceptance node for Tom under ADR 0075.
- **The existing composition site's provenance repair rides with the retention, because the retention is what makes it expressible.** `the_assembled_split_program_matches_the_partitioned_sum_oracle` (`crates/tiler-compiler/src/pipeline/tests.rs`, near :6501 at 5cec07d0) feeds `strict_partitioned_sum` the tensor its first kernel produced — the artifact under test — which is design 2 of the settled fork at the composition boundary. It is sound at that base only because its fixture's prologue (`2.0 * x + 1.0`) carries no reassociation site, so `P' = P`. Once `P'` is retained, the oracle's input becomes a reference evaluation of `P'`, which is the change of provenance the derivation's Part 6 specifies.

## Non-goals

The driver and the `ValueId`-keyed reference primitive — those are [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md)'s items A and B, parked for Tom; re-deciding the settled fork.

## Decision packet — 2026-08-09

The completed design ticket selected retention but explicitly left its public boundary to Tom; it did not settle an exact accessor or owner. This node was therefore not implementation-ready.

- **Option A — retain `P'` on each public plan alternative and expose a read-only semantic-program accessor (recommended).** This follows the selected design directly and makes the oracle's subject explicit, but adds a labelled public compiler surface that must name its lifetime and exclusion from artifact identity.
- **Option B — retain `P'` in an internal conformance side table keyed by the selected alternative.** This avoids a public accessor initially, but creates a second ownership/indexing relation and cannot support external conformance consumers without a later boundary and migration.

Tom needs to accept the exact siting. The design-1 choice itself is not reopened; this decision is only where the retained candidate lives and who may read it.

## Closes when

The selected candidate is retained and reachable per the settled design, the composition-site test computes its expected prologue value from `P'` rather than from the first kernel's output, both are covered by tests that fail under the reverted provenance, and any public accessor is a labelled draft with its acceptance node filed.
