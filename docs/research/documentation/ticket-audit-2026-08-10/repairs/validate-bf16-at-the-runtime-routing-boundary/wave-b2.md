Ticket: validate-bf16-at-the-runtime-routing-boundary
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/validate-bf16-at-the-runtime-routing-boundary/5df5f7ad0602_c99ac54950f2.md
Pre-edit content hash (from ledger): 5df5f7ad0602fb4292435636b24277fbf209b06dc53721946d5f375c0572beb8
Post-edit content hash: 6d474a9e8879650899e35b96bf897abaa64432bc4acea83686a4fac18f5e1243

Changes applied:
  - Graph maintenance: historized present-tense claim that `docs/dtype-support.md` records Runtime semantic validation as `absent/unsupported` for f32; **Correction — 2026-08-10** that `move-the-runtime-semantic-validation-cells-for-f32-and-bf16` moved both f32 and BF16 cells to `tested guarantee, dtype refusal at the routing boundary only`.
  - Outcome "did not discharge" dtype-support bullet: past-tense "was not edited by this ticket" / "was to move"; same dated correction pointing at Graph maintenance and the move ticket.
  - Why-phase Measurement: Finding 26 named as pipeline creation + `XPC_ERROR_CONNECTION_INTERRUPTED`; `PreparedKernelPreflight` attributed as this workspace's phase-vocabulary mapping of post-ADR-0051-commit failure, not a Finding 26 string.
  - Outcome host-earned bullet: landing-time tautology preserved; **Correction — 2026-08-10** that declare-host later emitted/read rows but left producer-declared equality on facade/Candle; "does not exist" still names host-earned eligibility, not a missing declare-host landing.
  - Metadata left unchanged (status, dependencies, related, scopes) per report.

Optional items skipped (with reason):
  - none (both optional clarity bullets applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required. Report Exact files listed ticket only; ordinal-permutation gap is documented evidence boundary, not a missing product ticket; host-earned observation remains owned by declare-host / adapter / ADR 0086.

Verification:
  - files read:
    - tickets/validate-bf16-at-the-runtime-routing-boundary.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/validate-bf16-at-the-runtime-routing-boundary/5df5f7ad0602_c99ac54950f2.md (full)
    - docs/dtype-support.md (Runtime semantic validation matrix rows + f32/BF16 cell prose via grep)
    - docs/research/apple-targets/numerical-behaviour.md (Finding 26 paragraph via grep)
    - tickets/move-the-runtime-semantic-validation-cells-for-f32-and-bf16.md (status)
    - tickets/declare-host-dtype-dispatchability-at-the-consumer-boundary.md (status + Outcome host-earned / emission claims)
  - checks:
    - matrix cells: IEEE f32 and BF16 Runtime semantic validation both `tested guarantee, dtype refusal at the routing boundary only`
    - Finding 26: `fails pipeline creation with \`XPC_ERROR_CONNECTION_INTERRUPTED\``; no `PreparedKernelPreflight` in numerical-behaviour.md
    - move-cells and declare-host both `status: done`
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
