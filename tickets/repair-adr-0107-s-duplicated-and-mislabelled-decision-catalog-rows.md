---
id: repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows
title: Repair ADR 0107's duplicated and mislabelled decision catalog rows
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## ADR 0107 appears twice, wrong once, and is missing from the chronology

Four defects in `docs/decisions/README.md`, all about one record, from the 2026-08-07 read-only audit:

1. **A row under "Foundation, semantics, and extensions" carries ADR 0077's metadata** — `contracts: [System architecture]; evidence: [Prototype crate layout and Rust MSRV]` — copied from the adjacent row. ADR 0107 declares `applies_to: ["tiler.contract.ir"]` and `catalog_group: "physical-planning-lowering"`, so it is in the wrong group as well as carrying the wrong facts.
2. **A second row for 0107 exists** further down with correct metadata. Two contradictory rows for one record.
3. **Both read `— proposed`.** ADR 0107's frontmatter is `decision_status: "accepted"` — Tom accepted it on 2026-08-07 and the record carries the acceptance provenance.
4. **The chronology omits it entirely**: 107 ADR files on disk, 106 rows, the block ending at `0106`.

## Why this was not caught, and what it means for the fix

The block was emitted by `scripts/docs.py`, deleted at `e197176f`. Its `BEGIN GENERATED` markers were removed on 2026-08-07 and the catalogs restated as **hand-maintained**, which is now the true and recorded obligation — so these rows must be repaired by hand and will stay correct only by reading.

**Verify each of the four at your base before editing**, including the file counts, and report per-defect. The audit that found them is a claim; two of its sibling findings needed correcting on nuances the auditor missed.

## Scope note

Adding the missing chronology entry means the block's population changes. **Report the count before and after** so a reader can tell a repair from a drift — this repository has repeatedly found counts that were correct when written and went stale, and the distinction between "was wrong" and "went stale" is worth stating.

## Closes when

ADR 0107 has exactly one row, in its declared `catalog_group`, carrying its own `applies_to` and evidence, reading `accepted`; the chronology's population equals the ADR file count; and the counts are stated before and after.
