Ticket: decide-whether-a-derived-budget-belongs-in-the-request-subject
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-a-derived-budget-belongs-in-the-request-subject/42f9870c6a77_c99ac54950f2.md
Pre-edit content hash (from ledger): 42f9870c6a77f213bc1c51eaeac1cb327f6858b0be5bfa098da1e90389e7f918
Post-edit content hash: 00423780c309c58384141b6b95b9fb45dcbbecad6e361e9be028b8fa0ec6bf29

Changes applied:
  - Replaced present-tense live domain `tiler.compiler.request-subject.v5` with `tiler.compiler.request-subject.v6` in Tom's question and the "If the answer is yes" encoding-step sentence (encoder prefix verified `tiler.compiler.request-subject.v6` in `request.rs`).
  - Added `## Fact audit — 2026-08-10` recording live domain `v6` (shape-environment subject fold), residual comment-only `v5` in `request.rs`, and that this ticket's present-tense names were corrected.
  - Rewrote scopes-correction body so removal of `research/verification` rests on this ticket not editing those paths; struck false "neither … exists" existence claim.
  - Dated correction that `docs/research/verification/` and `spikes/verification/` both exist at current tree; scope removal still correct.
  - Tightened pin-comment-pair note per Fact 12: the two `request.rs` anchors address budget-byte pins vs staged-subject pins, not two budget-pin populations.
  - Dropped `research` tag (optional hygiene; body already re-scoped to decision; `decision` / `needs-tom` remain).

Optional items skipped (with reason):
  - none applied that were skipped; optional research-tag drop and Fact 12 tighten were both cheap and applied.

Residuals not applied (docs/crates/new tickets/authority):
  - Tom's keep vs remove decision remains open; status stays `awaiting-decision`.
  - Keep branch still needs `docs/compiler/optimizer.md` open-slot sentence flip after Tom answers (docs/ out of wave B).
  - Remove branch still needs a separate identity-domain migration ticket on live domain `v6` with enumerated pin population (do not land under this node).
  - Stale `v5` domain strings in `request.rs` comments remain crate residual until a landing opens that file.

Verification:
  - files read:
    - tickets/decide-whether-a-derived-budget-belongs-in-the-request-subject.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-a-derived-budget-belongs-in-the-request-subject/42f9870c6a77_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/request.rs (rg for request-subject.v[0-9]; live encode is v6)
    - docs/research/verification and spikes/verification (directory existence)
  - checks:
    - `rg -n 'request-subject\.v5|neither of which exists' tickets/decide-whether-a-derived-budget-belongs-in-the-request-subject.md` → empty
    - `shasum -a 256 tickets/decide-whether-a-derived-budget-belongs-in-the-request-subject.md` → 00423780c309c58384141b6b95b9fb45dcbbecad6e361e9be028b8fa0ec6bf29

Recommended next ledger state:
  integrated
