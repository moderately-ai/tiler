Ticket: qualify-the-model-level-claims-per-apple-device-and-toolchain-row
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/qualify-the-model-level-claims-per-apple-device-and-toolchain-row/82107e0b1aca_c99ac54950f2.md
Pre-edit content hash (from ledger): 82107e0b1acab5ed0650cd4e5e987a0bafc233fc94150457ccbee854394f49df
Post-edit content hash: 78cdd7d92da0dbf9fb33fec4e88e4c525f941db92bbcf3fddfa83829d4c29c7c

Changes applied:
  - Replaced overloaded "four claims" device-dependency list in "## Why this exists…" with L8's six matrix rows under four named claims; explicitly included Estimated cost (device none; target-profile identity; 8/9 components Unknown).
  - Rewrote "## Closes when" from "one row per claim" to L8 claim-axis (six rows or equivalent nested four-claim presentation that still carries halves and Estimated cost).
  - Added Required work bullet: reference-side State must reflect 2026-08-01 joint measurement (P-reorder, P-flush, P-elem measured jointly), not the understated transferred L8 State string.
  - Added "## Fact audit — 2026-08-10" dated correction for schema alignment and reference-side understatement.
  - Left environment-key wording standing (named profile + host + OS/Xcode/offline compiler; native translator Unknown on AOT; not registryID).
  - Metadata unchanged (status todo, dependencies, related, scopes, tags sound per audit).

Optional items skipped (with reason):
  - none (optional dated correction applied as preferred)

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave; delivery still open once harness lands (product work, not ticket-record repair). No docs/crates edits; no new remainder tickets.

Verification:
  - files read:
    - tickets/qualify-the-model-level-claims-per-apple-device-and-toolchain-row.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/qualify-the-model-level-claims-per-apple-device-and-toolchain-row/82107e0b1aca_c99ac54950f2.md
    - docs/research/program-planning/model-level-qualification.md (four claims table; matrix rows; post-transfer 2026-08-01 correction)
  - checks:
    - rg anchors: Estimated cost row, "matrix row above is now understated", "P-flush and P-elem measurable today" in L8
    - shasum -a 256 on ticket post-edit

Recommended next ledger state:
  integrated
