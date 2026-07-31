---
id: repair-the-broken-proof-budget-anchor
title: Repair the broken proof-budget anchor in the fusion contract
status: done
priority: p3
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

The `proof budget` cross-reference in the fusion-and-scheduling contract resolves to a real heading instead of silently landing at the top of the optimizer contract.

## Why this slice exists

`docs/compiler/fusion-and-scheduling.md:35` links to `optimizer.md#refinement-is-exhaustive-finite-evidence-with-an-explicit-gap`. No such heading exists. The nearest candidates are `## Lowering capability resolution and index-region refinement` (line 160) and `### Refinement requires discharged index-domain evidence` (line 179). Reproduce with `grep -n '^#' docs/compiler/optimizer.md | grep -i refinement`.

Found while defining backend/device vocabulary under `define-backend-device-and-execution-context-vocabulary`, by a link resolver run over the whole corpus: it was the only broken local link among 2,495 checked, so this is a point defect rather than a class of drift.

## Implementation keys

- Decide whether the link should point at an existing heading or whether the optimizer contract is missing the section the citing sentence promises. That sentence attributes to the target the claim that a proof budget "leaves its subject's predicate open while the plan containing it stands", and the target must actually state it.
- Do not repoint the link at a heading that does not make that claim merely to resolve the anchor.

## Closes when

The link resolves and its target states what the citing sentence attributes to it.

## Outcome (2026-07-31)

**Fact — the promised content exists under an existing heading, so the repair is a repoint plus a correction of the citing clause, not a new section.** `docs/compiler/optimizer.md` has no heading matching the linked anchor (`grep -n '^#' docs/compiler/optimizer.md | grep -i refinement` returns lines 160 and 179 only). The claims the citing sentence needs live in `### Refinement requires discharged index-domain evidence` (line 179): the exhaustive access-verification budget (the sixteen-million-cell discharge budget), the `ResourceLimit` residual, and the disposition of an unproved predicate.

**Fact — the citing clause overstated the target and was corrected in the same change.** It read "leaving its subject's predicate open while the plan containing it stands"; the target states the opposite disposition — "the region is valid analysis state because nothing disproved the predicate, but no later stage may treat it as refined, insert an unattributed physical guard, or allow it into an executable frontier", and an `Unknown` discharge "refuses the occurrence atomically". The sentence now says the predicate stays open, the region stays valid analysis state, and the occurrence is refused rather than allowed into an executable frontier until the proof is discharged — which preserves the search-costs-an-alternative versus proof-costs-a-proof contrast the passage exists for.

**Fact — the sweep.** `grep -rn "refinement-is-exhaustive-finite-evidence" docs/` returns nothing after the edit; the anchor `#refinement-requires-discharged-index-domain-evidence` matches the line-179 heading under the corpus's GitHub slug convention.
