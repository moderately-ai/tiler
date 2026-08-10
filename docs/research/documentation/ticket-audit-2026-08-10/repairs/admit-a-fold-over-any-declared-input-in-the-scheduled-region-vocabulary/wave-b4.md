Ticket: admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary/6a1bb02cbf6f_c99ac54950f2.md
Pre-edit content hash (from ledger): 6a1bb02cbf6facdb4b3dcd40811d619c88dd6786347bbfbccbb322022a29a32a
Post-edit content hash: 83212380c984e52a140a949f030231839a39ddf4db58ec2d745683f35a32bc20

Changes applied:
  - Restated the Boundaries split/cooperative bullet: bare-sum serial + multi-pass partial + cooperative share `ContributorTensor::DeclaredDomain` (widen `admits`); fused paths use `Exactly(FIRST_INPUT)` / `read.tensor == FIRST_INPUT` and move with the fused half; compiler bare-sum is delete `sum-contributor-ordinal` (regions already bind via `contributor_tensor`); compiler fused is delete `fused_contributor_tensor`'s FIRST gate plus named test flips.
  - Dated **Correction — 2026-08-10.** on that bullet recording the retired false claims (Both name Exactly(FIRST_INPUT); whole compiler-side change is deleting the guard).
  - Optional graph hygiene: added `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` to `related` (write-half citation already in Boundaries; ticket status done).

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation remains this ticket's closes-when (IR DeclaredDomain widen, fused FIRST widen, request guard delete, test flips, identity recompute) — not in scope for wave B ticket repair.
  - Implementer note only (not a new ticket): IR unit test `a_bare_serial_sum_folds_a_declared_input_or_a_materialized_domain` currently asserts second-input refusal and must invert with DeclaredDomain widening; DeclaredDomain doc comments still say "the first input tensor".
  - Exact files for implementation (crates/tiler-ir schedule builder, tiler-compiler request/physical/conformance, possible docs comments) left as residual product debt.

Verification:
  - files read:
    - tickets/admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary/6a1bb02cbf6f_c99ac54950f2.md (full)
    - crates/tiler-ir/src/schedule/builder.rs (`DeclaredDomain::admits`; multi_pass_family / cooperative_family bare vs fused arms)
    - grep census: ContributorTensor::DeclaredDomain|Exactly in builder.rs; sum-contributor-ordinal / fused_contributor_tensor in tiler-compiler
  - checks:
    - StrictSerialSum multi_pass Partial → DeclaredDomain; cooperative bare sum → DeclaredDomain (verified)
    - fused/squared/maximum parallel → Exactly(FIRST_INPUT) (verified)
    - DeclaredDomain::admits still Intermediate || FIRST_INPUT (verified)
    - status left todo; no dependencies/scopes change required

Recommended next ledger state:
  integrated
