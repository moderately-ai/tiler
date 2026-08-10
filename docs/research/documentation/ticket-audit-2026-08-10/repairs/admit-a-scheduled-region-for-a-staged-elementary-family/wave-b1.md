Ticket: admit-a-scheduled-region-for-a-staged-elementary-family
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-scheduled-region-for-a-staged-elementary-family/66b1fa2f9481_c99ac54950f2.md
Pre-edit content hash (from ledger): 66b1fa2f9481627dde91403de704f6079f9bd358b7f202f4de8a2c0db70992fb
Post-edit content hash: c4ad42f4cea193892a34f55b028f14d0500f90672e951908562171214f0ea027

Changes applied:
  - "The surface this touches": replaced false public `#[non_exhaustive]` claim on `ScalarProgram` with the true deliberately-not-`#[non_exhaustive]` design, pointed at accept-the-fold-with-epilogue-scheduled-region.
  - Added **Current-state correction — 2026-08-10** after Outcome Checks: (a) fold-with-epilogue and family-realization-law-query accepted the parked drafts; (b) account-for staged realization / v11 removed program-assembly realization-stage-unaccounted; (c) measurement is `a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit` (old spells_both_stages name retired); (d) fourth closing condition holds at repository level via the remainder.
  - Optional historical mark on "Where the wall is" pre-landing Facts (base b3d5a9ed) so they are not re-read as live walls.

Optional items skipped (with reason):
  - none (the optional historical mark was applied as cheap prose hygiene on this same ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report (Exact files: ticket only; no remainder to file; metadata already correct).

Verification:
  - files read:
    - tickets/admit-a-scheduled-region-for-a-staged-elementary-family.md (entire, pre/post)
    - audit report 66b1fa2f9481_c99ac54950f2.md (entire)
    - tickets/accept-the-fold-with-epilogue-scheduled-region.md (Outcome + non_exhaustive wording + 2026-08-09 correction)
    - tickets/accept-the-registered-family-realization-law-query.md (Outcome + 2026-08-09 correction)
    - crates/tiler-ir/src/schedule/model.rs (ScalarProgram derive/enum, SquaredSerialSumThenEpilogue Accepted public surface, TAG 0x2A)
    - crates/tiler-compiler/src/pipeline/tests.rs (a_staged_family_program_compiles…, the_staged_regions_compute…; spells_both_stages absent)
    - crates/tiler-compiler/src/program.rs (AssemblyStagedRealization / push_staged_realization)
    - crates/tiler-ir/src/domains.rs (tiler.kernel-program.v11)
  - checks:
    - `ScalarProgram` has `#[derive(Clone, Debug, Eq, PartialEq)]` and no `#[non_exhaustive]`
    - accept-the-fold wording: `ScalarProgram is deliberately not #[non_exhaustive]`
    - shasum -a 256 ticket → c4ad42f4cea193892a34f55b028f14d0500f90672e951908562171214f0ea027
    - rg anchors: Historical pre-landing, deliberately not non_exhaustive, Current-state correction 2026-08-10, compiles_and_computes name

Recommended next ledger state:
  integrated
