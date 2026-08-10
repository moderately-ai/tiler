Ticket: expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate/d2a53fec0016_c99ac54950f2.md
Pre-edit content hash (from ledger): d2a53fec00165d713650008f603d99fede325039b853d8d5fe2175fe39e4e47a
Post-edit content hash: 3637013d73f90a8c96a032537e755ac8b7659ef940d1094a8d0c44760200758a

Changes applied:
  - Struck the false present-tense Outcome identity-pin claim (`explain.rs:4174`, `689c3aefc30f48d3` unmoved and green under the broad hex grep).
  - Added **Correction — 2026-08-10.**: discover the sole request-subject pin via `rg -n 'tiler-explain-v7 request=' crates/tiler-compiler/src/explain.rs` (currently the matches fixture carrying `7ba3d77a66f04638`); do not freeze the hex in this ticket; architectural claim that the land did not touch identity encoders/profiles stands; pin movement expected from unrelated snapshot/budget/registry work.
  - Same dated correction notes historical line-pin rot in the 2026-08-06 narrowing note (`required_elementary_accuracy` :739 vs current `pub(crate) fn required_elementary_accuracy`; softmax contract function start ~:468 vs ulp body a few lines below) and tells readers to locate by symbol.
  - Metadata unchanged: `status: done`, dependencies empty, related list left as-is.

Optional items skipped (with reason):
  - Reverse-related hygiene on `derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause` (would edit another ticket; report marks optional and out of this ticket's edit set).
  - Stale `todo` label for this id inside `reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller` body (sibling-ticket rot; report says not this ticket's frontmatter defect).

Residuals not applied (docs/crates/new tickets/authority):
  - None required by the report. No new remainder tickets; deferred remainders already filed. No docs/crates edits in scope for wave B2.

Verification:
  - files read:
    - tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md (full, pre- and post-edit)
    - audit report d2a53fec0016_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/explain.rs (pin discovery via rg)
    - crates/tiler-compiler/src/target/accuracy.rs (fn required_elementary_accuracy line)
    - crates/tiler-ir/src/semantic/softmax.rs (contract / ULP tolerance symbols)
  - checks:
    - `rg -n 'tiler-explain-v7 request=' crates/tiler-compiler/src/explain.rs` → sole pin value `7ba3d77a66f04638` at line 3883; no `689c3aefc30f48d3` under crates
    - `rg -n 'fn required_elementary_accuracy' crates/tiler-compiler/src/target/accuracy.rs` → line 799
    - `rg -n 'fn softmax_f32_exponential_accuracy_contract|SOFTMAX_F32_EXPONENTIAL_ULP_TOLERANCE|AccuracyPredicate::ulp' crates/tiler-ir/src/semantic/softmax.rs` → contract at 468, ulp at 493, tolerance 12 at 231
    - post-edit sha256 of ticket file recomputed after edit

Recommended next ledger state:
  integrated
