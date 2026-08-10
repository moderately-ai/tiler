Ticket: declare-host-dtype-dispatchability-at-the-consumer-boundary
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/declare-host-dtype-dispatchability-at-the-consumer-boundary/c80565b291b3_c99ac54950f2.md
Pre-edit content hash (from ledger): c80565b291b39eac7cc8e91434011ec8241160a066878adde5ea9e42881af928
Post-edit content hash: e1d3f10e73ff2484ab415ef543a923fa08cbd6912dd670eff0daddce96054f36

Changes applied:
  - Rewrote `### Surviving restatements` serial-sum bullet from present-tense "still a transcribed literal" + stale `:1097` line citation to past-tense leave-behind at this ticket's close, anchor `::declared_route_environment`, and a **Correction — 2026-08-10** that the related ticket `read-the-serial-sum-proofs-dtype-rows-from-its-declaration` is done and the site reads `dtype_dispatchability_rows()` like Candle; residual gap is producer-declared / ADR-0086 only.
  - Metadata left unchanged (status, dependencies, related, scopes) per report.

Optional items skipped (with reason):
  - none (report listed no optional repair items).

Residuals not applied (docs/crates/new tickets/authority):
  - none required. Report forbids re-opening host-earned dtype observation under this ticket; Exact files listed ticket only.

Verification:
  - files read:
    - tickets/declare-host-dtype-dispatchability-at-the-consumer-boundary.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/declare-host-dtype-dispatchability-at-the-consumer-boundary/c80565b291b3_c99ac54950f2.md (full)
    - prototypes/serial-sum-run/src/proof.rs (dtype_dispatch / declared_route_environment sites)
    - tickets/read-the-serial-sum-proofs-dtype-rows-from-its-declaration.md (status + Outcome open)
  - checks:
    - `rg dtype_dispatchability_rows prototypes/serial-sum-run/src/proof.rs` — accessor read present at declared_route_environment
    - related ticket `status: done`
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
