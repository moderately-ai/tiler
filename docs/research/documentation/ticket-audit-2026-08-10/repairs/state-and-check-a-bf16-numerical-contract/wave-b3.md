Ticket: state-and-check-a-bf16-numerical-contract
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/state-and-check-a-bf16-numerical-contract/9480cd301ba0_c99ac54950f2.md
Pre-edit content hash (from ledger): 9480cd301ba08f26544b6e562fb996da5540590d02b3f47725bb20c1a1b48668
Post-edit content hash: 5363ac43d0acf0a0ee2d90c0ee108ef9f2716378d7cd0f857dc5c0efd4424a58

Changes applied:
  - Under "## Why this is a separate boundary", pinned the unpinned "cannot ask" BF16 / only-`strict_f32` Fact as pre-landing at `aa09b5e` (past tense) and noted this ticket's delivery inverted it (width named; `arithmetic: ArithmeticType`; per-width resolution docs).
  - Under "## Graph maintenance", left the landing-time `dtype-f32` recognizer Fact in place and added a **Correction — 2026-08-10** that pure BF16 is recognized, a flush-accepting complete-table contract reaches a selected plan, the measured-ledger contraction/`Unknown` Fact remains live, and the admit-bf16 related edge is not a live post-feasibility wall.
  - Tightened "## User-visible outcome" flush-accepting clause to: clears the subnormal dimension; on the measured two-row BF16 ledger the next refusal is `Unknown` on contraction (no permanent dtype wall).
  - Metadata left unchanged (status done; dependencies/related correct).

Optional items skipped (with reason):
  - none (optional User-visible outcome tightening applied as cheap measurement-boundary hygiene on this same ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by audit (Exact files: ticket only; no new remainder; bind + navigation splits already closed).
  - Tom acceptance provenance remains session-witnessed only (report residual uncertainty; not a ticket-prose repair).

Verification:
  - files read:
    - tickets/state-and-check-a-bf16-numerical-contract.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/state-and-check-a-bf16-numerical-contract/9480cd301ba0_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/session.rs (STRICT_BF16 / strict_bf16 / "Every resolution is stated for exactly one")
    - crates/tiler-compiler/tests/bf16_numerical_contract.rs (module docs + `a_flush_accepting_bf16_contract_reaches_a_selected_plan`)
    - crates/tiler-ir/src/schedule/numerics.rs (BF16_NUMERICAL_CONTRACT_KEY_DOMAIN via grep)
  - checks:
    - re-verified live BF16 contract surface and inverted flush-accepting plan assertion against current tree before prose edit
    - post-edit sha256 of ticket file: 5363ac43d0acf0a0ee2d90c0ee108ef9f2716378d7cd0f857dc5c0efd4424a58

Recommended next ledger state:
  integrated
