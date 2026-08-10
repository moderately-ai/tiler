Ticket: execute-the-decode-step-path
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/execute-the-decode-step-path/ea805db8d296_c99ac54950f2.md
Pre-edit content hash (from ledger): ea805db8d296750afa10bc9488da2b7f23092a2fb72ada7216e6e7ebb2899f09
Post-edit content hash: 84c3958884c7b67100e2e4508dc3bb5f10a80988f5d6da54136a8e34134cc6b4

Changes applied:
  - Closes when narrowed to one-step C1 criteria only: step 1 at C=10,T=1,S=11 yields S=11 retained pair; post-commit dispatch failure leaves inputs bit-identical, no observable output, cursor at 10. Dropped "tiled variant is selected at S=16 and only there."
  - Submission-receipt / retention sentence separated from the landed plan_dispatch / allocate_dispatch split: ADR 0051 Unrealized (submission receipt, terminal-status observation, resource retention) is named as adapter obligation, not as types reconcile shipped.
  - Dated **Correction — 2026-08-10** under Closes when naming one-step vs S=16 scope and synthetic multi-variant / deferred-Metal ownership (bind-repeated, integrate-loop / prove-c1; not realize-the-tiled body).
  - Prose in touched sections unwrapped per house style; status, deps, scopes, related left unchanged (todo; evaluate retained; candle-only).

Optional items skipped (with reason):
  - Add bind-repeated-invocations-over-caller-retained-tensors to related — skipped because preferred close no longer cites multi-variant / S=16 selection; scope-correction prose already names bind as owner of generic multi-binding checks.

Residuals not applied (docs/crates/new tickets/authority):
  - Product delivery (decode-step driver, multi-binding device/context refusal, watched-failing tests) remains open under this ticket's todo outcome — not wave-B ticket repair.
  - prototypes/candle-metal-adapter and/or candle-scoped conformance driver changes listed in Exact files — out of scope for ticket-only wave.
  - No new remainder ticket filed; tiled multi-S evidence stays on bind-repeated / integrate-loop / prove-c1; deferred realize-the-tiled node already exists.

Verification:
  - files read: tickets/execute-the-decode-step-path.md (full pre-edit); report ea805db8d296_c99ac54950f2.md (full); docs/decisions/0051-make-runtime-routing-commit-one-way.md (Unrealized submission-receipt / resource-retention clauses); sample 2026-08-10 ticket correction style from other tickets
  - checks: pre-edit shasum matched ledger ea805db8d296750afa10bc9488da2b7f23092a2fb72ada7216e6e7ebb2899f09; post-edit shasum 84c3958884c7b67100e2e4508dc3bb5f10a80988f5d6da54136a8e34134cc6b4; no deps/status/scopes change required by report

Recommended next ledger state:
  integrated
