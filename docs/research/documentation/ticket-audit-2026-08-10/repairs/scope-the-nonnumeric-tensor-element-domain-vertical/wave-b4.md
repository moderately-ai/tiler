Ticket: scope-the-nonnumeric-tensor-element-domain-vertical
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-nonnumeric-tensor-element-domain-vertical/106f9c0a52e4_c99ac54950f2.md
Pre-edit content hash (from ledger): 106f9c0a52e420381707decc44708af74a12fe8dc56cf25410839daef0cb12ef
Post-edit content hash: 1c5019a58521b0f206bbf5c15681fb0a2e079dc90dc0604edd86eaad03be00a2

Changes applied:
  - Why-this-exists ledger Fact: rewrote "type-system reservations at recognized identity and physical carrier, and `absent/unsupported` everywhere else" to "type-system reservation at recognized identity, physical carrier, and ABI/materialization; `absent/unsupported` on every other maturity column" so the Fact matches both maturity matrices (ABI/materialization is also type-system reservation).
  - Trigger check log 2026-08-04: dropped stale `:239` line citation (landed on D-13); cite by section anchor `#### D-14 — Nonnumeric tensor element domains` (trigger paragraph).

Optional items skipped (with reason):
  - exact dated correction on ledger-cell wording — not required when the Fact is simply rewritten (report marks optional).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required ticket prose only; metadata unchanged; no remainder filing).

Verification:
  - files read:
    - tickets/scope-the-nonnumeric-tensor-element-domain-vertical.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-nonnumeric-tensor-element-domain-vertical/106f9c0a52e4_c99ac54950f2.md (full)
    - docs/dtype-support.md (Nonnumeric semantic + physical maturity rows)
    - docs/research/numerics/dtype-family-research-tracks.md (`#### D-14` heading + trigger paragraph; line 239 still D-13)
  - checks:
    - Physical maturity Nonnumeric row: type-system reservation for Physical carrier and ABI/materialization; absent elsewhere — matches rewrite.
    - D-14 heading present; `:239` would still point at D-13 External/vendor track.

Recommended next ledger state:
  integrated
