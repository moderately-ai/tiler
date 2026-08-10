Ticket: model-resource-pressure-from-a-register-and-occupancy-model
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/model-resource-pressure-from-a-register-and-occupancy-model/87296e4dd8b9_c99ac54950f2.md
Pre-edit content hash (from ledger): 87296e4dd8b9c99449bc01ece7cef99821def6b69e52ba4b5324160fa62b818f
Post-edit content hash: dc94f1732988af06e2d0165df1f008212630e3066a3b5a89497cc1db23226710

Changes applied:
  - Replaced the incomplete `ResourceRequirements` one-line census ("…and four numerical fields — no register count") with the load-bearing claim only: schedule `ResourceRequirements` carries no register count, anchored at `tiler_ir::schedule::ResourceRequirements` / `crates/tiler-ir/src/schedule/model.rs`.
  - Added **Correction — 2026-08-10** documenting the incomplete enumeration at audit base `c99ac54950f2` (fields include `index_arithmetic`, `synchronization`, subnormals, four `NumericalPermission`s, exceptional-value assumptions; still no register count) and reaffirming the narrowed trigger has not fired.
  - Optional clarity: next-trigger sentence now names live `TargetProfile` / `CheckedTargetProfile` as successor of retired `PrototypeTargetProfile` rather than leading with the absent name.
  - Trigger check log: 2026-08-10 **not fired** entry with recheck anchors.

Optional items skipped (with reason):
  - none (profile-naming optional clarity applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave; report Exact files for eventual implementation (component_cost, CapabilityAxis, estimate consumers, pipeline census) remain product debt outside Phase B ticket-only repair. Metadata (status deferred, dependencies [], related, scopes) left unchanged as required.

Verification:
  - files read:
    - tickets/model-resource-pressure-from-a-register-and-occupancy-model.md (pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/model-resource-pressure-from-a-register-and-occupancy-model/87296e4dd8b9_c99ac54950f2.md (entire report)
    - crates/tiler-ir/src/schedule/model.rs (`ResourceRequirements` full field set)
    - crates/ grep for PrototypeTargetProfile / TargetProfile / CheckedTargetProfile under crates/
  - checks:
    - ResourceRequirements has no register-related field; full field set matches audit verdict 4
    - PrototypeTargetProfile absent as live type name for this surface; CheckedTargetProfile / TargetProfile present
    - status/dependencies/related/scopes unchanged
    - post-edit sha256: dc94f1732988af06e2d0165df1f008212630e3066a3b5a89497cc1db23226710

Recommended next ledger state:
  integrated
