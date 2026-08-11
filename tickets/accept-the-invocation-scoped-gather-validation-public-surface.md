---
id: accept-the-invocation-scoped-gather-validation-public-surface
title: Accept the invocation-scoped gather validation public surface
status: todo
priority: p1
dependencies: [admit-an-invocation-scoped-gather-index-validation-receipt]
related: [accept-adr-0108-data-dependent-index-coordinate-siting, emit-the-indirect-gather-on-metal]
scopes: [contracts/decisions, contracts/artifacts, contracts/integrations, contracts/foundation, implementation/runtime, implementation/frontend]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, runtime, gather]
---
## User-visible outcome

Tom accepts or revises the exact public authoring, validation, refusal, inspection, and consumption surface of the narrow host-visible gather receipt after it exists and is tested.

## Review boundary

Present every public type, constructor, field, method, error, and call-site change; the exact included host-visible immutable-snapshot lane; all excluded mutable, device-resident, callback, assertion, fallback, and general-indirect cases; and the identity/version consequences. Nothing in the implementation ticket self-accepts this surface under ADR 0075.

## Closes when

The exact included and excluded surface, compatibility stance, acceptance provenance, and any required visibility narrowing are recorded and the downstream Metal ticket names the accepted boundary rather than the draft.
