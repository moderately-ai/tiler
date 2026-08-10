Ticket: implement-general-dag-partitioning
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/implement-general-dag-partitioning/b7b16e6e2744_c99ac54950f2.md
Pre-edit content hash (from ledger): b7b16e6e2744f03ab856e3cffb7d1ae879bf317fb51ef764b3e66005fb5746c5
Post-edit content hash: 270762832663f132a59d90ffa6cadb7b9e79cbb0882d52bdfb336160049e5d60

Changes applied:
  - Added `## Fact audit — 2026-08-10` with a single **Correction — 2026-08-10.** covering the three required live-false / stale claims without rewriting the 2026-08-02 re-read or 2026-08-04 Outcome in place (retired wording retained above for grep).
  - Marked false the Fact that `PhysicalAuthorities::composed` has no production caller / sole production is `governed()` at `pipeline.rs:591`; production is `compile_with_physical_providers` → `composed`; `governed()` and `pipeline::compile` are `#[cfg(test)]`; anchors preferred over line numbers.
  - Marked false the Fact that the enforcer restart condition "now has a graph edge" to `drive-an-external-physical-implementation-provider-through-compilation` as an enforcer dependency; enforcer frontmatter lacks that id in `dependencies` and `related`.
  - Marked Outcome "ends at `review` rather than `done`" as landing-time history; live frontmatter is `status: done`.
  - No status/dependencies/related/scopes metadata changes (report: board edges for this ticket are correct).

Optional items skipped (with reason):
  - Optional graph repair on `implement-boundary-property-enforcers` to restore a `drive-an-external-…` dependency: report marks it optional and owned by enforcer/re-point, not this ticket's Closes when; wave B3 is ticket-only on this file.
  - IMPRECISE ADR 0090 `:125` line-citation / wording (verdict 20): not in required Repair items 1–3; substance directionally right; not rewritten.

Residuals not applied (docs/crates/new tickets/authority):
  - Enforcer frontmatter missing `drive-an-external-physical-implementation-provider-through-compilation` edge (if still intended) — residual on `tickets/implement-boundary-property-enforcers.md` / re-point graph, not partition search.
  - No docs/crates edits (wave B ticket-only); cover search delivery needs none.
  - No new remainder ticket required for partition-search Closes when; `activate-shared-work-duplication-on-the-compile-path` already holds compile-path activation remainder.

Verification:
  - files read:
    - tickets/implement-general-dag-partitioning.md (full, pre- and post-edit)
    - audit report b7b16e6e2744_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/pipeline.rs (`compile` cfg(test); `compile_with_physical_providers` → `PhysicalAuthorities::composed`)
    - crates/tiler-compiler/src/frontier.rs (`governed` under `#[cfg(test)]`; `composed` production)
    - tickets/implement-boundary-property-enforcers.md frontmatter (`dependencies` / `related` lack drive-an-external; prose-only mentions)
  - checks:
    - `rg 'PhysicalAuthorities::composed|PhysicalAuthorities::governed' crates/tiler-compiler/src/` — composed at production pipeline entry; governed only under tests / cfg(test)
    - enforcer frontmatter dependencies are only boundary-property-model + transfer-synchronization; related has device-placement + this ticket
    - post-edit `shasum -a 256` → hash above

Recommended next ledger state:
  integrated
