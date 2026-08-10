Ticket: check-synchronization-realization-before-the-routing-commit
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/check-synchronization-realization-before-the-routing-commit/fa1f1fa5c697_c99ac54950f2.md
Pre-edit content hash (from ledger): fa1f1fa5c6977373db77c9ca021fcb589bc09b461d4e2305474d01940bc5d167
Post-edit content hash: 0a877069519b8ab31836f91958e9b4ca11c1d5c108ddf12326951a68f773ead7

Changes applied:
  - frontmatter `related`: kept `realize-parallel-reduction-strategies-on-metal`; added `discharge-the-derived-requirements-in-the-candle-metal-adapter` and `carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit`
  - Graph maintenance: struck live "candle discharges *no* derived requirement" remainder; reclassified as historical pre-sibling gap and pointed at discharge sibling (`awaiting-decision` for public-boundary acceptance only)
  - added `## Fact audit — 2026-08-10` / **Correction — 2026-08-10.** documenting candle `mod discharge` + `evaluate_synchronization` / `evaluate_index_arithmetic` and that this ticket's scopes stay closed

Optional items skipped (with reason):
  - refresh ExecutionScope/BarrierOrdering line-number side notes — optional; substance already correct; anchors preferred over re-stale numbers

Residuals not applied (docs/crates/new tickets/authority):
  - none required for this ticket-record repair; Exact files listed only this ticket; no crates/prototypes/docs product paths, no new remainder tickets

Verification:
  - files read:
    - tickets/check-synchronization-realization-before-the-routing-commit.md
    - audit report fa1f1fa5c697_c99ac54950f2.md
    - tickets/discharge-the-derived-requirements-in-the-candle-metal-adapter.md (status: awaiting-decision)
    - tickets/carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit.md (status: awaiting-decision)
    - prototypes/candle-metal-adapter/src/adapter.rs (rg: mod discharge, evaluate_synchronization, check_direct_requirements)
  - checks:
    - shasum -a 256 of ticket after edit → 0a877069519b8ab31836f91958e9b4ca11c1d5c108ddf12326951a68f773ead7
    - candle adapter still calls evaluate_synchronization in derived_requirements_hold path

Recommended next ledger state:
  integrated
