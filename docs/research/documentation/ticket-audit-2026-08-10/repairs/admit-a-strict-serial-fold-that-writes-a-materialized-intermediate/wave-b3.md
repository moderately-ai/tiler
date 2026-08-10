Ticket: admit-a-strict-serial-fold-that-writes-a-materialized-intermediate
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-strict-serial-fold-that-writes-a-materialized-intermediate/2ba9de4187e6_c99ac54950f2.md
Pre-edit content hash (from ledger): 2ba9de4187e668d879cc2e3fd86498780993f3d0461bbf2c8915204b0c81f26f
Post-edit content hash: a51e4a69bef816a371461fbb23beb994a7fe34716b230b4ebdabe58e4f156249

Changes applied:
  - Outcome arm-set / population census: past-tensed landing "all four" / "three of the four"; added **Correction — 2026-08-10** for live five-family `serial_fold_families()` including `SquaredSerialSumThenEpilogue` and `assert_eq!(families.len(), 5)`.
  - Outcome population-test paragraph: reworded "length is 4" / "three of four" as landing-day census; live length 5 via correction cross-ref.
  - Outcome scopes paragraph: framed `reduction_region` hard-coding Output as landing-time fact; **Correction — 2026-08-10** for live `RegionWrite` + `write.tensor()` after epilogue ticket; noted residual Output hard-code comment is cooperative-tree path only.
  - Optional anchor cleanup: replaced stale line numbers on `ValueRole::fills`, `check_stage_accesses`, five `push_tensor_role` / `push_tensor_role_name` sites, and schedule `encode_identity` ownership path with searchable symbol anchors.
  - Added `## Fact audit — 2026-08-10` summarizing the two false live claims and anchor cleanup; status/deps/related/scopes unchanged (done stays done).

Optional items skipped (with reason):
  - research docs' landing-day "four serial arms" prose in `general-compilation-boundary.md` / `minimum-correct-physical-realization-profile.md` — report labels optional and Exact files list them as optional; wave B edits tickets only, residual recorded below.

Residuals not applied (docs/crates/new tickets/authority):
  - optional: `docs/research/program-planning/general-compilation-boundary.md` and `docs/research/program-planning/minimum-correct-physical-realization-profile.md` if catalog hygiene wants current five-arm census beyond landing-day "four" narrative (no product change; wave B ticket-only).
  - none for this ticket's outcome. No remainder ticket; fifth arm write admission already covered by CoverAssigned + five-family test.

Verification:
  - files read:
    - tickets/admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-strict-serial-fold-that-writes-a-materialized-intermediate/2ba9de4187e6_c99ac54950f2.md
    - crates/tiler-ir/src/schedule/builder.rs (`serial_fold_families`, length-5 assert, CoverAssigned serial arms)
    - crates/tiler-compiler/src/physical.rs (`reduction_region` / `RegionWrite` / `write.tensor()`)
    - greps for `fills`, `check_stage_accesses`, `push_tensor_role`, `encode_identity`, `ownership_proof.tensor`
  - checks:
    - re-verified five families and `assert_eq!(..., 5,` in builder.rs
    - re-verified `pub(crate) fn reduction_region` takes `RegionWrite` and uses `write.tensor()`
    - shasum -a 256 of ticket post-edit

Recommended next ledger state:
  integrated
