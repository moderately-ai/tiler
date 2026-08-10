Ticket: admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs/62d7f11c8b7d_c99ac54950f2.md
Pre-edit content hash (from ledger): 62d7f11c8b7d0181cfe9f0a797c6dbc7dc98144eb6e6b899173b5d18873cfd0f
Post-edit content hash: 6ea0101dbe17d2a760f037a2a14095bf2a99612a1ba54d45bce42364b732762e

Changes applied:
  - IR prerequisite: restated live rule as non-descending / ascending with gaps (with intermediate-read strict-ascent landing and two-reads non-descending evolution); renamed test anchor to `read_accesses_must_name_non_descending_declared_inputs`; **Correction — 2026-08-10.** for retired strictly-ascending name/rule.
  - Outcome identity measurement: framed `request=689c3aefc30f48d3` and metal `e3ac0aee…` / `14cbccad…` as landing-time (not live sealed values); **Correction — 2026-08-10.** records live explain pin `7ba3d77a66f04638` and live metal `39e76563…` / `7e00d9fa…`.
  - Optional: soft-dated FIRST_INPUT "five places" census as landing-time; **Correction — 2026-08-10.** points at fold ticket eleven-site census (2026-08-09).
  - Metadata: none (status, related, scopes, dependencies already correct).

Optional items skipped (with reason):
  - none — optional five-places soft-date applied as cheap graph hygiene on this ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required ticket-only prose; no crates/docs edits; fold remainder already filed and related; no new remainder ticket.

Verification:
  - files read:
    - tickets/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs/62d7f11c8b7d_c99ac54950f2.md (full)
    - crates/tiler-ir/src/schedule/builder.rs (grep: `read_accesses_must_name_non_descending_declared_inputs`, non-descending comment)
    - crates/tiler-compiler/src/explain.rs (grep: live pin `request=7ba3d77a66f04638`)
    - crates/tiler-build/src/metal_plan.rs (live ARTIFACT_IDENTITY / supersession history for e3ac0aee / 14cbccad)
    - house-style sample: tickets/admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary.md Correction blocks; peer repair wave-b1 for two-reads ticket
  - checks:
    - live IR test name `read_accesses_must_name_non_descending_declared_inputs` present in builder.rs
    - live explain pin `request=7ba3d77a66f04638` sole sealed pin string match
    - live metal pins `39e76563…` / `7e00d9fa…`; historical `e3ac0aee…` / `14cbccad…` only in supersession history
    - ticket no longer presents strictly_ascending test name as live; pin values framed as landing-time
    - sha256 post-edit: 6ea0101dbe17d2a760f037a2a14095bf2a99612a1ba54d45bce42364b732762e

Recommended next ledger state:
  integrated
