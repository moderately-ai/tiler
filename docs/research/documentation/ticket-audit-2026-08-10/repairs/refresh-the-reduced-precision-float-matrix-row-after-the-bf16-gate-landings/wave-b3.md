Ticket: refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings/d27aba67a87a_c99ac54950f2.md
Pre-edit content hash (from ledger): d27aba67a87a5187d8d1fec131adc2ea6738f5c9e4bdbc9f66f978359b7ad923
Post-edit content hash: 6dd142ffee75fe8fe1f6ae485b4f55b2fd162d8741456c17cf6344b5f3ac0bb0

Changes applied:
  - Rewrote `## The exact drift` Fact 1 into past tense / pre-`82b82edf` framing so the old "remaining rungs are gated by" closing sentence is not readable as live roadmap content.
  - Fixed the reproduce command: pre-landing hit via `git show 82b82edf^:docs/roadmap.md | grep -n "remaining rungs are gated by"`; post-landing absence via the same phrase on current `docs/roadmap.md`; explicitly warned that grepping the ticket id alone still matches the historical board-derivation mention.
  - Marked Fact 2 (admit/declare done + dtype ledger) as still present-tense verified context; left `status: done` and related/deps/scopes unchanged.
  - Framed the open-time Inference as at-open gate-list staleness; noted `ScalarArithmetic::new` draft boundary still holds in `target.rs` module docs.
  - Added dated `## Outcome — 2026-08-10` terminal record: delivery `82b82edf`, close `5f810e9a`, eight board-derived gates with ownership, R5–R7 unmoved at close, no dtype-ledger edit, other reduced-precision members reread untouched, spin-off filed (now closed obsolete), optional post-close recounts attributed to later landings, and a short Correction line that `status: done` stays correct for the authorized navigation work.

Optional items skipped (with reason):
  - Adding the spin-off to `related`: report marks optional; spin-off is already terminal (`closed` / obsolete); not required for graph hygiene.

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report authorized ticket-only prose; no roadmap/docs/crates remainder of this ticket's authorized work; no new remainder ticket.

Verification:
  - files read:
    - tickets/refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings/d27aba67a87a_c99ac54950f2.md
    - docs/roadmap.md (reduced-precision row: `remaining rungs are gated by` absent; `admit-a-bf16-scalar-arithmetic-subject` count 1; `Recounted 2026-08-07` present)
    - tickets/correct-the-discharged-bf16-target-profile-claim-in-compiler-docs.md frontmatter (`status: closed`, `closed_reason: obsolete`)
    - git show 82b82edf --stat / subject; 5f810e9a subject
  - checks:
    - `grep -n "remaining rungs are gated by" docs/roadmap.md` empty at current tree
    - `grep -c "admit-a-bf16-scalar-arithmetic-subject" docs/roadmap.md` = 1 (historical mention only)
    - ticket still `status: done`; `## Outcome — 2026-08-10` present
    - post-edit sha256: 6dd142ffee75fe8fe1f6ae485b4f55b2fd162d8741456c17cf6344b5f3ac0bb0

Recommended next ledger state:
  integrated
