Ticket: compose-a-declared-reduction-topology-into-a-semantic-program-evaluation
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation/6ea371250754_c99ac54950f2.md
Pre-edit content hash (from ledger): 6ea371250754c02aa3f38de96ab6c724f0b0d4a2519d043f6b87c8598f7a4f32
Post-edit content hash: 69ad03e2ba62a2f0b9ddde36f76b4a90836e138b9b40ccfb06b8a8da23115ad1

Changes applied:
  - Why-this-exists: stripped numeric `file:line` from live Facts; anchors are now `strict_partial_sums_under` / `strict_partitioned_sum_under`, `ReferenceEvaluator::evaluate`, `pointwise_region` / `NormalizedSerialSum::prologue` / `multi_pass_topology` / `contributor_tensor(subject)`.
  - Outcome 2026-08-06: removed present-tense line cites from The answer, Eliminated, and Finding paragraphs; rewrote “Citations corrected against this tree” into an explicit historical-at-`b9146836` block that no longer asserts “is exact” for moved lines; measurement-boundary text notes those lines have since moved.
  - Added **Correction — 2026-08-10**: Outcome line cites and cooperative/multi-pass re-pins are base-relative to `b9146836` only, not HEAD-valid at `c99ac549`; live construction anchors listed (`partial_reduction_region` / prologue doc clause / `fused_region`+`ReductionTopology::Serial` / `strict_partial_sums_under` / `the_assembled_split_program_matches_the_partitioned_sum_oracle`); status and semantic conclusions unchanged.
  - Metadata (status, dependencies, related, scopes, tags) left unchanged per report.

Optional items skipped (with reason):
  - none required; optional docs-pass note for research-record line rot recorded under residuals rather than a new ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/reference/composed-realization-evaluation.md` and `docs/research/reference/plan-freedom-sites.md` Part 7.4 may share citation line rot — out of ticket-only wave B3.
  - No new remainder ticket; retain + composed-surface acceptance already own implementation/mechanism work.
  - Historical Outcome “Owed navigation row” still mentions `docs/research/README.md:50` as insertion context; Current correction 2026-08-09 already records the row as present (no live authority claim).

Verification:
  - files read:
    - tickets/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation.md (full, before and after)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation/6ea371250754_c99ac54950f2.md (full)
    - crates greps for live anchors: `strict_partial_sums_under`, `strict_partitioned_sum_under`, `partial_reduction_region`, `leaves the prologue, if there is one, where it was`, `multi_pass_topology`, `the_assembled_split_program_matches_the_partitioned_sum_oracle`, `CanonicalValue::utf8("strict-left-fold")`, `SplitAxis`, `ReferenceEvaluationRequest`
  - checks:
    - symbol greps confirm anchors exist under crates/
    - post-edit `shasum -a 256` of ticket → 69ad03e2ba62a2f0b9ddde36f76b4a90836e138b9b40ccfb06b8a8da23115ad1
    - residual numeric cite scan: only historical “Citations at landing” prose quoting old numbers plus the historical navigation insertion line

Recommended next ledger state:
  integrated
