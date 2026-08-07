---
id: carry-the-l6-bounded-measurement-class-into-the-research-catalog
title: Carry the L6 record's bounded-measurement class into the research catalog
status: in-progress
priority: p3
dependencies: []
related: [audit-the-ingestion-records-no-measurements-header-claim]
scopes: [contracts/navigation]
shared_scopes: []
paths: []
tags: [documentation]
claimed_from: todo
assignee: agent-orch-docs
lease_expires_at: 1786076377
---
## User-visible outcome

`docs/research/README.md`'s entry for the L6 ingestion and execution record names the same evidence classes its frontmatter does, so the catalog and the record agree.

## The finding

**Fact.** [`audit-the-ingestion-records-no-measurements-header-claim`](audit-the-ingestion-records-no-measurements-header-claim.md) corrected `docs/research/program-planning/complete-model-ingestion-and-execution.md`'s header, which claimed "This record contains no measurements and takes none" while the body relays six dated **Measurement** paragraphs, and widened its frontmatter to `evidence_classes: ["primary-source-synthesis", "bounded-measurement"]`. That audit held `research/program-planning` and could not touch `docs/research/README.md`, which is `contracts/navigation`.

**Fact.** `docs/research/README.md` still reads `pending; primary-source-synthesis` for that record. Sibling entries already carry the compound form — line 22 reads `primary-source-synthesis, executable-model, bounded-measurement` — so the catalog's own convention is to mirror the frontmatter list.

## The work

Restate the L6 entry's evidence classes as `primary-source-synthesis, bounded-measurement`, matching the record's frontmatter. Its two `experiments:` links (the identity-growth spike and the Qwen3 C1 conformance fixture) are already correct and are the sources of four of the six relayed measurements; nothing else on the line moves.

While there, check whether any other catalog entry's evidence classes have drifted from its record's frontmatter, and report the count rather than assuming this is the only one.

## Closes when

The catalog line and the record's frontmatter state the same evidence classes.
