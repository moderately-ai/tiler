Ticket: derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound/663f52bb47f1_c99ac54950f2.md
Pre-edit content hash (from ledger): 663f52bb47f1d346d7f9b36e3397e0021e20127193b0095762f67304b8c394bc
Post-edit content hash: 138dd6d8d2393607b3868212a4cd4e2104b6417fc92dcc08f3c1757e7849d40c

Changes applied:
  - Added one dated **Correction — 2026-08-10.** under Four-outcome disposition: separate ticket `done`; measure trigger fired 2026-08-09 and status `todo`; expose ticket `done` with `elementary_relative_accuracy` retrieving eps_exp so ADR 0095 second reopening condition has third and second clauses satisfied and first (admission rule) still open; catalog rows transferred to research and spikes READMEs. Landing narrative left standing.
  - Soft-edited Closes when "replaced by a statement covering both" → "supplemented by a sibling derivation that closes the underived-tree axis (covering both forms)" (optional item from report, same-ticket prose hygiene).

Optional items skipped (with reason):
  - none (optional Closes when soft-edit applied)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/numerics/tree-fold-online-softmax-bound.md disposition sentences that may still say eps_exp is not retrievable (sibling research drift; Exact files optional product path; out of wave B ticket-only edit)

Verification:
  - files read:
    - tickets/derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound.md (full, pre and post)
    - audit report 663f52bb47f1_c99ac54950f2.md (full)
    - tickets/separate-the-rescaling-price-from-the-observed-fold-divergence.md (status: done)
    - tickets/measure-whether-a-targets-exponential-is-exact-at-zero.md (status: todo; trigger log 2026-08-09 fired)
    - tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md (status: done; elementary_relative_accuracy)
    - crates/tiler-compiler/src/target/accuracy.rs (elementary_relative_accuracy present)
    - docs/research/README.md and spikes/README.md (tree-fold catalog rows present)
  - checks:
    - grep status on separate/measure/expose tickets
    - rg elementary_relative_accuracy in accuracy.rs
    - rg tree-fold catalog anchors in research/spikes READMEs
    - sha256 post-edit: 138dd6d8d2393607b3868212a4cd4e2104b6417fc92dcc08f3c1757e7849d40c

Recommended next ledger state:
  integrated
