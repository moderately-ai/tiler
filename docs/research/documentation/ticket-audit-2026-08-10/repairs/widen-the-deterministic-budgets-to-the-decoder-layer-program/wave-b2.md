Ticket: widen-the-deterministic-budgets-to-the-decoder-layer-program
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/widen-the-deterministic-budgets-to-the-decoder-layer-program/facf5423b0f5_c99ac54950f2.md
Pre-edit content hash (from ledger): facf5423b0f54decf5770515a5e990b3dc2c64874b161707d09af2288dc05eaf
Post-edit content hash: 9e1c1d0f7c0778f20389dbe43704f23d6628277777fd2ce7d030831afda1d44b

Changes applied:
  - Why this exists: struck stale live Fact (`:1886`, buffers 4, `4.max` actual) as historical filing premise; dated correction points at live 80/62/12/51/30 and owner ticket for 51/30/12; L6 inference marked historical
  - Outcome values table: "Now" → "Landing (`62c63061`)" with dated correction freezing 80/62/3/43/21 and pointing at bound-the-assembled-region-count… for live 51/30/12 and output-aware actuals
  - regions inference: past-tense landing derivation + dated correction for multi-output 12
  - refusal measurement table and pin/domain Facts: labelled landing; dated corrections for live matrix, v6 domain, pin past 8e06e11fdc3a2889 (`7ba3d77a66f04638`)
  - Scope: "Two files" → three compiler sources; stale contracts/optimizer rationalization noted
  - `related`: added bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals and correct-the-l6-budget-item-for-the-output-derived-budgets
  - `## Fact audit — 2026-08-10` summary block
  - status remains done; no reopen

Optional items skipped (with reason):
  - none — optional related edges and Scope "Two files" hygiene applied as cheap same-ticket work

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report; intermediate pin sequence between 8e06e11fdc3a2889 and 7ba3d77a66f04638 not enumerated (audit residual uncertainty, not a ticket edit)
  - docs/crates out of wave scope; no remainder filing required

Verification:
  - files read:
    - audit report facf5423b0f5_c99ac54950f2.md (full)
    - tickets/widen-the-deterministic-budgets-to-the-decoder-layer-program.md (full, pre/post)
    - crates/tiler-compiler/src/request.rs — governed() body (80/62/12/51/30); check_program_budgets output-aware actuals
    - crates/tiler-compiler/src/explain.rs — EXPLAIN_RENDERER_VERSION 7; sealed-trace pin 7ba3d77a66f04638
    - crates/tiler-compiler/src/domains.rs / request.rs — request-subject.v6
    - ticket ids exist: bound-the-assembled-region-count…, correct-the-l6-budget-item-for-the-output-derived-budgets
  - checks:
    - live governed matches audit correction targets
    - Outcome no longer presents 43/21/regions=3 as present-tense live Facts
    - Why premise no longer stands as live Fact with `:1886`/buffers-4/`4.max`
    - shasum -a 256 post-edit ticket

Recommended next ledger state:
  integrated
