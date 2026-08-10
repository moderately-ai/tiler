Ticket: accept-the-registered-family-region-sequence-query
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-registered-family-region-sequence-query/5dd5676afd91_c99ac54950f2.md
Pre-edit content hash (from ledger): 5dd5676afd91aa5cf4c8db439113c4b2c980d1ba39f89940b6fcfe400828f77e
Post-edit content hash: 6403d632d23541bcbcf92c330cf5a4dd80d756a2e9f9520217ab20d83d9b5379

Changes applied:
  - Evidence: replaced stale softmax-as-no-law false row with current four-row matrix (rms-norm true, softmax true / StagedSoftmaxF32, multiply false / single-region, slice false / no law); anchored to test docblock "Four rows and one agreement"; agreement program still named as rms-norm + multiply.
  - Closes when: marked historical terminal focus superseded by Outcome and 2026-08-09 source correction; noted draft label is gone while method remains in use.
  - Added ## Evidence correction — 2026-08-10 recording that post-softmax-law registration moved the no-law exemplar from softmax to slice under the same accepted predicate.

Optional items skipped (with reason):
  - related array: report listed sibling accept-the-registered-family-realization-law-query as optional graph completeness only and stated metadata/related set remain valid — left unchanged.

Residuals not applied (docs/crates/new tickets/authority):
  - none (repair is ticket prose only; no docs/crates remainder required).

Verification:
  - files read:
    - tickets/accept-the-registered-family-region-sequence-query.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-registered-family-region-sequence-query/5dd5676afd91_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/refinement.rs (test the_family_region_sequence_query_agrees_with_the_resolved_law + "Four rows and one agreement" docblock; family_realizes_region_sequence presence)
    - crates/tiler-ir/src/index/law.rs (realizes_region_sequence match arms including StagedSoftmaxF32)
  - checks:
    - assert! family_realizes_region_sequence(rms_norm) true; softmax true; multiply false; slice false with no-law message
    - IndexRealizationLaw::realizes_region_sequence matches StagedSoftmaxF32 among multi-region variants
    - shasum -a 256 on ticket after edit

Recommended next ledger state:
  integrated
