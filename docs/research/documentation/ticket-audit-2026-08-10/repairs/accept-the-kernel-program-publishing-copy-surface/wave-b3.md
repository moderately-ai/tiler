Ticket: accept-the-kernel-program-publishing-copy-surface
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-kernel-program-publishing-copy-surface/fb7da3d9a0dd_c99ac54950f2.md
Pre-edit content hash (from ledger): fb7da3d9a0dd368c68a4f14fce4961ac17f9023701dafa00de2e2665e0996a8f
Post-edit content hash: 5ffcab021b62e17e45bedb5437a92a136824e5a1a4aea02d13aee15551cf5976

Changes applied:
  - Restated the UncoveringStage widening claim as historical at acceptance (**two** accounts: combiner and publishing-copy publisher) rather than live present tense.
  - Added **Correction — 2026-08-10.** naming the live **three** accounts (combiner, publisher, staged-realization consumer), that the third arm landed with staged realization / v11 not this surface, and that the publisher arm and undeclared refusal remain as accepted.
  - Included optional stale line-citation note for sweep anchors `model.rs:1464` / `builder.rs:712` in the same correction block (cheap hygiene on this ticket).

Optional items skipped (with reason):
  - none (optional sweep line-citation note applied with the required correction).

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report Exact files expected only this ticket; no docs/crates remainder and no new remainder ticket.

Verification:
  - files read:
    - tickets/accept-the-kernel-program-publishing-copy-surface.md (pre- and post-edit)
    - audit report fb7da3d9a0dd_c99ac54950f2.md
    - crates/tiler-ir/src/program/verify.rs (verify_stage_accounts docs and three-account checks)
    - crates/tiler-ir/src/program/error.rs (UncoveringStage docs naming three accounts)
  - checks:
    - source anchor `This profile admits exactly three accounts, and all three are declarations:` in verify.rs
    - publisher, combiner, and staged-realization consumer arms present in verify_stage_accounts
    - post-edit sha256 recomputed via shasum -a 256

Recommended next ledger state:
  integrated
