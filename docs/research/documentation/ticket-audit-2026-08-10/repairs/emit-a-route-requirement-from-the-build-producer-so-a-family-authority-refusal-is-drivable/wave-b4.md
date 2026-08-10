Ticket: emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable/dfde7ebe1d60_c99ac54950f2.md
Pre-edit content hash (from ledger): dfde7ebe1d60949d50d07429336c3485808e2b48a3c1b1043b08b9db1f209807
Post-edit content hash: 8e44917f72d6c27fff1e4f787dd2b2bcfcad332067751b47ed0f5b56d61356e5

Changes applied:
  - Replaced stale line pins in "Dispatch outcome and two review corrections — 2026-08-03" with searchable anchors: `fn region_proposal` / `const fn index_arithmetic_requirement` (CompleteU64 → IndexArithmeticU64; classifying carried schedule value, not re-deriving from KernelType::Index); emit `msl_type(KernelType::Index)` / Index arm `uint64_t`; ten goldens (not six); authority ledger Index arithmetic / `64-bit integer math` Apple3 row; artifact-abi `The requirement is a derived requirement and mints no route row` and `what belongs here is decided by derivability` (not :280).
  - Superseded ResourceRequirements omission paragraph: field `pub index_arithmetic: IndexArithmetic` is present and encoded; ownership remains carry-* direct check, not BackendFeature under this ticket.
  - Added `## Fact audit — 2026-08-10` dated correction recording citation repairs and ResourceRequirements supersession; deferred status and trigger text unchanged.
  - Appended 2026-08-10 trigger-check log line: not fired (tiler-build still mints zero route rows; BF16 still profile authority).
  - Metadata left unchanged (status deferred, dependencies [], related, scopes).

Optional items skipped (with reason):
  - none (optional trigger-log line applied as cheap graph hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required; product outcome (producer mint when trigger fires) remains deferred and out of wave B scope. No new remainder tickets; do not absorb carry-*.

Verification:
  - files read:
    - full audit report dfde7ebe1d60_c99ac54950f2.md
    - full ticket before edit
    - crates/tiler-compiler/src/physical.rs (region_proposal / index_arithmetic_requirement anchors)
    - crates/tiler-metal/src/emit.rs (msl_type Index → uint64_t)
    - crates/tiler-metal/goldens/*.metal (ten files, all uint64_t)
    - docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md (64-bit integer math)
    - docs/artifact-abi.md (derived-requirement / derivability anchors)
    - crates/tiler-ir/src/schedule/model.rs (pub index_arithmetic)
    - crates/tiler-build (no RouteRequirement / require_route)
  - checks:
    - shasum -a 256 of ticket after edit → 8e44917f72d6c27fff1e4f787dd2b2bcfcad332067751b47ed0f5b56d61356e5
    - goldens count: 10
    - grep RouteRequirement under tiler-build: empty

Recommended next ledger state:
  integrated
