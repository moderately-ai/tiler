Ticket: carry-the-exhausted-resource-through-the-budget-refusal
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-the-exhausted-resource-through-the-budget-refusal/323055062ba0_c99ac54950f2.md
Pre-edit content hash (from ledger): 323055062ba03b9f7b32727137a02b7bdd33a42813a904cf6c20075f3bd6b5b3
Post-edit content hash: 9162c9cbb094cb739a83c86c31174cd2455ddc050d741136f4fefa437eaef509

Changes applied:
  - Rewrote **Why this exists** so unit-variant drop and "discarded at the boundary" are explicitly historical (pre-delivery at `acc26984`), with present-tense note that `class_of` now carries payload fields.
  - Annotated 2026-08-08 fact-audit table: unit-arm anchors resolve only via `git show acc26984`; post-change tree has payload arm / `resource.key()`; table rows for unit variant and rule_of marked historical.
  - Added **Fact audit — 2026-08-10** dated correction for the present-tense unit-variant / discarded-at-boundary claims and stating draft delivery + awaiting-decision.
  - Rewrote **Required work** from imperative implementation checklist into "Delivered draft pending Tom's acceptance" with remaining open item = Tom accept/revise only.

Optional items skipped (with reason):
  - Expand `related` to tickets that cite this surface more tightly (`derive-the-region-shape-budgets-from-the-declaration`, `restore-a-planning-phase-refusal-to-the-identity-growth-harness`) — report marks optional graph hygiene only; not required for correctness.

Residuals not applied (docs/crates/new tickets/authority):
  - Identity-growth spike README gap bullet and WALLS unit-variant construction (`spikes/program-planning/identity-growth`) — out of ticket scopes; optional harness hygiene only.
  - Tom acceptance/revision of the public `BudgetExhausted` / `BudgetResource` / `BudgetRefusal` surface — product decision residual; ticket correctly remains `awaiting-decision`.

Verification:
  - files read: full audit report; full ticket; `session.rs` `class_of` payload arm and `rule_of` `resource.key()` arm; regression name in `region_search_budget_coverage.rs`.
  - checks: `shasum -a 256` post-edit; status/deps/scopes/related left unchanged; no crates/docs product edits.

Recommended next ledger state:
  integrated
