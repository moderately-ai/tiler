Ticket: lower-the-concatenate-occurrence-through-partitioned-writes
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/lower-the-concatenate-occurrence-through-partitioned-writes/6e05771249a6_c99ac54950f2.md
Pre-edit content hash (from ledger): 6e05771249a683097a477ba1e7ce05367712f185910553a86a7d6f4f3d88e1a9
Post-edit content hash: 88224f843ccb93dc60670bb210809e64c194069940ba7e1f6d69d5d28d2ca9c6

Changes applied:
  - Public boundary: struck present-tense "Not self-accepted" / "parked at awaiting-decision" block; retitled section away from "draft, for Tom"; added **Correction — 2026-08-10** that Tom accepted the surface on 2026-08-07 via accept-the-partitioned-concatenate-realization-law (done, without exclusion); noted law.rs "Draft boundary" doc-comment lag is not an open architecture choice on this ticket.
  - Explicit non-goals residual-wall sentence: struck live attribution to admit-the-structural-families-into-the-scheduled-region-vocabulary; **Correction — 2026-08-10** routes the live concatenate request-boundary residual to admit-the-concatenate-family-into-the-scheduled-region-vocabulary (awaiting-decision); kept LogicalAccess-no-partitioned-write and "this ticket did not move the wall".
  - Outcome non-goals residual-wall sentence: same dated correction as above (structural-families done for reindex/broadcast; live residual is the concatenate scheduled-region vocabulary ticket).
  - Left Outcome measurement prose at a86fddc2 (capability count 17; pin 0aa252e0bfa16451; nextest 2935) as historical measurements without silent rewrite.

Optional items skipped (with reason):
  - Optional footnote of live GOVERNED_INDEX_ACCESS_CAPABILITIES = 20 / live explain pin 7ba3d77a66f04638: not required; historical counts and pin stay clearly scoped to a86fddc2; no reader is told those figures are live.
  - related list: report metadata said none; residual is already ticketed elsewhere and is not a dependency of this closed ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - law.rs still labels IndexRealizationLaw::PartitionedConcatenate as "**Draft boundary.** … awaiting Tom's decision" (crates/tiler-ir; out of wave B ticket-only scope). Acceptance body does not record a flip obligation; residual is documentation lag under the accept ticket's aftermath, not missing close work on this ticket.
  - Live residuals already ticketed elsewhere: admit-the-concatenate-family-into-the-scheduled-region-vocabulary; prove-partition-coverage-for-symbolic-extents.

Verification:
  - files read:
    - tickets/lower-the-concatenate-occurrence-through-partitioned-writes.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/lower-the-concatenate-occurrence-through-partitioned-writes/6e05771249a6_c99ac54950f2.md (full)
    - tickets/accept-the-partitioned-concatenate-realization-law.md (status: done; ## Accepted — 2026-08-07; without exclusion)
    - tickets/admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md (status: awaiting-decision)
    - tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md (status: done)
    - crates/tiler-ir/src/index/law.rs (PartitionedConcatenate still carries Draft boundary doc comment)
  - checks:
    - accept ticket status done; Accepted — 2026-08-07 present
    - structural-families ticket status done; concatenate-family ticket status awaiting-decision
    - ticket post-edit contains three **Correction — 2026-08-10** anchors and no live present-tense "parked at awaiting-decision" / "Not self-accepted" outside struck prose
    - shasum -a 256 of ticket after edit → 88224f843ccb93dc60670bb210809e64c194069940ba7e1f6d69d5d28d2ca9c6

Recommended next ledger state:
  integrated
