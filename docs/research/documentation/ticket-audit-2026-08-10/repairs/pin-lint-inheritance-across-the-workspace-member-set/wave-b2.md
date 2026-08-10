Ticket: pin-lint-inheritance-across-the-workspace-member-set
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/pin-lint-inheritance-across-the-workspace-member-set/32b9f8d687bd_c99ac54950f2.md
Pre-edit content hash (from ledger): 32b9f8d687bdae8b71ae73947a2b781486b2d3df5de5a7760590cb972b72c6dc
Post-edit content hash: 81ffb55abc251bac3c2fcd59364bea3ba63aa16cd0dfa9edb6fcd1c810651471

Changes applied:
  - Frontmatter `related`: added `pin-the-admitted-unsafe-sites-in-the-workspace-gate` so the closed remainder is graph-visible.
  - `## Remainder, not closed here`: past-tense at-close framing (2026-08-07); added `**Correction — 2026-08-10.**` that the site half was closed later on 2026-08-08 by that ticket via `crates/tiler/tests/workspace_unsafe_sites.rs` (ADR 0079 **Closed workspace-wide 2026-08-08**); do not reopen or file a new site-census ticket.
  - `### Remainder, deliberately not closed`: same past-tense framing and matching dated correction cross-link; both remainders no longer assert present-tense absence of a per-site check.
  - Outcome, close condition, and `status: done` left unchanged.

Optional items skipped (with reason):
  - none (optional related-list graph hygiene was applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required only ticket remainder dating + optional related entry; no docs/crates edits; no new remainder ticket).

Verification:
  - files read:
    - tickets/pin-lint-inheritance-across-the-workspace-member-set.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/pin-lint-inheritance-across-the-workspace-member-set/32b9f8d687bd_c99ac54950f2.md
    - tickets/pin-the-admitted-unsafe-sites-in-the-workspace-gate.md (status: done)
    - docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md (Closed workspace-wide 2026-08-08)
    - crates/tiler/tests/workspace_unsafe_sites.rs (ADMITTED_SITES includes prototypes/serial-sum-run/src/buffer.rs)
  - checks:
    - site ticket frontmatter `status: done`
    - ADR 0079 carries **Closed workspace-wide 2026-08-08** and names `workspace_unsafe_sites.rs`
    - ADMITTED_SITES lists both prototype buffer paths
    - ticket related list includes the site pin; both remainders dated with Correction — 2026-08-10
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
