Ticket: repair-downstream-records-after-sourced-semantic-results
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/repair-downstream-records-after-sourced-semantic-results/a5bcc069e617_c99ac54950f2.md
Pre-edit content hash (from ledger): a5bcc069e6171371f13f6498e43a02a61398d913292a8bddbcbbb6c9d24bdc10
Post-edit content hash: 0262eed3d7cc5b879aae7aaee1cb3fb88b7a4e8f977765e904d33702f356895c

Changes applied:
  - Annotated Completion record with **Correction — 2026-08-10.** stating the close condition is not fully discharged: the Complete-model execution/identity-counts parenthetical at `still thirteen as of 2026-08-08` still restates in present tense that `ValueFact` holds a fixed `Shape`, which is false against `SourcedShape` carriage; five delivered sites remain; residual is research-prose-only.
  - Second **Correction — 2026-08-10.** noting the 2026-08-08 repair-downstream correction's D-19 link text `deferred` is stale; ticket is `awaiting-decision` after 2026-08-09 trigger move.
  - Left frontmatter unchanged (`status: done`; deps/related/scopes already sound per report).

Optional items skipped (with reason):
  - none beyond the report's "optionally annotate Completion" (applied) and optional remainder-ticket path (not filed: no concrete id; Class C ticket-only wave).

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/program-planning/complete-model-ingestion-and-execution.md` — live parenthetical at `still thirteen as of 2026-08-08` still claims `ValueFact` holds a fixed `Shape`; needs dated correction or parenthetical rewrite using the Whole-model 2026-08-08 re-derivation grounds (`static_operand_shape`, pending frontend/compiler, broadcast attribute/D-19, `SourcedShape::encode`). Class C wave forbids docs edit.
  - Same file's 2026-08-08 repair-downstream correction — D-19 presented as `deferred`; redate or rewrite to `awaiting-decision`. Class C wave forbids docs edit.
  - No remainder ticket filed (report allows one under `research/program-planning` owning only this residual if not fixed in-place; no concrete id supplied; coordinator may file or fold into a docs pass).

Verification:
  - files read:
    - entire audit report `…/a5bcc069e617_c99ac54950f2.md`
    - entire ticket (pre- and post-edit)
    - `docs/research/program-planning/complete-model-ingestion-and-execution.md` residual parenthetical and 2026-08-08 correction (anchors `still thirteen as of 2026-08-08`, deferred D-19 link)
    - `tickets/define-the-widening-relation-over-a-symbolic-broadcast-extent.md` frontmatter (`status: awaiting-decision`)
    - `crates/tiler-ir/src/semantic/operation.rs` (`pub(super) shape: SourcedShape`)
  - checks:
    - `rg -n 'still thirteen as of 2026-08-08'` complete-model → live fixed-`Shape` restatement present
    - `rg -n 'SymbolicOperandUnsupported' crates/` → empty
    - D-19 ticket `status: awaiting-decision`
    - `shasum -a 256` ticket → `0262eed3d7cc5b879aae7aaee1cb3fb88b7a4e8f977765e904d33702f356895c`

Recommended next ledger state:
  integrated
