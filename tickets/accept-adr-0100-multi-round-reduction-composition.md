---
id: accept-adr-0100-multi-round-reduction-composition
title: Accept ADR 0100 multi-round reduction composition
status: done
priority: p2
dependencies: []
related: [derive-the-multi-round-two-level-reduction-composition]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

[ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) moves from `proposed` to `accepted`, or is rejected. **Only Tom closes this ticket**; its permanent status is `awaiting-decision`. Acceptance would supersede ADR 0096 decision 8 only, and the acceptance sweep executes that supersession explicitly plus both catalog views. The record self-accepts nothing; its five public-boundary items are enumerated in the derivation record for separate decisions. Filed by the coordinator at integration per the carrier convention — the drafting ticket's outcome (a proposed record) is complete and closed.

## Decided — accepted

Accepted by Tom on 2026-08-05 at the third live decision review in the coordination session, witnessed first-hand by the coordinator. Sweep executed in the same change: `decision_status` flipped, both catalog views gained the 0100 rows, the research record's catalog row landed (executing `catalogue-adr-0100-and-the-multi-round-composition-record` in the same change), and ADR 0096's decision-8 correction paragraph records the supersession — decision 8 alone, in prose on both records, because a whole-record frontmatter edge would overstate it.
