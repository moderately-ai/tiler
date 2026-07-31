---
id: accept-the-public-backend-provider-composition-boundary
title: Accept or revise the public backend-provider composition boundary
status: todo
priority: p1
dependencies: [draft-the-backend-provider-composition-adr]
related: [draft-public-extension-seam-ownership-adr]
scopes: [contracts/decisions, contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, decision, needs-tom]
---
## User-visible outcome

Tom receives one evidence-backed atomic decision packet for the backend-provider composition boundary, and no production or public implementation conditional on that model becomes dispatchable before acceptance.

## Decision boundary

Present the exact proposed ADR after eliminating every option that cannot preserve target-independent semantics, re-verification, deterministic identity, partial provider composition, AOT build/runtime separation, routing safety, and long-term multi-backend evolution. State what each surviving option enables and prevents, its counterpoint, and the recommendation.

This node is not research or implementation work. It remains parked until the proposed record exists and Tom accepts it or requests revisions.

## Closes when

Tom accepts or revises the ADR; its status, acceptance date, body, catalogs, governed contracts, and implementation boundary agree; proposal-only disclosures are removed or corrected; and all dependent implementation tickets are released by this node becoming `done`.

## Graph maintenance

- Only Tom approves or revises the decision. After his answer, the implementing agent records it durably, applies every acceptance consequence, runs the checks, and closes this node.
- If the ADR is revised, amend the still-proposed record rather than creating a superseding accepted fiction.
- Keep multi-device, dynamic-library plugins, untrusted providers, and stable plugin ABI outside the accepted initial boundary unless the evidence unexpectedly forces one of them into scope.
