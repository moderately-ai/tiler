Ticket: scope-the-bit-reinterpretation-family-against-its-storage-carrier
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-bit-reinterpretation-family-against-its-storage-carrier/aba2942e0b40_c99ac54950f2.md
Pre-edit content hash (from ledger): aba2942e0b40fdcb7e073fb52edd870de9d8a85e0ea846ab50bbc25b5d458b43
Post-edit content hash: cb5d7974a4e8164ad940659b1056db17116f010eb1626eb851740ec20883c447

Changes applied:
  - Narrowed ADR 0018 Fact: replaced "tension … stated and unresolved" / taxonomy "not an accepted rule" claim with accepted preserve-bits posture from ADR 0018 Decision + Consequences (`bit reinterpretation do not destroy payload bits`); residual open work is RQ-OP-02 classification and writing that boundary into the admit/refuse record; non-canonical payload reaching a bitwise consumer is a consequence of preserve, not proof ADR 0018 lacks a position.
  - Aligned "What the work would be" so it no longer says land the boundary "as a rule rather than as this record's inference"; now "record ADR 0018's already-accepted preserve-bits boundary in the admit-or-refuse record".
  - Optional 2026-08-10 trigger log: **not fired**; U4 honourable on measured profile but still one packing / no bit-reinterpretation producer; marks 2026-08-05 "never dispatched" as historical.

Optional items skipped (with reason):
  - none (optional dated trigger recheck applied as cheap on-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - Taxonomy F-04 language still saying the preserve posture is "not an accepted rule" lags ADR 0018 Consequences; out of deferred ticket scope (report: no taxonomy edit unless separate repair filed).
  - Fact 2 still folds taxonomy's "first family whose semantic result depends on a physical fact" Inference into a Fact paragraph; report required only ADR 0018 narrow, not that relabel.
  - No crates/docs product work; status remains deferred (trigger unfired).

Verification:
  - files read: audit report; full ticket; ADR 0018 (bit reinterpretation / preserve anchors); metal test name via grep; packing ticket honourability correction for cross-check.
  - checks: status/deps/related/scopes unchanged; trigger still not fired; post-edit sha256 computed.

Recommended next ledger state:
  integrated
