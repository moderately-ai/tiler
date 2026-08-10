Ticket: decide-whether-the-appended-explain-event-steps-the-schema-version
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-appended-explain-event-steps-the-schema-version/6ffc47e56f0e_c99ac54950f2.md
Pre-edit content hash (from ledger): 6ffc47e56f0efae76c641a2df8d88fdc793b8d828fb269bfd932bcfb693eecf6
Post-edit content hash: 261018c47e7d027bada874214f63b72875e2377a63404a2c71c38504aaf2cdf1

Changes applied:
  - Under Outcome “Nothing decodes” paragraph: added **Correction — 2026-08-10.** striking the one-hit `tiler.explain.trace.v1` census; live tree has two hits (encoder in `explain.rs`, `PinnedDomain` in `domains.rs`); domains row is pin not decoder; no-reader conclusion retained.
  - Same correction block: optional line-citation hygiene — absolute Outcome line numbers are historical; anchors authoritative.

Optional items skipped (with reason):
  - none (optional line-citation note applied cheaply in the same dated block)

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report (Exact files: ticket only; decision and contracts remain correct)
  - residual archaeology noted in the correction itself: whether `domains.rs` held the pin on Outcome write day (not re-derived from git)

Verification:
  - files read:
    - tickets/decide-whether-the-appended-explain-event-steps-the-schema-version.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-appended-explain-event-steps-the-schema-version/6ffc47e56f0e_c99ac54950f2.md (full)
  - checks:
    - `rg -n 'tiler\.explain\.trace\.v1' crates/` → two hits (`explain.rs` encoder, `domains.rs` PinnedDomain)
    - `rg -n 'EXPLAIN_SCHEMA_VERSION: u32|EXPLAIN_RENDERER_VERSION: u32' crates/tiler-compiler/src/explain.rs` → still 9 and 7

Recommended next ledger state:
  integrated
