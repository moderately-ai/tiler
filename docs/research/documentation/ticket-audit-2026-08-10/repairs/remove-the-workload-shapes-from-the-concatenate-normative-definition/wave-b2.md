Ticket: remove-the-workload-shapes-from-the-concatenate-normative-definition
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-workload-shapes-from-the-concatenate-normative-definition/22717b607d94_c99ac54950f2.md
Pre-edit content hash (from ledger): 22717b607d947e68c428f81312be076f8ef61fc891ca82d5f94a2145e4be6aae
Post-edit content hash: cbdfc518292c40ef4644556522855bc4edac9d4a1ba32fa0dcd7fd6303e97d40

Changes applied:
  - In `## Current follow-through — 2026-08-09`, replaced the false claim that `ab64f334` removed the workload instance from the normative definition with the verified IR merge `3948ca3c`; added a short **Correction — 2026-08-10.** noting that `ab64f334` only closed tickets (this follow-through and the related roadmap remainder).

Optional items skipped (with reason):
  - none (report listed no optional repairs; metadata was already coherent).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report: no docs/crates edits; no new remainder tickets; identity/API consequences already landed).

Verification:
  - files read:
    - full audit report `22717b607d94_c99ac54950f2.md`
    - full ticket `tickets/remove-the-workload-shapes-from-the-concatenate-normative-definition.md`
    - `git show --stat ab64f334` (ticket-only close) and `git show --stat 3948ca3c` (IR + explain pin + registry guard)
  - checks:
    - commit subjects match audit Fact 9: `ab64f334` = "Close the concatenate shape removal"; `3948ca3c` = "Remove the workload shapes from the concatenate normative definition"

Recommended next ledger state:
  integrated
