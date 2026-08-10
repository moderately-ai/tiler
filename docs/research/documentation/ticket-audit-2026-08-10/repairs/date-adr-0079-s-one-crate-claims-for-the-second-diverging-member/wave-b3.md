Ticket: date-adr-0079-s-one-crate-claims-for-the-second-diverging-member
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/date-adr-0079-s-one-crate-claims-for-the-second-diverging-member/08c763d22732_c99ac54950f2.md
Pre-edit content hash (from ledger): 08c763d2273230aad701d97f3cbd789d6b5a342737b5e7cc1aa54ddd1b2166f3
Post-edit content hash: 39cfe69a008416d6fcac98824975de78c714d1e331127fa247c974eddcc54466

Changes applied:
  - related: added pin-the-admitted-unsafe-sites-in-the-workspace-gate and pin-lint-inheritance-across-the-workspace-member-set for graph honesty (optional; both tickets exist)
  - Outcome: appended **Residual — 2026-08-10 (ticket audit).** noting Decision item 4 Proposal was outside d4863d6d's dated set and remains stale present-tense (second-member prospective + "none of the three is" enforcement clause), with pointers to Context Closed mechanically later on 2026-08-07 and Implementation Updated again 2026-08-08; secondary note on Decision item 2 "now reachable as an edit"; landed scope stays closed

Optional items skipped (with reason):
  - none on the ticket; related edges and Outcome residual note both applied

Residuals not applied (docs/crates/new tickets/authority):
  - docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md: date Decision §4 Proposal (second-member arithmetic + "since then none of the three is") rather than overwrite; wave B3 ticket-only — product path residual
  - same ADR: optional re-pin of Context line numbers still citing 25e76d5d digits (device_buffer drifted +1) or symbol-only anchors
  - same ADR: optional date on Decision item 2 "Both of those are now reachable as an edit" for consistency with Context closed-mechanically note (secondary; pin-lint post-land drift)
  - no new remainder ticket filed (report left carrier vs one-line remainder as coordinator preference; residual owned on this ticket Outcome)

Verification:
  - files read: audit report 08c763d22732_c99ac54950f2.md; full ticket; ADR 0079 Proposal/item-2 anchors via rg; pin-ticket and pin-lint-inheritance ticket ids
  - checks: rg confirmed Proposal still contains "three specific extensions" / "since then none of the three"; "Both of those are now reachable as an edit" still present; related ticket ids resolve under tickets/

Recommended next ledger state:
  integrated
