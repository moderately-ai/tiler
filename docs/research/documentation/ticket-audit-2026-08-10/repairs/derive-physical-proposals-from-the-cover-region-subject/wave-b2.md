Ticket: derive-physical-proposals-from-the-cover-region-subject
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-physical-proposals-from-the-cover-region-subject/1d2613418636_c99ac54950f2.md
Pre-edit content hash (from ledger): 1d26134186367b3d47df8445aa84a6370e7191679563f12bc3de0bc7bb906926
Post-edit content hash: 092ca74c063422ca63697b3d6fa962169e4d67fba0e7f58f1e0fbcb44d1a6209

Changes applied:
  - Added `## Outcome` documenting delivery at `51042613` (closed `ece8021e`, 2026-08-04): subject-derived proposals via `spell_region`, cover-sourced `RegionWrite`, `UnspellableRegion` declines (tag `0x04`), occurrence-keyed frontier explain subjects, production `record_coverage_gaps` over `SelectedPortfolio::rejections()` at `ExplainStage::CandidateEnumeration`, recognized-proposal golden, contracts/profile/boundary updates, activate-shared-work trigger 1 named/fired.
  - Reframed the three live present-tense Facts under "Why this exists" as **Fact, superseded 2026-08-04** (historical at `57474a09` / pre-landing, with present post-landing clauses), matching the activate-shared-work pattern.
  - Updated the Inference under "Why this exists" so it no longer implies stage-8 silence still blocks shared-work activation; surviving block named as region vocabulary.
  - Optional precision: "What must be true" item 5 now states emission on the plan-selection recording path at `ExplainStage::CandidateEnumeration` with per-region `blocked-covers` aggregation; close condition 2 reworded from "per-cover coverage gap" to one record per unimplemented region with `blocked-covers` multiplicity.
  - Added `## Fact audit — 2026-08-10` summarizing the repair. Status, dependencies, related, and scopes left unchanged.

Optional items skipped (with reason):
  - none (report's optional explain-stage / gap-aggregation precision applied on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required. Report Exact files listed only this ticket; no new remainder tickets; vocabulary widenings and shared-work activation already have owners.

Verification:
  - files read:
    - tickets/derive-physical-proposals-from-the-cover-region-subject.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/derive-physical-proposals-from-the-cover-region-subject/1d2613418636_c99ac54950f2.md
    - tickets/activate-shared-work-duplication-on-the-compile-path.md (superseded-Fact house style)
    - crates/tiler-compiler/src/frontier.rs (spell_region / UnspellableRegion / Silence is not among)
    - crates/tiler-compiler/src/physical.rs (pointwise_region signature; spell_region)
    - crates/tiler-compiler/src/pipeline/trace.rs (record_coverage_gaps, CandidateEnumeration stage comment)
  - checks:
    - git rev-parse / log -1 on `51042613` and `ece8021e` (landing 2026-08-04, close 2026-08-04)
    - grep spell_region / UnspellableRegion / record_coverage_gaps / blocked-covers under crates/tiler-compiler
    - shasum -a 256 of ticket post-edit

Recommended next ledger state:
  integrated
