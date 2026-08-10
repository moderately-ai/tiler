Ticket: decide-whether-the-bf16-conformance-evidence-cell-overstates
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-bf16-conformance-evidence-cell-overstates/a3d9c0ceed62_c99ac54950f2.md
Pre-edit content hash (from ledger): a3d9c0ceed627702b6bbf975d8bd1dbc257ef38082c374056a8225a8cf507434
Post-edit content hash: 63d1cb0ca6a35c0be9b1fbcd36c559fb47f36e80e9090cc4e23de09a83915030

Changes applied:
  - Outcome opening: past-tensed intermediate cell text (2026-08-06 set to `tested guarantee, per-layer corpora only; no end-to-end run`); added **Correction — 2026-08-10.** that that wording is not current, naming the 2026-08-07 end-to-end restatement to `tested guarantee, per-layer corpora and one device run crossing neither the optimizer, the artifact envelope, nor the routing commit` via the Decision paragraph's Corrected 2026-08-07 clause.
  - Question section: marked raise-time framing as **State at raise (2026-08-06)** (vertical `blocked`, no end-to-end run, unstatable flush) and added **Superseded 2026-08-07 / correction 2026-08-10.** pointing at both related tickets `done`, live flush/device vertical, and Verdict for live cell text.
  - Body "What exists" / "What does not exist": retensed as historical 2026-08-06 evidence for the qualification; past-tense no-dispatch and unstatable flush; end-to-end was `blocked` at raise.
  - Added **Correction — 2026-08-10 (composition and flush).** for wire + end-to-end discharge and the live dual-bound cell.
  - Kept historical diagnosis load-bearing (wrong cell quoted; bare `tested guarantee` overstated; vocabulary requires cell-local bound; f16 isolation; maturity cell-read rule; scope "Not done here").
  - Metadata unchanged (`status: done`; related list and scopes left as-is per report).

Optional items skipped (with reason):
  - none (report listed no optional prose/graph items beyond keeping related/scopes, which were left alone).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket repair. Report Exact files list only this ticket; `docs/dtype-support.md` already carries the 2026-08-07 cell restatement — no docs/crates edit owed in wave B. No remainder ticket to mint (composition gaps already owned on ledger / closed end-to-end ticket).

Verification:
  - files read:
    - tickets/decide-whether-the-bf16-conformance-evidence-cell-overstates.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-bf16-conformance-evidence-cell-overstates/a3d9c0ceed62_c99ac54950f2.md (full)
    - docs/dtype-support.md (BF16 physical row field 10; Decision paragraph Corrected 2026-08-07 clause)
    - tickets/conform-the-bf16-vertical-end-to-end.md (`status: done`)
    - tickets/carry-a-bf16-subnormal-realization-the-reference-can-be-told.md (`status: done`)
  - checks:
    - BF16 physical matrix field 10 = `tested guarantee, per-layer corpora and one device run crossing neither the optimizer, the artifact envelope, nor the routing commit`
    - Decision paragraph contains `Corrected 2026-08-07: the run exists and the cell is restated`
    - related tickets both `status: done`
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
