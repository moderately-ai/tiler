Ticket: implement-boundary-property-enforcers
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/implement-boundary-property-enforcers/74c4f8d1986b_c99ac54950f2.md
Pre-edit content hash (from ledger): 74c4f8d1986bbb5c1cfbb1d6e393dbe14bd91004a45a39ae9eb988484a5dc0e5
Post-edit content hash: a42d51bb62498ba7fed5c557f9e2b0355c88d09ad7217c7a831daa6288e05ac8

Changes applied:
  - Added dated section "Citation pins from 2026-08-04 have drifted again (2026-08-10)": retires absolute line pins from the 2026-08-04 citation repair; greps/symbol names remain the recheck; argument (carrier-parameterized frontier helpers, multi-site construction, deferral substance) unchanged.
  - Appended 2026-08-10 trigger-check log entry: **not fired** at audit base `c99ac549` (and later main); recheck is `OpaqueCallRegistry::new()` inline in `compile_with_physical_providers` / `PhysicalAuthorities::composed`; live trigger unchanged from 2026-08-09.
  - Frontmatter status/deps/related/scopes left unchanged (report required none).

Optional items skipped (with reason):
  - Optional 2026-07-27 Materialization-row clarity (AliasView constructible via opaque call): report marks not load-bearing while later restatements remain; skipped to avoid rewriting historical dated inventory.

Residuals not applied (docs/crates/new tickets/authority):
  - Product enforcer implementation (compiler insertion / IR) remains deferred — not wave B work.
  - Premise question (if opaque registration stays forever compiler-owned, restart may need a different varying property) left as owner decision; no new ticket filed.

Verification:
  - files read:
    - tickets/implement-boundary-property-enforcers.md (full, pre/post)
    - audit report 74c4f8d1986b_c99ac54950f2.md (full)
    - greps under crates/tiler-compiler/src for bounded_*, UnsatisfiedReason, UndischargedHandoff, tripwire tests, OpaqueCallRegistry::new / PhysicalAuthorities::composed
  - checks:
    - `fn bounded_requirements` / `fn bounded_guarantees` live in frontier.rs (not the 2026-08-04 pins)
    - `enum UnsatisfiedReason` boundary.rs; `UndischargedHandoff {` selection.rs — pins drifted
    - pipeline.rs production: `PhysicalAuthorities::composed(providers, OpaqueCallRegistry::new())` still inline empty registry
    - status remains deferred; no metadata repair required

Recommended next ledger state:
  integrated
