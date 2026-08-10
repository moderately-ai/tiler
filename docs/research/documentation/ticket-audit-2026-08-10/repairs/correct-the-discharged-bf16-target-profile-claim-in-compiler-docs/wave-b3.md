Ticket: correct-the-discharged-bf16-target-profile-claim-in-compiler-docs
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-discharged-bf16-target-profile-claim-in-compiler-docs/d5f9672111e0_c99ac54950f2.md
Pre-edit content hash (from ledger): d5f9672111e02d43fc34ceea17d9f1c4db6954a12a14d8f8605d4e9524de684d
Post-edit content hash: b4cf52cf7f6044e722c7b8c57a5f40781dc54e576393bfdaaf8ca6087158b09d

Changes applied:
  - Added `## Fact audit — 2026-08-10` with **Correction — 2026-08-10.**: 2026-08-09 closure overstated completeness; primary `operation_capabilities` / explain.rs defects already gone; residual live clause in `UNPLANNED_OPERATIONS` gather-contrast (“no target has been asked about their format”); parent stays closed/obsolete; residual is comment-only in `policy.rs`; optional remainder noted.
  - Left frontmatter closed / `closed_reason: obsolete` (prefer dated correction over reopening per report).

Optional items skipped (with reason):
  - Filing new remainder ticket “remove residual discharged BF16 target-profile clause from UNPLANNED_OPERATIONS doc”: optional; wave B does not create new ticket ids; residual product debt recorded below.
  - Reopening parent / clearing closed_reason/closed_note: optional; report prefers keep closed + dated correction.

Residuals not applied (docs/crates/new tickets/authority):
  - `crates/tiler-compiler/src/policy.rs` `UNPLANNED_OPERATIONS` doc gather-contrast: replace “The BF16 rows are unplanned because no target has been asked about their format” with wording consistent with the corrected intro (rowless because no BF16 occurrence consumes a numerical freedom this table may list without width-specific evidence; not because targets cannot state BF16 facts). Wave B3 is ticket-only; crates path out of scope.
  - Optional narrow remainder ticket for that one comment clause (or fold into an open compiler-docs repair wave).

Verification:
  - files read:
    - audit report d5f9672111e0_c99ac54950f2.md (full)
    - tickets/correct-the-discharged-bf16-target-profile-claim-in-compiler-docs.md (full, pre/post)
    - crates/tiler-compiler/src/policy.rs (rg + gather-contrast window around UNPLANNED_OPERATIONS)
  - checks:
    - `rg -n 'no target has been asked about their format' crates/tiler-compiler/src/policy.rs` → hit at residual gather-contrast clause (still live; crates residual)
    - `rg -n 'no target profile can even state' crates/tiler-compiler/src/policy.rs` → only as quoted falsehood under `The ground moved and the conclusion did not`
    - post-edit `shasum -a 256` ticket → b4cf52cf7f6044e722c7b8c57a5f40781dc54e576393bfdaaf8ca6087158b09d

Recommended next ledger state:
  integrated
