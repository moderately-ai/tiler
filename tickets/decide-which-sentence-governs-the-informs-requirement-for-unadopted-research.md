---
id: decide-which-sentence-governs-the-informs-requirement-for-unadopted-research
title: Decide which sentence governs the informs requirement for unadopted research
status: in-progress
priority: p3
dependencies: []
related: [repair-the-four-mistyped-typed-frontmatter-edges]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, metadata, schema]
claimed_from: todo
assignee: agent-informs-rule
lease_expires_at: 1786057673
---
## User-visible outcome

`docs/document-metadata.md` states one rule for whether a research record must carry `informs`, instead of a required-field table and a prose sentence that disagree about unadopted records.

## The tension, surfaced by the edge repair

**Fact** (from [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md)'s outcome). The metadata contract's required-field table types `informs` as required on *every* research record, while the sentence below it binds only *adopted or partially adopted* research. Dropping the open-ticket audit's inadmissible portal edge left that record with no `informs` at all — admissible under the prose sentence, a violation under the table. The corpus now sits in the gap twice: `enforcer-input-property-exclusion` and `open-ticket-audit-2026-07-27`, both `pending`-disposition research with no admissible contract to inform.

**Why it is a decision rather than a fix.** Requiring `informs` on pending research forces either a premature contract edge (the drift the typed-edge check exists to catch) or a dummy target; binding only adopted research means a pending record's connection to the corpus rests on catalogs and body links alone. Both are coherent; the contract currently states both.

## What this must produce

One governing rule, stated once in the contract with the other site rewritten to agree, and the two in-gap records either left conforming (if the adopted-only reading wins) or given admissible edges (if the always-required reading wins). The typed-edge reproducing script in the repair ticket is the validator; re-run it and state the counts. A schema change to a contracts document is a contract edit — if the resolution is consequential beyond these two records, draft and park for Tom rather than self-deciding.

## Closes when

The contract states one rule, the two in-gap records conform to it, and the typed-edge check reports zero mistyped edges at the stated population.
