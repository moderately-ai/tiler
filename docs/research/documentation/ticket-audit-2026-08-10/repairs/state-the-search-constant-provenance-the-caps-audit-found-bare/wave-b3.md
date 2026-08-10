Ticket: state-the-search-constant-provenance-the-caps-audit-found-bare
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/state-the-search-constant-provenance-the-caps-audit-found-bare/0565cef6329a_c99ac54950f2.md
Pre-edit content hash (from ledger): 0565cef6329a217cafbe5fdb7baccdd230a144510a28b21429bb0c1ed3df547c
Post-edit content hash: ea5d54e5240fcc41fc94d6fa2d3d1d4fca2088edbe1caa0e69af2bcdc1f71e49

Changes applied:
  - Struck the live Why census "nine bare search-budget values beside five exhaustively-derived ones" as original 2026-08-06 framing that is false at the tree; pointed to eight derived / six literal (Superseded section + superseding ticket).
  - Re-anchored `cover.rs:1521's argued exclusion` to cover's argued exclusion / `is_exhaustive` (dropped rotting line number).
  - Added **Correction — 2026-08-10.** noting Why still carried false 9/5 after Superseded already stated 8/6.

Optional items skipped (with reason):
  - Reciprocal `related` on `state-the-rule-that-a-deterministic-budget-is-a-derivation` naming this closed id — cosmetic graph hygiene on a different ticket; wave B3 permits edit of this ticket only.

Residuals not applied (docs/crates/new tickets/authority):
  - None required. Status/related/scopes/closed_reason unchanged (already correct). Remainder already owned by superseding ticket (`awaiting-decision`); no new remainder under this id.

Verification:
  - files read:
    - tickets/state-the-search-constant-provenance-the-caps-audit-found-bare.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/state-the-search-constant-provenance-the-caps-audit-found-bare/0565cef6329a_c99ac54950f2.md
    - crates/tiler-compiler/src/request.rs (DeterministicBudgets fields + governed docs region)
  - checks:
    - DeterministicBudgets still has fourteen fields; six non-derived literals match superseding 8/6 partition (normalization_rewrites, region_candidates_per_seed, region_expansions, region_covers, region_cover_expansions, physical_plan_combinations).
    - shasum -a 256 of ticket after edit → ea5d54e5240fcc41fc94d6fa2d3d1d4fca2088edbe1caa0e69af2bcdc1f71e49

Recommended next ledger state:
  integrated
