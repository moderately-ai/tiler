---
schema: "tiler-doc/v1"
id: "tiler.research.documentation.information-architecture-audit"
kind: "research"
title: "Information architecture and provenance audit"
topics: ["documentation", "provenance", "navigation"]
catalog_group: "documentation-governance"
research_status: "complete"
disposition: "adopted"
implementation_status: "implemented"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.document-metadata"]
adopted_by: ["ADR-0054"]
ticket: "docs-status-reconciliation"
---

# Information architecture and provenance audit

## Question

Can a new reader move from project status to normative contracts, accepted
decisions, supporting research, executable evidence, and live work without
mistaking proposals or spikes for implemented product behavior?

## Method

The audit walked the repository from its root and documentation portals, parsed
every governed Markdown frontmatter block, resolved typed relationships, checked
experiment entrypoints and ticket references, and compared repeated schema
claims across central contracts. Independent passes examined provenance,
onboarding, ticket navigation, open-question disposition, and catalog shape.

## Findings

- The corpus needed explicit status, design-map, research, experiment, and work
  portals; directory layout alone did not provide progressive disclosure.
- Bidirectional stored metadata had already drifted. One authoritative edge per
  relation is both simpler and stricter.
- Experiment entrypoints mixed document-relative and repository-relative paths,
  and several reproducible records omitted their evidence class or verification
  date.
- Architecture, IR, scheduling, artifact, and runtime documents repeated some
  schemas without saying which document owned the fields.
- A flat open-question list mixed product choices, implementation contracts,
  bounded evidence gates, and deliberately deferred work.
- Topics are useful facets but do not by themselves produce a stable, balanced
  catalog; coarse catalog groups need an explicit controlled value.

## Result

The repository now treats stable typed metadata as a checked interface. Forward
relations are stored once, experiment entrypoints are repository-root paths,
central contracts state their ownership boundaries, and open work is separated
by decision type. Generated catalogs remain a convenience view over source
metadata rather than a second authority.

This was a repository-structure audit, not evidence that the proposed compiler
has been implemented or that unresolved platform compatibility gates are closed.

## Traceability

Adopted by [ADR 0054](../../decisions/0054-use-typed-documentation-metadata.md)
and reflected in the
[documentation metadata contract](../../document-metadata.md). Work history is
recorded by the `docs-status-reconciliation` ticket.
