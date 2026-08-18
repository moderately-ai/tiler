---
id: repair-the-checkpoint-no-cast-control
title: Repair the checkpoint no-Cast control
status: in-progress
priority: p2
dependencies: []
related: [ingest-the-checkpoint-as-f32-program-inputs]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [documentation, verification]
claimed_from: todo
assignee: worker-checkpoint-no-cast
lease_expires_at: 1787017663
---
## Fact audit — exact main `dc105234`

**False.** The completed ingestion ticket records `rg -n '\\bCast\\b' ...`. In a single-quoted shell argument those doubled backslashes reach ripgrep literally, so the command does not match `Cast` and cannot prove absence. `printf 'Cast\n' | rg -n '\\bCast\\b'` exits 1, while `printf 'Cast\n' | rg -n '\bCast\b'` prints `1:Cast`. The current fixture source passes the corrected scan. Purpose unchanged: repair only the durable check and record a reachable negative control.

## Closes when

The completed ingestion ticket uses the executable word-boundary expression, records the deliberate literal-`Cast` control and the corrected empty source scan, and `tkt lint`, `make citations`, and `git diff --check` pass.
