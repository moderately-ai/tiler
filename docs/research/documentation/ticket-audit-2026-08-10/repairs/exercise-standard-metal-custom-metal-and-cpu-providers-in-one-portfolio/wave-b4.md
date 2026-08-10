Ticket: exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio/10198816e604_c99ac54950f2.md
Pre-edit content hash (from ledger): 10198816e604ded62ad6274f88bcdad90254434ee4569ed6fe87cce5c404d2ca
Post-edit content hash: 606565c7ec44d1e3b6b115e0d75416efa3ce9d3a1af19c5b267170935d931bc2

Changes applied:
  - Replaced "public provider composition facade" with accepted per-responsibility composition path (`with_physical_providers`, offered+selected physical provenance, `assemble_plan_artifact`, host `ExecutionEnvironment` + `route_with_adapter`, family policy from express); struck facade wording as false relative to ADR 0090 item 4.
  - Split the policy matrix into within-Metal physical-provider selection vs across-family typed policy; rewrote missing-adapter to host/environment mismatch consistent with select-executable no-registry Outcome; struck custom-Metal-preferred as a family-policy tier.
  - Softened independent target identity claim to one assessed variant-level `TargetProfileRef` / numerical contract via `check_subject`, with independence for backend, representation, payload profile, and compilation subjects across members.
  - Added **Corrected 2026-08-10** block citing ADR 0090 item 4 and select-executable; clarified custom Metal as physical-provider row claim, not a third backend family.
  - Optional related-list hygiene: added `decide-whether-a-loading-host-may-state-several-backend-families`, `select-executable-variants-across-registered-backend-families`, `publish-the-backend-provider-conformance-suite`.

Optional items skipped (with reason):
  - Optional scope add `research/target-profiles` — not applied; spike plan consumes scalar-cpu evidence without editing `spikes/target-profiles/**`; report says add only if that tree is edited.

Residuals not applied (docs/crates/new tickets/authority):
  - Product delivery of `spikes/runtime/backend-provider-portfolio` and live three-backend proof (this ticket's remaining work once decide→express land).
  - No crates/docs product edits in wave B4.
  - No new remainder tickets; residual implementation stays on this node.

Verification:
  - files read:
    - tickets/exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md (pre/post)
    - audit report 10198816e604_c99ac54950f2.md (full)
    - tickets/expose-explicit-backend-provider-and-selection-policy-composition.md (correction style + parent Outcome)
    - docs/decisions/0090 (anchor `There is no BackendProvider`)
    - tickets/select-executable-variants-across-registered-backend-families.md (missing-adapter retired)
    - crates/tiler-artifact/src/program/realization.rs (`check_subject` pins one TargetProfileRef)
    - spikes/runtime/ listing (no backend-provider-portfolio)
  - checks:
    - `shasum -a 256` on ticket after edit
    - status remains `todo` (deliverable still absent; no status change required)

Recommended next ledger state:
  integrated
