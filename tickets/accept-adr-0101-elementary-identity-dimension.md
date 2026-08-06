---
id: accept-adr-0101-elementary-identity-dimension
title: Accept ADR 0101 elementary identity dimension
status: done
priority: p2
dependencies: []
related: [carry-the-elementary-identity-dimension-adr]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

[ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md) moves from `proposed` to `accepted`, or is rejected. **Only Tom closes this ticket**; its permanent status is `awaiting-decision`. The record names the fourth numerical-permission dimension (elementary-identity rewriting) as named-and-unpermissioned in ADR 0080's shape; acceptance catalogues the dimension and its refusal wording — it grants no permission (the grant is `decide-whether-to-admit-an-elementary-identity-permission`, deferred with its own trigger). The sweep flips `decision_status`, both catalog rows, and the source record's frontmatter-as-landed note. Filed by the coordinator at integration per the carrier convention.

## Decided — accepted

Accepted by Tom on 2026-08-06 at the live decision review in the coordination session, witnessed first-hand by the coordinator. The sweep executed in the same change: `decision_status` moved to `accepted` with the provenance in the record's own status line, both catalog rows flipped, the source record's disposition moved to `adopted` with the `ADR-0101` edge set and its status/traceability text corrected, the research catalog row updated, and the dimension's definition entered [Numerical semantics](../docs/numerical-semantics.md#elementary-function-identity-is-a-fourth-dimension) as the record's normative-destination line required — with the sketch-absence paragraph extended so the contract states the reserved-not-declined half. One reading recorded for the next steward: the registered softmax fact's clause "which-no-declared-dimension-names" reads against the *typed* permission vocabulary (`NumericalRealization`'s fields), which still lacks the dimension — ADR 0101 names it at the catalog level and admits no permission, so the fact stays true and moves only if [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) admits a typed dimension, as part of that step. The trigger evaluation for that deferral is logged on its own ticket in this change.
