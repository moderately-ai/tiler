Ticket: assemble-the-decoder-layer-program
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/assemble-the-decoder-layer-program/7a515d50ab88_c99ac54950f2.md
Pre-edit content hash (from ledger): 7a515d50ab886c61bff4a64d99e83cf055944a4cd3188b19c1abd9d10667f54a
Post-edit content hash: 3f0e10ca5fa4c4bdada682108ad685bbbab52272d41cfbab74d27218bbcce68a

Changes applied:
  - Outcome: struck false live claim that pre-widen binding resources included `buffers` 4 against `4.max(input_count + 1)` actual of 19; dated **Correction — 2026-08-10** restates the historically accurate four-resource refusal set (buffers already 6 as `input_count + 3` → 21; host_expression_nodes 32 vs 43) and keeps semantic_values 16/80 and semantic_operations 8/62 as pre-widen sizing inputs.
  - Outcome: dated the identity-growth coefficients as the 2026-08-05 quadratic ladder; live reading pointed at `spikes/program-planning/identity-growth/` and results README without restating a present-tense fit.

Optional items skipped (with reason):
  - none (metadata already coherent; Exact files listed only this ticket for prose; identity note was required not optional in Repair required).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this audit's repairs (report: no source/code changes; no new remainder).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/assemble-the-decoder-layer-program/7a515d50ab88_c99ac54950f2.md
    - tickets/assemble-the-decoder-layer-program.md
    - tickets/widen-the-deterministic-budgets-to-the-decoder-layer-program.md (buffers premise reconstruction)
    - spikes/program-planning/identity-growth/results/README.md (2026-08-05 quadratic row)
    - crates/tiler-compiler/src/request.rs (grep: live buffers actual `input + output * 4`, governed buffers 30)
  - checks:
    - widen ticket Correction/Fact anchors: buffers was 6 / `input_count + 3` / host also refused
    - identity-growth results README: `134n² + 3650n + 710` on 2026-08-05 ladder; standing fit linear

Recommended next ledger state:
  integrated
