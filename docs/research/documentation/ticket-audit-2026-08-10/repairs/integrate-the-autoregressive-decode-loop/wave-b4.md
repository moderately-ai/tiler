Ticket: integrate-the-autoregressive-decode-loop
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/integrate-the-autoregressive-decode-loop/edf1af67f7f6_c99ac54950f2.md
Pre-edit content hash (from ledger): edf1af67f7f6b639ce55c8a12e3067279b84f0d11573b92be41ebb136c4decf1
Post-edit content hash: 5632384a9b71257cbf3cca56d826f47b9ef061890b2a66114a2bfffe5608f239

Changes applied:
  - Failed-invocation Required behaviour bullet: split Tiler's stage-named typed failure (outputs withheld) from the driver's step attribution over a synchronous route result; replace fused "typed error naming the step" wording; state post-commit non-retry and no silent skip; state product rule that the loop stops on any refusal including pre-commit (routing-legal re-preflight not used here).
  - Added **Correction — 2026-08-10.** recording that prior step-naming fused Tiler stage failure with driver loop ordinal and is not a Tiler-public ordinal field.
  - Preserved 2026-08-04 poisoned-state correction under supersede-the-runtime-owned-kv-state-design.

Optional items skipped (with reason):
  - none (optional pre-commit product-rule note applied as required clarity for the residual "never retries" ambiguity)

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required; delivery still product work under scopes (consumer/driver, not this wave)

Verification:
  - files read: audit report; ticket; grep for "naming the stage" in L5 + adapter.rs (confirmed)
  - checks: ticket content hash post-edit via shasum -a 256; metadata left unchanged (status todo, deps, related, scopes)

Recommended next ledger state:
  integrated
