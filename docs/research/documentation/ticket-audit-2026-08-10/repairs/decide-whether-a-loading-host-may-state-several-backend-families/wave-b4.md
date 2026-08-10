Ticket: decide-whether-a-loading-host-may-state-several-backend-families
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-a-loading-host-may-state-several-backend-families/a12b3938f92e_c99ac54950f2.md
Pre-edit content hash (from ledger): a12b3938f92e5e55e98fc1ffd8528dbfcea4f4cde4704ffc715330a75560e1a1
Post-edit content hash: f5590f4a4def720a09488d5bc425e135b06f1fa20099f75b0c7ff82170ea0177

Changes applied:
  - related: added express-the-typed-backend-family-selection-policy (dependencies remains empty)
  - Closes when: replaced stale "composition ticket's family-policy key is rewritten" with host-model authority sites (ADR 0090 item 4 / host.rs / any new fragment Tom requires) and express Implementation-key rewrite; explicit do-not-reopen-terminal-composition note
  - exercise Fact: named express and join as co-dependencies alongside composition
  - Option A cost: "three out-of-scope consumers" → composition-named consumers plus other direct ExecutionEnvironment call sites
  - Option B prevent: "crate nothing may depend on" → consumer facade workspace libraries must not depend on (distinguished from tiler-conformance)

Optional items skipped (with reason):
  - none (all optional tightenings applied as same-ticket graph/prose hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - Tom decision / close still pending: host-model record under docs/decisions/ and/or host.rs; express keys rewritten against A or B; Outcome + provenance; possibly express scopes if B
  - exact post-decision document of record (ADR 0090 correction vs new ADR vs host.rs-only) left to Tom
  - no new remainder tickets (express and exercise already correctly edged)

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-a-loading-host-may-state-several-backend-families/a12b3938f92e_c99ac54950f2.md
    - tickets/decide-whether-a-loading-host-may-state-several-backend-families.md (pre- and post-edit)
    - tickets/express-the-typed-backend-family-selection-policy.md (exists check for related link)
    - tickets/join-build-time-producers-to-runtime-adapters-through-artifact-identity.md (exists check for exercise Fact links)
  - checks:
    - shasum -a 256 tickets/decide-whether-a-loading-host-may-state-several-backend-families.md → f5590f4a4def720a09488d5bc425e135b06f1fa20099f75b0c7ff82170ea0177
    - status left awaiting-decision (Tom has not decided)
    - related successor link targets present on disk

Recommended next ledger state:
  integrated
