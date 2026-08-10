---
id: repair-the-ticket-audit-report-citation-and-link-breakage
title: Repair the ticket-audit report citation and link breakage
status: done
priority: p0
dependencies: []
related: [close-the-ticket-audit-report-unclosed-code-span-blind-spots]
scopes: [research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gate]
---
## User-visible outcome

The checked-in 2026-08-10 ticket-audit reports remain readable evidence without making the repository citation gate fail on placeholder links, historical external-source spellings, or report-local shorthand.

## Per-Fact audit — 2026-08-10 at `b3a9bfadc0ca4126fafa599b90ade872d1082d5c`

- **Verified.** `make citations` fails on this exact base with 16 unresolved source citations and 27 unresolved Markdown links. Every reported failure is under `docs/research/documentation/ticket-audit-2026-08-10/reports/`; the command still resolves 1,187 pinned citations and 6,477 local links before returning exit 2.
- **Verified.** The audit [`README.md`](../docs/research/documentation/ticket-audit-2026-08-10/README.md) calls `ledger.json` / `ledger.jsonl` the living audit state and the per-ticket files content-addressed semantic reports. The reports are retained evidence, not a disposable generated directory.
- **Verified.** `check-citations.sh` deliberately checks a document with no status facet rather than skipping it. Its source anchor `A document with no status facet at all is checked rather than skipped` applies to these plain report files; classifying the whole report corpus as superseded merely to silence the gate would contradict that population rule.
- **Verified.** Representative failures are report prose being parsed as repository promises: literal placeholders such as `[...](...)`, ticket-relative links copied into a deeply nested report directory, and external citations copied without the version or SDK root that the live source now carries. The report meaning does not require any of those forms to remain clickable Markdown or live tree citations.
- **Inference.** The correctness-dominant repair is local to the reports: use code spans for illustrative or quoted Markdown syntax, make intended repository links root-correct, and qualify or quote historical external-source forms so the gate checks only claims that are meant to resolve. No citation-checker exclusion is justified by the audited population contract.

## Outcome

Integrated 2026-08-10. Every failing report-local link was made literal where it quotes historical Markdown, and every failing external citation was either qualified with the source revision or restated as a historical filename plus line range. `check-citations.sh` was not changed, no report was deleted or relabelled, and the audit findings remain unchanged.

The clean rerun resolved 1,171 pinned citations and 6,451 local Markdown links across 1,568 live ticket/comment/document files. It also printed 65 unclosed-code-span warnings. Those warnings do not fail this ticket's named gate but can hide later citations, so [`close-the-ticket-audit-report-unclosed-code-span-blind-spots`](close-the-ticket-audit-report-unclosed-code-span-blind-spots.md) owns that broader reachability repair.

## Closes when

- `make citations` exits 0 on the exact integrated tree.
- `tkt lint --format json` reports `ok: true`.
- `git diff --check` passes.
- Every changed report preserves the underlying audit finding; only link/citation syntax or provenance changes.
