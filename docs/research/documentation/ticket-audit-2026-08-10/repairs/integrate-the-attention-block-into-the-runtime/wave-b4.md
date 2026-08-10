Ticket: integrate-the-attention-block-into-the-runtime
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/integrate-the-attention-block-into-the-runtime/fe074d6d4942_c99ac54950f2.md
Pre-edit content hash (from ledger): fe074d6d4942770cef7f25c0db7407324deb701e3160c2fc5c5d22d652130b1b
Post-edit content hash: d3de95eef1a6049b440d86e752964bccbd300b99eba039bd2dd09310e371ccc2

Changes applied:
  - User-visible outcome: qualified "first time a transformer block of any kind runs" to Metal device through the accepted AOT and runtime route, and noted host reference evaluation already exists via assemble-the-causal-self-attention-block-program.
  - Evidence prerequisite thirty-case proof Fact: replaced historical `FlushSubnormalsToZeroF32` with live `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`, retaining the historical spelling in parentheses.
  - Metadata left unchanged (status todo, dependencies, related, scopes, tags, priority) per Repair required.

Optional items skipped (with reason):
  - none (optional FLUSH spelling clarity was applied as a cheap same-ticket fix).

Residuals not applied (docs/crates/new tickets/authority):
  - none for wave B; product delivery (plan-the-materialized-attention-decomposition, realize-the-attention-contractions-on-metal, device route, roadmap L4 capability cell) remains the ticket's open work, already owned by existing upstream/downstream tickets.

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/integrate-the-attention-block-into-the-runtime/fe074d6d4942_c99ac54950f2.md
    - tickets/integrate-the-attention-block-into-the-runtime.md
    - tickets/assemble-the-causal-self-attention-block-program.md (anchor: evaluates end to end at the C1 row)
    - crates/tiler-compiler/src/session.rs (pub const FLUSH_SUBNORMALS_TO_ZERO_F32)
  - checks:
    - rg 'evaluates end to end at the C1 row' tickets/assemble-the-causal-self-attention-block-program.md → hit
    - rg 'pub const FLUSH_SUBNORMALS_TO_ZERO_F32' crates/tiler-compiler/src/session.rs → hit
    - rg attention|causal_self under prototypes/ → no matches (device path still absent)
    - shasum -a 256 tickets/integrate-the-attention-block-into-the-runtime.md → d3de95eef1a6049b440d86e752964bccbd300b99eba039bd2dd09310e371ccc2

Recommended next ledger state:
  integrated
