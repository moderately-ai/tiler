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

**Dated correction — 2026-08-10, implementation standing only.** The two paragraphs above are retained verbatim because they record the accepted metadata model and the enforcement claimed when it was accepted; this correction retires the present-tense implementation promises without changing the decision. Strict metadata, stable IDs, typed relationships, and one stored direction per relationship remain the authority rule. Catalog and backlink prose must restate those authoritative edges rather than become a second stored direction, but the checked-in catalogs are maintained by hand and `derived` no longer asserts that a standing generator or validator exists. The surviving gate is narrower: `make citations` runs `check-citations.sh` over non-terminal tickets and their comments, documents not marked `superseded`, and repository-root Markdown documents. Within the script's documented syntax and provenance exclusions, it checks locally resolvable pinned source citations and local path existence for supported Markdown links. A green result says only that a checked citation or link points somewhere; it does not establish the surrounding claim or the target's meaning. No standing check validates frontmatter or schema, stable-ID uniqueness, typed relationship targets, document `ticket` or experiment `entrypoints` references, supersession correctness, heading fragments, quotation fidelity, catalog or backlink derivation, or whether an entrypoint or catalog lists the right records. Historical hand-run ticket scripts measured some catalog and typed-edge properties at named commits, but they are not repository gates. [`reconcile-adr-0054-metadata-check-promises-with-the-surviving-documentation-gate`](../../tickets/reconcile-adr-0054-metadata-check-promises-with-the-surviving-documentation-gate.md) records the source audit and exact checker boundary.

## Consequences

- A blank reader can distinguish normative contracts, accepted choices,
  evidence, experiments, and planned work before reading deeply.
- Renames do not change graph identity, while broken paths and IDs fail checks.
- Authors maintain one relationship edge instead of synchronized backlinks.
- Adding a governed document carries a small, explicit metadata obligation.

**Dated consequence correction — 2026-08-10.** The rename clause remains the model: preserving a record's stable `id` preserves its graph identity. The combined present-tense guarantee `broken paths and IDs fail checks` is retired. Only paths reached by the bounded citation/link gate above fail mechanically when they do not resolve; broken document IDs and the other unchecked metadata properties named there do not.

## Alternatives considered

Free-form prose links cannot support strict integrity checks. Storing both
directions makes local browsing convenient but duplicates authority and had
already diverged. A separate database would make Git history and ordinary
GitHub reading worse.

## Traceability

The [information-architecture audit](../research/documentation/information-architecture-audit.md)
records the observed corpus failures and migration used to validate the
[metadata contract](../document-metadata.md).
