---
id: execute-the-adr-0102-acceptance-sweep
title: Execute the ADR 0102 acceptance sweep
status: todo
priority: p2
dependencies: []
related: [accept-adr-0102-conversion-pair-decomposition, land-the-conversion-pair-decomposition-adr]
scopes: [contracts/decisions, contracts/navigation, contracts/numerics, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [acceptance-sweep, adr, conversion]
---

## The acceptance this applies

**Tom accepted ADR 0102 on 2026-08-06 at the live session's decision round** (provenance on [`accept-adr-0102-conversion-pair-decomposition`](accept-adr-0102-conversion-pair-decomposition.md)). A decision recorded is not a decision applied; this ticket is the whole application, in one change, because an acceptance applied in halves is how a draft gets read as settled.

## The sweep, enumerated

1. `docs/decisions/0102-key-conversion-families-by-the-ordered-pair-and-derive-their-fields.md`: `decision_status: proposed` → `accepted`, with the acceptance sentence (who, date, venue) added per the corpus convention.
2. `docs/decisions/README.md`: both catalog rows' `— proposed` suffix → `— accepted` (theme row under Numerical operations, chronology row after 0101).
3. `docs/roadmap.md`: the Cast-and-convert row's trigger cell sentence ("sits at `proposed`, so the decision is Tom's and this row's rung moves on neither outcome") corrects to the accepted state — the rung still does not move (the ADR registers nothing), but the pending-decision framing is now false.
4. `docs/numerical-semantics.md`: the widening-and-narrowing section titled "derived at the BF16/binary32 pair" gains the accepted general rule's statement (or a pointer to the ADR as its owner), and the document's `evidence` frontmatter array gains the research record id if the convention requires it — read the sibling acceptances (ADR 0091's sweep) for the exact shape rather than inventing one.
5. The stale "`RQ-OP-04` leaves … open" clause in the minimum-correct-physical-realization profile record (locate under `docs/research/numerics/` by content): plainly wrong at acceptance per the carrier's own analysis — correct in tense.
6. Sweep for any other sentence whose truth depended on the proposed status (`grep -rn '0102' docs/ tickets/` and read each hit).

## Why it is held rather than dispatched

`contracts/navigation` is live-claimed by the navigation cell batch at filing time. TRIGGER: that claim releases — dispatch or execute coordinator-inline immediately after its merge.

## Closes when

All six items land in one change, every 0102 mention agrees with the accepted status, and `tkt lint` passes.
