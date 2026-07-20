---
schema: "tiler-doc/v1"
id: "ADR-0054"
kind: "decision"
title: "Use typed documentation metadata and derived backlinks"
topics: ["documentation", "governance", "traceability"]
catalog_group: "documentation-governance"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.document-metadata"]
evidence: ["tiler.research.documentation.information-architecture-audit"]
ticket: "docs-status-reconciliation"
---

# 0054: Use typed documentation metadata and derived backlinks

**Status:** accepted

## Context

Tiler's design corpus spans contracts, ADRs, research, executable spikes, and
live tickets. Prose links alone did not distinguish authority from evidence or
make stale relationships detectable. Storing both directions of a relationship
also produced drift during the first metadata migration.

## Decision

Governed Markdown uses the strict `tiler-doc/v1` metadata contract. Stable IDs
identify records independently of paths, relationships are typed, and each
relationship has one stored direction. Catalogs and backlinks are derived and
validated from those authoritative edges.

The repository checks metadata, relationship targets, entrypoints, ticket
references, and deterministic generated catalog sections. Ticketsplease remains
the authority for live workflow state; document metadata records durable design
and evidence relationships only.

## Consequences

- A blank reader can distinguish normative contracts, accepted choices,
  evidence, experiments, and planned work before reading deeply.
- Renames do not change graph identity, while broken paths and IDs fail checks.
- Authors maintain one relationship edge instead of synchronized backlinks.
- Adding a governed document carries a small, explicit metadata obligation.

## Alternatives considered

Free-form prose links cannot support strict integrity checks. Storing both
directions makes local browsing convenient but duplicates authority and had
already diverged. A separate database would make Git history and ordinary
GitHub reading worse.

## Traceability

The [information-architecture audit](../research/documentation/information-architecture-audit.md)
records the observed corpus failures and migration used to validate the
[metadata contract](../document-metadata.md).
