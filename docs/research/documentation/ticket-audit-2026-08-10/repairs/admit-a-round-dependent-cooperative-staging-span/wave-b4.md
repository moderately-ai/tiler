Ticket: admit-a-round-dependent-cooperative-staging-span
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-round-dependent-cooperative-staging-span/c536b55d7fac_c99ac54950f2.md
Pre-edit content hash (from ledger): c536b55d7fac2bbdb0cedcb72d9378316957148c8f45e6f8bad000a339161fc7
Post-edit content hash: 0923e82a4945b2f084d966d7baaef1467118f881c63e3f60f913011120ce8e5c

Changes applied:
  - Replaced stale numeric line citations with searchable anchors: `builder.rs:1205-1210` → `verify_cooperative_tile` occupancy phrase `phase sequence once, which is exactly one round`; `cooperative.rs:70-75` → module-doc anchor `A log-depth tree needs two things`; `cooperative.rs:887` / `:79` on trigger log → `rounds: 1` next to anti-dependency comment / `A log-depth tree needs two things`.
  - Restated tiled-contraction Fact with `for (uint k0 = TILE; …)` and identical pre-loop slot indices (no fragile line range).
  - Added optional related edge `derive-the-multi-round-two-level-reduction-composition` (2026-08-05 log already treats it as second round-invariance consumer; not a depends-on).
  - Added `## Fact audit — 2026-08-10` / **Correction — 2026-08-10.** recording the rotten line ranges at base `c99ac54950f2`.
  - Left status `deferred`, empty `dependencies`, scopes/tags unchanged; no new trigger-check line (2026-08-09 still holds).

Optional items skipped (with reason):
  - None material; optional related hygiene and optional dated correction both applied.

Residuals not applied (docs/crates/new tickets/authority):
  - None for this wave. Future activation (not audit repair) would still touch `StagedSpan` / verifier / possible identity under crates — product work, not B4.

Verification:
  - files read:
    - tickets/admit-a-round-dependent-cooperative-staging-span.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-round-dependent-cooperative-staging-span/c536b55d7fac_c99ac54950f2.md
    - crates/tiler-ir/src/schedule/cooperative.rs (module docs ~75–98; `workgroup_tree_tile` `rounds: 1` ~891)
    - crates/tiler-ir/src/schedule/builder.rs (occupancy map ~1888–1893)
    - spikes/scheduling/metal_contraction_vertical/kernels.metal (pre-loop 117–119; k0 loop 128–133)
  - checks:
    - `rg` anchors: `phase sequence once, which is exactly one round` (builder.rs); `per-access active-participant subset` / `functions of the round ordinal` / `rounds: 1` (cooperative.rs)
    - `shasum -a 256` ticket → 0923e82a4945b2f084d966d7baaef1467118f881c63e3f60f913011120ce8e5c
    - no remaining live `builder.rs:N` / `cooperative.rs:N` citations outside the dated correction note

Recommended next ledger state:
  integrated
