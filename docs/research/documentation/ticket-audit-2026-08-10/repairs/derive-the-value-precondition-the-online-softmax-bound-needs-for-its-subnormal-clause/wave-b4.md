Ticket: derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause/6cf13174c4e3_c99ac54950f2.md
Pre-edit content hash (from ledger): 6cf13174c4e37f3bbc1e7796a93b2ccc1ec3f5ef9f89f600ed1e2445ef6525b7
Post-edit content hash: 7696c8f7f17f2e692e3a412634bde21484857ecda8c681a25e434e6c66cd47cc

Changes applied:
  - Split third "What this ticket must produce" bullet into (a) subnormal-site discharge predicate (spread + rescaled-product family + tree site count) on the first bullet, and (b) optional price-sharpening under max-stable / non-increasing, citing certified-bounds zero-divergence cases, explicitly marked separate from (a).
  - Trigger arm 1: replaced closed reassess ticket as sole admitting actuator with live authorities ADR 0095 (supersession / second reopening condition → admit) and ADR 0101 / decide-whether-to-admit-an-elementary-identity-permission; kept both-admit OR second-rule structure; noted reassess is terminal done / reaffirm decline.
  - Added 2026-08-10 trigger-check log restating not-fired under repaired arm 1 wording.

Optional items skipped (with reason):
  - none (optional trigger log applied as cheap graph hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none — report Exact files for now is this ticket only; activation-time research record and Outcome update deferred with the product work; no new remainder ticket required.

Verification:
  - files read: full ticket; full audit report; reassess ticket Decided/status (done, reaffirm); decide-whether-to-admit Trigger clause on closed reassess; certified-bounds zero-divergence / discharge-rather-than-inherit anchors via grep.
  - checks: shasum -a 256 on edited ticket; status/scopes/dependencies/related left unchanged per report.

Recommended next ledger state:
  integrated
