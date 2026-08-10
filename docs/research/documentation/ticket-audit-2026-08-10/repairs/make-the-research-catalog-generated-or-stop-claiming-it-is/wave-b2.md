Ticket: make-the-research-catalog-generated-or-stop-claiming-it-is
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/make-the-research-catalog-generated-or-stop-claiming-it-is/4453f7346e72_c99ac54950f2.md
Pre-edit content hash (from ledger): 4453f7346e72efe05115ba534303d80aa4e72c262ffcb160700db49e8131efe7
Post-edit content hash: 5cf73eeb89bb6b7154586f3b242f32168f549efd25ffb57725f9ca4a78a8bc16

Changes applied:
  - Added **Correction — 2026-08-10.** under Outcome (after the "107th record is uncatalogued right now" paragraph) stating that `catalog-the-kani-verification-research-and-spike` later catalogued `docs/research/verification/kani-bounded-encoder-verification.md` and the matching spike row, so the present-tense uncatalogued claim and (`todo`, unclaimed) parenthetical are historical; audit-base population was 107/107 with kani present; detection of uncatalogued records remains absent unless a generator/gate is authorized.

Optional items skipped (with reason):
  - none (report required only the dated Outcome correction; metadata/problem-statement changes were none; no optional bullets).

Residuals not applied (docs/crates/new tickets/authority):
  - Do not file a generator ticket from this audit (would reverse `e197176f` and contradict the merged contract without Tom's decision; Outcome already records the question).
  - Sibling ticket rot out of ownership (e.g. `catalog-the-four-…` still quoting `BEGIN GENERATED RESEARCH CATALOG` as current, or kani ticket text saying this ticket "is open") — noted only per report.
  - No docs/crates edits in this wave.

Verification:
  - files read:
    - tickets/make-the-research-catalog-generated-or-stop-claiming-it-is.md (full, before/after)
    - audit report 4453f7346e72_c99ac54950f2.md (full)
    - docs/research/README.md (marker + kani row via grep)
    - tickets/catalog-the-kani-verification-research-and-spike.md (status: done)
  - checks:
    - pre-edit sha256 matched ledger: 4453f7346e72efe05115ba534303d80aa4e72c262ffcb160700db49e8131efe7
    - `<!-- BEGIN RESEARCH CATALOG -->` present; kani row in research catalog
    - kani carrier ticket status: done
    - post-edit sha256: 5cf73eeb89bb6b7154586f3b242f32168f549efd25ffb57725f9ca4a78a8bc16

Recommended next ledger state:
  integrated
