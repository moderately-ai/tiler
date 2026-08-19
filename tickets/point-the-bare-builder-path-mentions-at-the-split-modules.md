---
id: point-the-bare-builder-path-mentions-at-the-split-modules
title: Point the bare builder-path mentions at the split modules
status: in-progress
priority: p3
dependencies: []
related: [re-anchor-the-schedule-builder-line-citations, keep-a-module-size-and-complexity-census-with-a-split-queue]
scopes: [contracts/navigation, contracts/decisions, research/target-profiles, research/program-planning, research/scheduling, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, maintainability]
claimed_from: todo
assignee: worker-bare-paths
lease_expires_at: 1787151418
---
## User-visible outcome

Live documents stop sending readers to the deleted `crates/tiler-ir/src/schedule/builder.rs`: each bare-path or ambiguous `builder.rs:NNNN` mention in a live document either names the split submodule that now holds the content or is de-pinned as a dated historical statement.

## Why this exists — filed 2026-08-19 from the re-anchoring lane's out-of-fence report

The pinned-citation population was repaired at the batch merge, but twelve live documents still mention the deleted path in forms the citation checker deliberately does not resolve (bare paths and prose suffixes): `docs/status.md`, `docs/roadmap.md`, ADRs 0074/0097/0100, and the research records `transformer-nonlinear-normalization-and-reductions`, `flash-class-capability-set`, `cpu-vector-lane-tier`, `multi-round-two-level-reduction-composition`, `scheduled-region-model`, `subgroup-execution-tier`, `two-level-subgroup-workgroup-reduction`. None fails a gate; each misleads a reader. Re-derive the population at the working base with `grep -rln "schedule/builder\.rs" docs/` filtered to live documents (dated snapshots stay historical).

## Required work

Per mention: read the surrounding claim; if it describes current code, point it at the owning `schedule/builder/` submodule (mod.rs, intrinsic, copy, contraction, elementwise, family, coverage, reduction, tile, proof, diagnostics, tests); if it is a dated statement about a past tree, leave the prose and add the smallest dated locator note only where a reader would otherwise search the wrong file. ADR edits are navigation maintenance only. Gates: `make citations`, `tkt lint`, `git diff --check`, `tkt guard`.

## Closes when

`grep -rln "schedule/builder\.rs" docs/` returns only dated historical records, and every live mention verified against the claim it carries.
