Ticket: carry-an-exact-rational-quantity-in-the-explain-fact-vocabulary
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-an-exact-rational-quantity-in-the-explain-fact-vocabulary/653d4141d687_c99ac54950f2.md
Pre-edit content hash (from ledger): 653d4141d687d96815201b3c3f1f0e3c1e6df94432df09e2dc748987ecf98dde
Post-edit content hash: 6de0ed0d420ba8c5f4f5ae1d2ed4a0808f500ef069e2d2b1843c45324b283575

Changes applied:
  - Replaced line-number citations for `FactValue` (`:376`), `Quantity` (`:523`), and version constants (`:35-36`) with symbol anchors (`pub(crate) enum FactValue`, `pub(crate) enum Quantity`, `pub(crate) const EXPLAIN_SCHEMA_VERSION` / `EXPLAIN_RENDERER_VERSION`); re-anchored `ExactRational` to `pub struct ExactRational` instead of line 175.
  - In "What this ticket must produce", replaced the claim that the addition forces schema and renderer version steps with the live forced/unforced ledger rule: append-only fact-value/quantity tags are Bits-style unforced unless existing payload/spelling moves; recompute pins only when existing identity or presentation bytes change.
  - Added 2026-08-10 trigger-check-log line: not fired; reassess done-declining; elementary-identity permission deferred; FactValue still lacks rational arm; no price-bearing rewrite.

Optional items skipped (with reason):
  - none (optional trigger-log line applied as cheap hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none. Post-trigger implementation of rational fact/quantity arm remains deferred product work; no docs/crates edits in this wave.

Verification:
  - files read:
    - tickets/carry-an-exact-rational-quantity-in-the-explain-fact-vocabulary.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/carry-an-exact-rational-quantity-in-the-explain-fact-vocabulary/653d4141d687_c99ac54950f2.md
    - crates/tiler-compiler/src/explain.rs (FactValue six arms; Quantity eight u64 arms; EXPLAIN_SCHEMA_VERSION=9; EXPLAIN_RENDERER_VERSION=7; forced/unforced ledger including Bits kind 8 unforced)
    - crates/tiler-ir/src/semantic/accuracy/rational.rs (pub struct ExactRational; NotInLowestTerms)
  - checks:
    - symbol anchors resolve: `pub(crate) enum FactValue`, `pub(crate) enum Quantity`, `pub(crate) const EXPLAIN_SCHEMA_VERSION`, `pub(crate) const EXPLAIN_RENDERER_VERSION`, `pub struct ExactRational`
    - status remains deferred; no deps/scopes/priority change
    - post-edit sha256 of ticket file computed

Recommended next ledger state:
  integrated
