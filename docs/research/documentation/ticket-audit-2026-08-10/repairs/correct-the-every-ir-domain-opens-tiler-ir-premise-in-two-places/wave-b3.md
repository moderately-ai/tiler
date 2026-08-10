Ticket: correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-every-ir-domain-opens-tiler-ir-premise-in-two-places/6ad0c57021ab_c99ac54950f2.md
Pre-edit content hash (from ledger): 6ad0c57021ab2387815968cd8e00e7d22a7cb6cbf2ef92b47bcedac8d16e1c6a
Post-edit content hash: 976aa240137b4238f48b797e85b8382f88d6adfa3022bbb6b0cbc62cb50bbbea

Changes applied:
  - Required prose: 2026-08-08 audit bullet "Seven `tiler-artifact` domains agree with `EXPR_DOMAIN` for 23 bytes" → "Six … (the six `tiler.artifact-program.*` governed domains)", with explicit note that `ROUTE_REQUIREMENT_DOMAIN` shares only 14 bytes through `tiler.artifact`.
  - Optional opening-block clarity: added **Superseded — 2026-08-10.** after the two Reported paragraphs, marking **24 of 38** / **684** fully superseded (46 of 60; 1080), source half retired at `f9b0b67d`, live contract remainder owned by the follow-up.

Optional items skipped (with reason):
  - optional related/remainder edges for unsplit source residuals (codec test doc; refinement first-byte / dependency framing): not applied — would need new ticket ids or unowned related targets; residuals recorded below.

Residuals not applied (docs/crates/new tickets/authority):
  - Live contract false premise in `docs/artifact-abi.md` — already owned by `repair-the-artifact-abis-stale-cross-crate-no-prefix-argument` (not this ticket).
  - `crates/tiler-artifact/src/program/codec/tests.rs` still states a cross-crate dependency sentence as if neither crate depends on the other (false on the dependency edge); unsplit comment defect, separate ticket if pursued.
  - `crates/tiler-ir/src/index/refinement.rs` residual first-byte-after-`tiler.` / dependency-direction framing for coverage-graph domain that lags the `domains.rs` NUL-terminator argument; unsplit comment defect, separate ticket if pursued.

Verification:
  - files read:
    - full audit report `6ad0c57021ab_c99ac54950f2.md`
    - full ticket before edit
    - grep of `tiler.artifact-program.*` / program identity domains under `crates/tiler-artifact/src` (six `tiler.artifact-program.*` + `ROUTE_REQUIREMENT_DOMAIN` = `tiler.artifact.route-requirement.v1\0`)
  - checks:
    - Six-domain claim matches ARTIFACT_DOMAIN, STAGE_KEY_DOMAIN, PAYLOAD_KEY_DOMAIN, PROVIDER_KEY_DOMAIN, DEFERRED_KEY_DOMAIN, DELIVERED_REALIZATION_DOMAIN
    - post-edit sha256: 976aa240137b4238f48b797e85b8382f88d6adfa3022bbb6b0cbc62cb50bbbea

Recommended next ledger state:
  integrated
