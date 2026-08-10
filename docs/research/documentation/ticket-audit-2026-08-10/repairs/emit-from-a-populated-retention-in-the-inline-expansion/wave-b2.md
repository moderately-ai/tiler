Ticket: emit-from-a-populated-retention-in-the-inline-expansion
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/emit-from-a-populated-retention-in-the-inline-expansion/b6f1f2ec3420_c99ac54950f2.md
Pre-edit content hash (from ledger): b6f1f2ec3420e40b5925cfb841f4fcaa73ace8520364f7940e944155eadccf85
Post-edit content hash: 88b52532b20228e9b806dbe32529f1690cae7293eb38476fa065aff5dd3c5fe2

Changes applied:
  - Required: dated strike (Corrected 2026-08-10) on `## What actually remains, after the 2026-08-07 correction` present-tense claims that the frontend ignores retention / holds no `DebugRetention` reference; points at Outcome `08714fd7` (`retention.rs` + `report_retained_output` in `aot::deliver`) and the accept-boundary remainder.
  - Required: reframed the remaining read-back / selection / Inference paragraphs so they read as re-scope history closed by Outcome, not live open work.
  - Soft-fixed metal_cache "Always stated" citations `435-440` → `434-440` (two sites: Why-this-exists block and Trigger FIRED narrative).
  - Past-tense + dated correction on the Trigger FIRED "holds no `DebugRetention` reference" sentence (true at fire write; false after Outcome).
  - Optional graph hygiene: `related` now includes `accept-the-retention-read-back-s-caller-visible-boundary` beside the storage-seam parent.
  - Optional log completeness: 2026-08-07 **FIRED** row added under `## Trigger check log` (short form; full narrative still under `## Trigger`).

Optional items skipped (with reason):
  - none (both optional report items were cheap same-ticket hygiene and were applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required ticket prose/metadata only; no docs/crates edits; accept-boundary remainder already exists; no new remainder; status/deps/scopes unchanged).

Verification:
  - files read:
    - tickets/emit-from-a-populated-retention-in-the-inline-expansion.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/emit-from-a-populated-retention-in-the-inline-expansion/b6f1f2ec3420_c99ac54950f2.md
    - crates/tiler-build/src/metal_cache.rs (Always stated docs at 434–440; stage_retention call site)
    - crates/tiler-macros/src/retention.rs (DebugRetention import; report_retained_output)
    - crates/tiler-macros/src/aot.rs (deliver wire of report_retained_output)
  - checks:
    - `rg -n 'Always stated, never discovered' crates/tiler-build/src/metal_cache.rs` → line 434
    - `rg -n 'DebugRetention|report_retained_output' crates/tiler-macros/src` → retention.rs + aot deliver
    - no live `435-440` citations remain on the ticket
    - dated Corrected 2026-08-10 strike present; FIRED log row present; related includes accept ticket
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
