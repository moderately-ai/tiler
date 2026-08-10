Ticket: derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order/0a52598d5ce4_c99ac54950f2.md
Pre-edit content hash (from ledger): 0a52598d5ce4743a575cf571da60bba08e0b448f1383019b435fc1622e50a7a2
Post-edit content hash: 6e473507d974f032917644467471d4f888da7c8e11f9787c79378c5fe044325c

Changes applied:
  - Replaced stale line citations (`evaluate.rs:484`, `cooperative.rs:887`, `physical.rs:1654`, `builder.rs:4767`) with searchable anchors: `partition * chunk + within`; `fn workgroup_tree_tile`; `accumulation: request.numerical_contract().arithmetic,`; `fn multi_round_tile_fixture`; and the physical construction call `let tile = tiler_ir::schedule::workgroup_tree_tile(participants)`.
  - Rewrote the live CooperativeTile three-line compiler census as historical only; living deferral prose now uses the 2026-08-09 construction-anchor discipline (production path is `workgroup_tree_tile` with `rounds: 1`).
  - Narrowed the accumulation population: dated correction that `verify_accumulation_width` / `RealizationWitness` site 4.8 leave verified plans at element width; Trigger limb rewritten so accumulation fires only if the accumulation authority is widened, not merely if a field is written elsewhere.
  - Added 2026-08-10 Trigger check log entry: not fired; construction still `workgroup_tree_tile` + `rounds: 1`; CooperativeTile type-name hit count retired as population; verified accumulation spend empty; covers still exact product.
  - Status/deps/related/scopes left unchanged (deferred remains correct per report).

Optional items skipped (with reason):
  - Optional related cross-link to plan-freedom-sites / witness accumulation correction — not required; accumulation narrowing is already stated on this ticket with witness/builder anchors; adding graph edges would be a product/graph decision outside pure prose repair.

Residuals not applied (docs/crates/new tickets/authority):
  - None required. Report listed no docs/crates edits and no remainder ticket for board graph. Product debt when fired remains future tiler-reference evaluators / RealizationNotEvaluable (outside wave B ticket-only scope).

Verification:
  - files read:
    - full audit report `.../0a52598d5ce4_c99ac54950f2.md`
    - full ticket (pre-edit)
    - greps/reads: `partition * chunk + within` in evaluate.rs; `fn workgroup_tree_tile` + `rounds: 1` in cooperative.rs; `workgroup_tree_tile` / `CooperativeTile` in tiler-compiler; `accumulation: request.numerical_contract().arithmetic` in physical.rs; `verify_accumulation_width` + `fn multi_round_tile_fixture` in builder.rs; RealizationWitness accumulation doc in witness.rs; participant-round map docs in model.rs
  - checks:
    - production construction anchor present; workgroup_tree_tile still `rounds: 1`
    - verify_accumulation_width still equality-only
    - no status/metadata change needed

Recommended next ledger state:
  integrated
