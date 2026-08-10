Ticket: accept-the-evaluation-order-preservation-target-fact
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-evaluation-order-preservation-target-fact/a28fbcf78e82_c99ac54950f2.md
Pre-edit content hash (from ledger): a28fbcf78e827b031ab474408b805d11b8e11f5208a20aa9278f1a95abacb84f
Post-edit content hash: 9aad8b22d912b1da055b8f1b7fc35d18fa0bd344b3d87fb31c1aaddd254a86cc

Changes applied:
  - In `## What a reader should check before deciding`, replaced the live claim that `the_declared_profile_states_one_barrier_realization`'s 1,999-byte pin is the evaluation-order identity check with the evaluation-order-specific checks: `complete_descriptor`'s conditional section encode (`if !evaluation_order.is_empty()`) and `the_declared_profile_answers_unknown_on_evaluation_order_preservation`'s domain-substring absence for `tiler.target-profile.evaluation-order-preservation.v1` (report option b).
  - Added a short **Correction — 2026-08-10.** noting that the acceptance-time 1,999 pin was not evaluation-order-specific, is now 2_099 after the measured cost-row section (+100 encoding-predicted bytes), and evaluation-order contributed none of that move (optional dated pin-drift note, retained because the retired figure is named as historical context).

Optional items skipped (with reason):
  - Finding-34 reassoc-vs-reassoc+contract prose precision on the exclusion paragraph: report marks it optional / non-blocking; accepted surface remains sound under the Inference; exclusion paragraph was not required for the pin repair.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by this audit (Exact files listed only this ticket; no docs/crates edits owed).

Verification:
  - files read:
    - tickets/accept-the-evaluation-order-preservation-target-fact.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-evaluation-order-preservation-target-fact/a28fbcf78e82_c99ac54950f2.md (full)
    - crates/tiler-build/src/metal_declaration.rs (`the_declared_profile_states_one_barrier_realization` pin `2_099`; `the_declared_profile_answers_unknown_on_evaluation_order_preservation` domain absence)
    - crates/tiler-compiler/src/target.rs (`EVALUATION_ORDER_DOMAIN`; conditional `if !evaluation_order.is_empty()`)
    - docs/research/apple-targets/numerical-behaviour.md (finding 34 reassoc+contract measurement vs reassoc Inference — for optional skip decision)
  - checks:
    - `assert_eq!(descriptor.len(), 2_099` present in `the_declared_profile_states_one_barrier_realization`
    - evaluation-order domain substring absence asserted on the unknown-preservation test
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
