Ticket: decide-whether-stage-one-semantic-exploration-adopts-an-e-graph
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-stage-one-semantic-exploration-adopts-an-e-graph/8645af9603f4_c99ac54950f2.md
Pre-edit content hash (from ledger): 8645af9603f423d8a10bfe9d5b3cb0c0d765717e6ac554a39a1fa06a84151df4
Post-edit content hash: 7f556c4aa10b5e7d319cf7f3c67dbf50e8edac38adbbb9370c85e78ec8a49e0c

Changes applied:
  - User-visible outcome: `enumeration into region formation` → `enumeration into contract grouping (formalism stage 2)`
  - Why-this-exists Fact: dropped `(relayed)` on constant-factor inapproximability (both extraction halves read as of 2026-08-05)
  - Trigger 2026-08-09 anchors: `ORDERED_REASSOCIATE_ADD_RULE` / `ORDERED_REASSOCIATE_MULTIPLY_RULE` (not `*_F32_RULE`)
  - Trigger 2026-08-05 reproduction: `grep -rn 'RewriteRuleIdentity::new("tiler' crates/ --include='*.rs'`
  - What the decision owes bullet 4: acquisition closed; owed action is reading the FMCAD 2024 re-check, not locating the paper
  - Dated **Correction — 2026-08-10.** under Why this exists covering (a) region-formation → contract grouping, (b) relayed → read, (c) wrong const names + over-reporting grep
  - Metadata unchanged (status deferred, deps, related, scopes)

Optional items skipped (with reason):
  - none (optional FMCAD/closed-acquisition clarify applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - optimizer contract still carries a "relayed" extraction-inapproximability sentence relative to the formalism (report residual uncertainty; docs/ out of wave B edit scope)
  - product decision remains deferred pending probe measurement and rewrite-vocabulary growth — not wave B work

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-stage-one-semantic-exploration-adopts-an-e-graph/8645af9603f4_c99ac54950f2.md
    - tickets/decide-whether-stage-one-semantic-exploration-adopts-an-e-graph.md
    - crates/tiler-compiler/src/rewrite.rs (COMMON_SUBEXPRESSION_RULE, ORDERED_REASSOCIATE_ADD_RULE, ORDERED_REASSOCIATE_MULTIPLY_RULE)
    - docs/research/region-search/rewrite-search-formalism.md (extraction read status; enumeration into stage 2)
  - checks:
    - `rg 'ORDERED_REASSOCIATE_(ADD|MULTIPLY)_RULE' crates/tiler-compiler/src/rewrite.rs` → production consts present; F32-suffixed const names absent
    - `rg 'RewriteRuleIdentity::new\("tiler' crates/ --glob '*.rs'` → four production lines
    - formalism: both extraction halves read; stage-1 handoff is enumeration into stage 2
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
