---
id: repair-the-stale-row-4-and-row-8-cells-in-the-backend-composition-matrix
title: Repair the stale row 4 and row 8 cells in the backend composition matrix
status: in-progress
priority: p2
dependencies: []
related: [decide-the-backend-provider-conformance-harness-public-surface]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, backend-providers]
claimed_from: todo
assignee: worker-matrix
lease_expires_at: 1787457351
---
## User-visible outcome

A reader checking the eleven-externally-participated-rows-plus-two-exclusions census against its source table finds a table that agrees with ADR 0090's own dated corrections.

## Why this exists

Found 2026-08-22 by `worker-packet` while re-auditing Fact 5 of `decide-the-backend-provider-conformance-harness-public-surface` at base `3291b105`. The census count is right; the table a reader would verify it against is not.

**Fact — the live matrix row 4 is stale.** `docs/research/extensions/backend-provider-composition.md` line 45 prints `nothing installs one` in row 4's installation cell. ADR 0090's 2026-08-08 dated correction retires exactly that: `PhysicalImplementationProvider` is `pub` and re-exported through `pub mod physical_provider;`, and `InstalledPhysicalProviders::installed` installs it.

**Fact — the live matrix row 8 is stale.** The same document carries `no indirection at all — statically Metal` for row 8. ADR 0090 records row 8 as the promoted `assemble_plan_artifact` closure, accepted 2026-08-05.

**Fact — the count is unaffected, and the census must not be restated to match a repair.** `grep -c '^| [0-9]' docs/research/extensions/backend-provider-composition.md` returns `31` across three tables at that base; the responsibility matrix itself is thirteen rows and 13 − 2 = 11 holds.

**Note the grep count and the defect population differ, which is why this ticket names line numbers alongside anchors.** `grep -c 'nothing installs one'` returns `3`, and only **one** of the three is a defect: line 45 is the live matrix row, line 441 is 2026-08-05 audit prose, and line 467 is the dated correction that already refutes line 441 by quoting it. A worker who repairs all three sites would edit a correction into disagreement with the text it corrects.

## Required work

- Re-audit both cells at your own base before editing; the anchors above are `nothing installs one` and `no indirection at all — statically Metal`, both quoted from source rather than from a rendered view.
- Repair only the live matrix cells. Leave the dated audit prose and its correction standing, per this repository's convention that a correction preserves the wording it retires.
- Confirm the census count is unchanged and say which unit you counted.

## Closes when

Rows 4 and 8 of the responsibility matrix agree with ADR 0090's accepted corrections, the dated audit prose is untouched, and the eleven-plus-two census still derives from the table.
