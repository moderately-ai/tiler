Ticket: scope-the-ocp-reduced-precision-float-vertical
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-ocp-reduced-precision-float-vertical/27ce6d976957_c99ac54950f2.md
Pre-edit content hash (from ledger): 27ce6d9769570caf91e5835566a043c07c294c3792192c0fbbfc1eb97f510eb8
Post-edit content hash: fe154b0aa28da04ebfe04b4d46342bffc1ef8760c6a6f78e073f49012d911caf

Changes applied:
  - In `## Trigger check log` first bullet (2026-08-04), dropped the rot-prone `:171` line pin and re-cited Track D-5 by searchable anchors: heading `#### D-5 — OCP reduced-precision floats and E8M0 scale data` and trigger clause `**It has not fired.**` under that section in `docs/research/numerics/dtype-family-research-tracks.md`. Not-fired substance of the log entry is unchanged.

Optional items skipped (with reason):
  - Optional one-line dated note that the prior `:171` pin landed in D-4's status shell command — skipped because the required rewrite replaces the line pin with anchors; the report says the dated note is not required in that case.

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required no docs/crates edits, no new remainder tickets, and no metadata changes)

Verification:
  - files read:
    - full audit report at reports/scope-the-ocp-reduced-precision-float-vertical/27ce6d976957_c99ac54950f2.md
    - full ticket tickets/scope-the-ocp-reduced-precision-float-vertical.md
    - verified D-5 heading and `**It has not fired.**` under it via rg on docs/research/numerics/dtype-family-research-tracks.md
  - checks:
    - rg confirms `#### D-5 — OCP reduced-precision floats and E8M0 scale data` and D-5 trigger `**It has not fired.**` present
    - post-edit sha256: fe154b0aa28da04ebfe04b4d46342bffc1ef8760c6a6f78e073f49012d911caf
    - ticket no longer contains `:171` citation
    - status remains deferred; metadata untouched per report

Recommended next ledger state:
  integrated
