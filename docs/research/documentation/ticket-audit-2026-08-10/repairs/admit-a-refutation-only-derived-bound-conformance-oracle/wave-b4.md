Ticket: admit-a-refutation-only-derived-bound-conformance-oracle
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-refutation-only-derived-bound-conformance-oracle/6c7c5810b35c_c99ac54950f2.md
Pre-edit content hash (from ledger): 6c7c5810b35caefd8bde24eeb7f1d3ef43a09e25b7ebba98b2d1026aa8c659cf
Post-edit content hash: feafedd9dcf805fc01ce090403f7e97b7d883ccb1a1c5edf2734009883fd9842

Changes applied:
  - Moved 2026-08-06 and 2026-08-09 not-fired bullets from `## Graph maintenance` into `## Trigger check log` (Graph maintenance retains only the "Filed by …" sentence).
  - Fixed 2026-08-06 clause-1 reproduce command to anchors present in `docs/research/apple-targets/numerical-behaviour.md` (`### 34\.|does not survive`) and noted that token `NotPreserved` lives on the measure ticket Outcome.

Optional items skipped (with reason):
  - 2026-08-10 reconfirm under Trigger check log: report says not required once 2026-08-09 is relocated and remains accurate.

Residuals not applied (docs/crates/new tickets/authority):
  - `permitted-divergence-oracle.md` Part 7 item 5 overstates fire relative to this ticket's two-clause AND trigger; roll-up still labels the measure experiment `todo` while the measure ticket is `done` — research/reference record maintenance, out of ticket-only wave B.

Verification:
  - files read: audit report; full ticket; repro anchors via `rg` on numerical-behaviour.md (finding 34 heading) and measure ticket Outcome (`NotPreserved`).
  - checks: post-edit sha256 of ticket file; metadata (status/related/scopes) left unchanged per report.

Recommended next ledger state:
  integrated
