---
id: close-the-ticket-audit-report-unclosed-code-span-blind-spots
title: Close the ticket-audit report unclosed-code-span blind spots
status: in-progress
priority: p1
dependencies: []
related: [repair-the-ticket-audit-report-citation-and-link-breakage]
scopes: [research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, verification]
claimed_from: todo
assignee: terra-code-span-audit
lease_expires_at: 1786388862
---
## User-visible outcome

The citation gate parses every checked-in 2026-08-10 audit report and repair specification to the end without an unclosed inline-code span hiding later citations or links.

## Per-Fact audit — 2026-08-10 at the post-failure repair tree

- **Verified.** After [`repair-the-ticket-audit-report-citation-and-link-breakage`](repair-the-ticket-audit-report-citation-and-link-breakage.md) makes `make citations` exit 0, the same command reports `parse warn 65 file(s) ended inside an unclosed code span, so citations in them may have been missed`.
- **Verified.** The warned population is 63 content-addressed files under `docs/research/documentation/ticket-audit-2026-08-10/reports/` and two accepted repair specifications under the sibling `repairs/` directory.
- **Verified.** The warning is evidence of incomplete reachability, not cosmetic formatting: `check-citations.sh` says citations after an unclosed code-span opener may have been missed. A green link/citation count therefore cannot prove those suffixes were inspected.
- **Inference.** Repairing the report Markdown is preferable to suppressing the warning or excluding the corpus because the audit [`README.md`](../docs/research/documentation/ticket-audit-2026-08-10/README.md) defines these files as retained semantic reports and accepted repair specifications.

## Outcome

Balance or replace the malformed inline-code delimiters while preserving every report's literal source anchors and verdicts. Re-run the gate after deliberately planting one resolvable and one dangling link after a formerly unclosed span to prove the repaired suffix is reached.

## Closes when

- `make citations` exits 0 and reports `parse warn 0 file(s)` or omits the warning section because the count is zero.
- A subject perturbation places a dangling link after a repaired delimiter, and the gate fails with that exact path before the subject is restored.
- `tkt lint --format json` reports `ok: true` and `git diff --check` passes.
