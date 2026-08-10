Ticket: accept-the-installed-physical-provider-public-surface
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-installed-physical-provider-public-surface/e815b70fc1bb_c99ac54950f2.md
Pre-edit content hash (from ledger): e815b70fc1bb2656e2542a038adbf9c0354bebf6db417c1730d76e3526170441
Post-edit content hash: 9ddad00f918f09e7150459a2239996112847b75238c6a68718d23cd2deb54ec9

Changes applied:
  - scopes: added `contracts/optimizer` and `contracts/foundation` so the post-acceptance close sweep can edit `docs/compiler/optimizer.md`, `docs/operation-extensions.md`, and `docs/glossary.md` as the close condition requires; kept `contracts/decisions`, `implementation/compiler`, `implementation/build`.
  - Recommendation: tightened "one in-workspace consumer" to "one production in-workspace consumer (`crates/tiler-build/src/plan_artifact.rs`)" (audit optional polish; cheap same-ticket honesty).

Optional items skipped (with reason):
  - none (the only optional prose polish was applied).

Residuals not applied (docs/crates/new tickets/authority):
  - Tom's four open product questions (status correctly remains `awaiting-decision`).
  - Post-acceptance contract/ADR/module-doc draft→accepted language and optional `offered_providers` rename remain product work for the implementing agent after Tom answers — not wave B ticket repair.

Verification:
  - files read:
    - tickets/accept-the-installed-physical-provider-public-surface.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-installed-physical-provider-public-surface/e815b70fc1bb_c99ac54950f2.md
    - ticketsplease.toml (`contracts/optimizer` = docs/compiler/**; `contracts/foundation` includes operation-extensions.md and glossary.md)
    - greps on docs/compiler/optimizer.md, docs/operation-extensions.md, docs/decisions/0090-… for draft language
  - checks:
    - `shasum -a 256 tickets/accept-the-installed-physical-provider-public-surface.md` → post-edit hash above
    - frontmatter scopes line includes both new scopes
    - Recommendation line names production consumer path

Recommended next ledger state:
  integrated
