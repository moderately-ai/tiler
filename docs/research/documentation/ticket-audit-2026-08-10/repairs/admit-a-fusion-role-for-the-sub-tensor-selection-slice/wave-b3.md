Ticket: admit-a-fusion-role-for-the-sub-tensor-selection-slice
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-fusion-role-for-the-sub-tensor-selection-slice/8de0b290fa6d_c99ac54950f2.md
Pre-edit content hash (from ledger): 8de0b290fa6df12ff35b068c0a5bdead9111c0ba2bb8a49f0704a33fc02699b3
Post-edit content hash: c6dfc47eca2e80a7e26dec387fa10c3310ad08212124bd63d0e0b424eec3ced6

Changes applied:
  - Outcome Measurement on the explain pin: past-tense freeze as landing-time observation (no present-tense "still pins" / "passes unchanged").
  - Added **Correction — 2026-08-10, on the explain pin absolute string**: live pin at audit base `c99ac549` is `7ba3d77a66f04638` (re-read at `explain.rs` sealed-trace site), not landing-era `de9ad4cc087697d8`; mechanism Inference retained.

Optional items skipped (with reason):
  - None beyond the pin freeze itself was listed optional; the report labeled the pin freeze "optional but recommended" and it was applied because without it the Outcome Measurement was a false live Fact.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/indexing/sub-tensor-selection-fusion-role.md` stale key-count / frontmatter (`disposition: pending`, `implementation_status: not-started`) — Outcome already flagged; outside declared scopes; wave B3 ticket-only.
  - `docs/research/semantic-graph/operation-family-delivery-graph.md` O-06 M4 cell lag — same class of out-of-scope catalog debt; not this ticket's undeclared remainder.
  - Outcome `UNPLANNED_OPERATIONS` "fifth entry" wording is landing-population historical (list now six with gather); Repair required did not demand a freeze; left as historical Fact body.

Verification:
  - files read:
    - tickets/admit-a-fusion-role-for-the-sub-tensor-selection-slice.md (full)
    - audit report 8de0b290fa6d_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/explain.rs (pin grep: `7ba3d77a66f04638` at sealed-trace site)
    - crates/tiler-compiler/src/fusion_legality.rs (`roles.insert(` count 15; slice under CoordinateRelation)
    - crates/tiler-compiler/src/policy.rs (UNPLANNED list: six entries including slice and gather)
    - tickets/admit-a-fusion-role-for-the-sequence-extension-concatenate.md (house-style pin Correction pattern)
  - checks:
    - `rg 'tiler-explain-v7 request=' crates/tiler-compiler/src/explain.rs` → live pin `7ba3d77a66f04638`
    - `rg -c 'roles.insert\(' crates/tiler-compiler/src/fusion_legality.rs` → 15
    - `shasum -a 256` on ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
