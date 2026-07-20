---
id: docs-navigation-metadata
title: Define documentation metadata and navigation
status: in-progress
priority: p0
dependencies: []
related: []
scopes: [contracts/navigation, contracts/core]
shared_scopes: [contracts/decisions, project/tickets]
paths: []
tags: [docs-ia]
claimed_from: todo
assignee: codex
lease_expires_at: 1784559616
---
Establish the document metadata schema, progressive-disclosure portals, generated catalog structure, parked workflow states, and validator interface.

## Outcome

- Defined the constrained `tiler-doc/v1` metadata and typed relationship
  contract in `docs/document-metadata.md`.
- Added root, documentation, status, design-map, research, experiment, and work
  tracking portals with generated-catalog boundaries.
- Added `awaiting-decision` and `deferred` parked states and moved existing work
  to truthful workflow categories.
- Split documentation scopes so the three provenance migrations can run in
  parallel without claiming the same files.
- Created the dependency-ordered `docs-ia` initiative tickets. Strict
  validation and catalog rendering are delivered by `docs-integrity-gate`
  after metadata backfill.
