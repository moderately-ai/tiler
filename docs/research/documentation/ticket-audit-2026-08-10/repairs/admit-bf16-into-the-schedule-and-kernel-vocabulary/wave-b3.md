Ticket: admit-bf16-into-the-schedule-and-kernel-vocabulary
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-bf16-into-the-schedule-and-kernel-vocabulary/cd25a0b95add_c99ac54950f2.md
Pre-edit content hash (from ledger): cd25a0b95addcdc10c7442a866619234e8e2aa6b0dca39eadf2f03e8a49640af
Post-edit content hash: 4d59f3b05a33fe17b57d5a69cf8d40e9d0dc7b565b9bb552257bb1c1dfd23138

Changes applied:
  - Outcome evidence bullet: live test name `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_cannot_reach` → `…_the_request_now_reaches`, with rename reason (recognizer wall retired by `widen-the-strategy-recognizer-past-the-f32-wall`).
  - Outcome "Out of scope… confirmed unmoved": framed as landing-day measurement; recognizer `dtype-f32` wall and `msl_type` refusal struck as live claims; **Correction — 2026-08-10** records dtype-recognized/dtype-uniform replacement, request-reachable pure BF16, and `msl_type` → `Ok("bfloat")` after `lower-bf16-to-metal`.
  - Metal scope note: historical "comment now states" / "assertion unchanged" reworded as at-close; **Correction — 2026-08-10** that `the_admitted_bf16_type_has_no_metal_spelling` is gone.
  - Graph maintenance executed: lower-bf16 bullet past-tense at close; **Correction — 2026-08-10** that lower-bf16 is done and spells `bfloat`.
  - Physical-carrier cell: landing text kept as close-of-ticket move; **Correction — 2026-08-10** notes live `docs/dtype-support.md` qualifier `request-reached and schedule-assembled regions` (2026-08-07).
  - Metadata: none (status stays `done`; scopes unchanged).

Optional items skipped (with reason):
  - StorageScalar::Bf16 source doc "Nothing in this workspace produces a Bf16-carried boundary value yet" — report labels optional out-of-scope source hygiene, not ticket prose; residual product debt only.

Residuals not applied (docs/crates/new tickets/authority):
  - `crates/tiler-ir/src/program/model.rs` StorageScalar::Bf16 docs (false "no Bf16-carried boundary value yet") — Exact files optional source hygiene; wave B edits tickets only.
  - No new remainder ticket required by the report.

Verification:
  - files read: full audit report; full ticket; greps for `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_*` (now_reaches in `bf16_numerical_contract.rs`); `KernelType::Bf16 => Ok("bfloat")` in `tiler-metal/src/emit.rs`; no hits for `the_admitted_bf16_type_has_no_metal_spelling` under crates/; dtype-support BF16 Physical carrier cell text.
  - checks: post-edit `shasum -a 256` on ticket; required Repair bullets covered without metadata/docs/crates edits.

Recommended next ledger state:
  integrated
