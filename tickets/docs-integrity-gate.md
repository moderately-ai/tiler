---
id: docs-integrity-gate
title: Add strict documentation integrity gate
status: done
priority: p1
dependencies: [docs-status-reconciliation]
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets, research/indexing]
paths: [.gitignore]
tags: [docs-ia]
---
Implement the compact dependency-free metadata, graph, link, generated-index, ticket-outcome, and open-question validator with fixture tests and CI.

## Outcome

- Added one dependency-free validator for strict frontmatter, kind schemas,
  typed graph edges, cycles, local links, experiment entrypoints, ticket
  outcomes, stable open-question records, and generated catalog freshness.
- Added deterministic thematic ADR, research, and experiment catalog rendering.
- Added isolated parser/graph fixtures and a minimal GitHub Actions gate.
- Documented the local validation and regeneration commands in the normative
  metadata contract.
