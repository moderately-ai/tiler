Ticket: realign-the-result-side-display-strings-across-both-refinement-error-mirrors
Wave: B5
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/realign-the-result-side-display-strings-across-both-refinement-error-mirrors/6a62bb5d4f8b_c99ac54950f2.md
Pre-edit content hash (from ledger): 6a62bb5d4f8b3efe12892b3813ec4f663dbf3784014b83ee989fa08e754a1d83
Post-edit content hash: cfe78861c412d2cf9eebf42287fea940fd265e87f646fbd56439d901a8a5bf3c

Changes applied:
  - Rewrote the central Fact: three position-bearing arms (`ResultInterface`, `ResultValueType`, `IncompleteWrite`) still render `region output {position} …` while `position` is ordered result position; separately `ResultArity` renders `region produces {region_outputs} outputs for {results} results` with distinct-tensor `{region_outputs}`; dual-crate mirror and both-must-move constraints retained.
  - Tightened tags from `[documentation, diagnostics]` to `[diagnostics, error-messages]` so dispatch does not treat this as doc-comment-only work.

Optional items skipped (with reason):
  - none (optional tag tighten applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - Product work still open: restating the four result-side Display string pairs in `crates/tiler-ir/src/index/refinement.rs` and `crates/tiler-compiler/src/legality.rs` (Class E — ticket-only wave; crates not edited).
  - Adjacent unscoped debt (optional coordinator note only): IR field docs for the three position fields still say "output" after compiler docs were realigned; not expanded into this ticket's close condition.

Verification:
  - files read:
    - tickets/realign-the-result-side-display-strings-across-both-refinement-error-mirrors.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/realign-the-result-side-display-strings-across-both-refinement-error-mirrors/6a62bb5d4f8b_c99ac54950f2.md
    - grep of `"region output` / `"region produces` under crates/ (4 arms each in tiler-ir and tiler-compiler Display impls)
  - checks:
    - three `region output {position}` + one `region produces {region_outputs} outputs for {results} results` confirmed on both crates
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
