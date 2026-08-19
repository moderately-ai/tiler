---
id: re-anchor-the-schedule-builder-line-citations
title: Re-anchor the schedule-builder line citations
status: todo
priority: p2
dependencies: []
related: [split-the-schedule-builder-into-cohesive-submodules, keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [contracts/decisions, research/reference, research/scheduling, research/documentation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, citations, maintainability]
---
## User-visible outcome

The fifteen pinned line-only citations naming `crates/tiler-ir/src/schedule/builder.rs:NNN` are re-anchored as quoted-fragment citations against the split `schedule/builder/` submodules, so `make citations` is green with the builder split merged and the citations survive future code motion instead of rotting silently.

## Why this exists — filed 2026-08-19 at the builder split's delivery

The split (`56d95195`, delivered, held for batch integration) deletes `builder.rs`, and `check-citations.sh` reports `has 0 lines` for every line-only citation naming it. The citing documents, recorded by the split worker (also durably in its ticket's delivery note) — each contains one or more pinned citations naming `crates/tiler-ir/src/schedule/builder.rs` plus a line number; the numbers after each document below are the cited *builder.rs* lines, not lines in the document: three accepted ADRs — `docs/decisions/0012-physical-reduction-topology.md` (builder line 391), `docs/decisions/0014-reassociation-vs-permutation.md` (831), `docs/decisions/0022-reduction-identities-and-initial-values.md` (403); `docs/research/reference/permitted-divergence-oracle.md` (416, 4767); `docs/research/reference/plan-freedom-sites.md` (620–632, 906, 909, 1029, 1516); `docs/research/scheduling/two-dimensional-cooperative-staging-relation.md` (1147, 1671, 4484); and two `docs/research/documentation/ticket-audit-2026-08-10/reports/` snapshots (1516; 664). Re-derive the exact population at the working base with `grep -rn "schedule/builder.rs" docs/` rather than trusting this transcription — this ticket's first filing mis-stated the pairs as document-line locations and the citation checker caught three past end-of-file, itself a demonstration of why the repair must read each citing claim rather than relocate numbers.

This is exactly the line-only rot AGENTS.md's anchor discipline exists to prevent — the citations passed at base only because the checker verifies a line exists, not what it says.

## Required work

- For each citation, read the citing document's claim, find where the cited content now lives under `schedule/builder/`, and replace the line-only citation with a quoted distinctive fragment (verified by grep against the named file before commit) — several of the old cited lines are non-distinctive (`}`, `|| write.mode != AccessMode::Write`), so mechanical relocation is wrong; the claim decides the anchor.
- The two dated audit-snapshot reports are historical records: follow the repository's convention for them (a dated correction note beside the stale citation, or the checker's documented skip form, whichever the snapshots' own convention uses — read a sibling snapshot repair first rather than inventing one).
- ADR edits here are citation maintenance only; no decision content moves.
- `make citations` green on the tree containing the builder split is the check; run it and quote the summary.

## Coordination

This repair only makes sense on a tree containing the builder split; the coordinator lands it in the same integration batch as (or immediately after) the split merge. Until then `main` stays green because `builder.rs` still exists there.

## Closes when

All fifteen citations resolve as quoted anchors (or documented historical-skip forms), `make citations` is green with the split merged, and no citation names the deleted path.
