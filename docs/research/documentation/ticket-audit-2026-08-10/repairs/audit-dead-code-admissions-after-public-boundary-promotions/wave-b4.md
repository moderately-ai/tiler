Ticket: audit-dead-code-admissions-after-public-boundary-promotions
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/audit-dead-code-admissions-after-public-boundary-promotions/f78b7ac65485_c99ac54950f2.md
Pre-edit content hash (from ledger): f78b7ac6548565645e5d0f87d96a0eb602e862d26dede1d39010b1faae5d6282
Post-edit content hash: 04e1b575e1341843e04067f29d90fc76183f2570ecfe7335415dd37047fe1c99

Changes applied:
  - 2026-08-07 correction: replaced false present-tense "now carries only an item-level one" on realization.rs with "after wire-the-delivered-realization-record-into-the-artifact, carries **no** dead_code admission (file- or item-level)"; kept seven file-scope count.
  - Worked example: dropped stale frontier.rs line numbers (`:1346`, `:1410`, `:1436`, `:1456`); kept symbol names (`enumerate_frontier` and the three fns).
  - Worked example residual count: "ten lines" → "eleven residual lines".
  - Added **Correction — 2026-08-10** covering (a) realization zero dead_code allows, (b) frontier eleven residual dead_code lines, (c) full-sweep close set is the seven file-scope files plus unfinished item-level population.
  - Left status `todo`, dependencies, scopes, and related unchanged (none required).

Optional items skipped (with reason):
  - Optional related edge to `declare-metal-numerical-honourability` (named only inside a living source reason clause; empty related is acceptable per report).

Residuals not applied (docs/crates/new tickets/authority):
  - Product work of the admission sweep itself (remove/justify production dead_code allows under declared scopes) — ticket remains the remainder; not wave-B prose.
  - Item-level stale reasons noted in audit Fact 13 (policy/honourability/call_declaration/ImplementationBody) — execution of this ticket, not board repair.

Verification:
  - files read:
    - tickets/audit-dead-code-admissions-after-public-boundary-promotions.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/audit-dead-code-admissions-after-public-boundary-promotions/f78b7ac65485_c99ac54950f2.md
    - crates/tiler-artifact/src/program/realization.rs (dead_code search: empty)
    - crates/tiler-compiler/src/frontier.rs (dead_code: 11 matches)
    - production crates/*/src #![allow( file-scope census (seven dead_code: codec/mod, policy, boundary, explain, accuracy, honourability, feasibility)
  - checks:
    - realization.rs: no dead_code token
    - frontier.rs: eleven dead_code lines
    - seven production file-scope dead_code admits match 2026-08-09 list
    - status left todo; no deps/scopes/related edits

Recommended next ledger state:
  integrated
