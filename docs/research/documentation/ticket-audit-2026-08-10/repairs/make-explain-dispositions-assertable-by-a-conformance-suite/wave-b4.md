Ticket: make-explain-dispositions-assertable-by-a-conformance-suite
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/make-explain-dispositions-assertable-by-a-conformance-suite/32c1d9b8c48f_c99ac54950f2.md
Pre-edit content hash (from ledger): 32c1d9b8c48f43aa21f1c6626faba4fb40825410c0d1ee8fadd6659cf2ce01c2
Post-edit content hash: 43ec5177a718736d25b662dc41dd341d74555107f00e379cf6be4e7503615478

Changes applied:
  - Replaced stale line citations for `ExplainReport` / `render` / `mod explain` / re-export with searchable symbol and doc anchors (`pub struct ExplainReport`, `ExplainReport::render` / "not a parse target", `mod explain;`, `pub use crate::explain::VerifiedCompilationExplain;`).
  - Widened the public-surface Fact to inventory `VerifiedCompilationExplain::{render, semantic_candidate_count}` and state neither exposes dispositions (obligation gap preserved).
  - Relabeled User-visible outcome disposition list as suite-facing superset of ADR 0078's five-item obligation, not a contract quote; declined strategy / cost disadvantage called out as example optimizer outcomes the suite may enumerate.
  - Added **Correction — 2026-08-10** recording line-number drift, surface nuance, and disposition-list labeling.
  - Dropped fragile `:20` / `:24` / "key 19" line-ish references to the conformance-suite ticket in favor of paraphrased Implementation-key content.
  - Metadata unchanged (status deferred, deps/related/scopes correct).

Optional items skipped (with reason):
  - none (optional public-surface widen and disposition-list alignment applied as cheap honesty on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - No crates/docs product work (structured disposition accessor under ADR 0075, or suite maturity out-of-scope prose) — deferred until suite design + Tom decision; trigger remains not fired.
  - Research note `docs/research/extensions/backend-provider-composition.md` still carries stale `:1176` line citation (outside ticket scopes; audit cited only as corroboration).
  - No new remainder ticket; activation path stays suite design + Tom choice.

Verification:
  - files read: ticket; audit report; `crates/tiler-compiler/src/{lib,session,explain}.rs` anchors; `docs/operation-extensions.md` disposition obligation; suite and portfolio ticket `status: todo`.
  - checks: `rg` for `pub struct ExplainReport`, `not a parse target`, `mod explain;`, `VerifiedCompilationExplain`, `semantic_candidate_count`; both dependency tickets still `status: todo`; `shasum -a 256` post-edit.

Recommended next ledger state:
  integrated
