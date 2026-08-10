Ticket: scope-the-remaining-bit-preserving-structural-families
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-remaining-bit-preserving-structural-families/73595513da1f_c99ac54950f2.md
Pre-edit content hash (from ledger): 73595513da1f0a63e5d666157e5f2eef2848e6289936ca3ef8b9db61a4b01f71
Post-edit content hash: 8993db642c60adafca2c0094e9a3a2f607c3c27e0b6d77ddc7c61e8d139e69d1

Changes applied:
  - Replaced stale matrix quotation "R5 for the two admitted families; views and bit-preserving copies stay R2" with live matrix wording "R6 for the two admitted families, bounded to offline translation on one measured toolchain row and with R7 unmet; views and bit-preserving copies stay R2".
  - Rewrote the shared-implementation half of the "## Why this is deferred…" Inference: dropped false claims that identity is an empty-permutation Reindex and that repetition is exactly the map shape broadcast emits; retained valid split-rule / O-15 grouping (no numerical content, Pure, D7 read map or copy); recorded reindex identity refusal (`reindex.form.identity-mapping` / `ReindexFormError::IdentityMapping`) and broadcast non-unit stretch refusal (`broadcast.mapping.stretch-source-not-unit`).
  - Appended 2026-08-10 trigger-check-log entry: **not fired**; no identity/copy/tile/repeat keys; historical note that 2026-08-05 "eighteen registered operation keys" count is stale and population includes at least `tiler::gather-f32@1`.
  - Metadata left unchanged (status deferred, empty dependencies, related list, scopes, tags) per report.

Optional items skipped (with reason):
  - Optional activation-time design note (F-03 cannot be admitted as a Reindex form without overturning identity refusal; F-27 needs a new relation class beyond Broadcast): not required for board correctness; non-goals and rewritten Inference already make the refusals load-bearing without inventing product decisions.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required; Exact files listed only this ticket.

Verification:
  - files read:
    - tickets/scope-the-remaining-bit-preserving-structural-families.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-remaining-bit-preserving-structural-families/73595513da1f_c99ac54950f2.md (full)
    - docs/roadmap.md (structural matrix row R6 wording)
    - crates/tiler-ir/src/semantic/reindex.rs (IdentityMapping / reindex.form.identity-mapping)
    - crates/tiler-ir/src/semantic/broadcast.rs (StretchSourceNotUnit / broadcast.mapping.stretch-source-not-unit)
    - crates grep for gather-f32 and absence of identity-f32/copy-f32/tile-f32/repeat-f32 keys
  - checks:
    - matrix cell matches R6 admitted / R2 views and bit-preserving copies
    - no identity/copy/tile/repeat keys under crates
    - gather-f32 registered (historical count drift)
    - shasum -a 256 on ticket after edit

Recommended next ledger state:
  integrated
