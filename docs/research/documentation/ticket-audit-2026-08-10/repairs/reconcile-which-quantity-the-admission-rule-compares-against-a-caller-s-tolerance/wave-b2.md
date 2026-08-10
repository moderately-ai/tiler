Ticket: reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance/e7152def2e26_c99ac54950f2.md
Pre-edit content hash (from ledger): e7152def2e26489fac611736f56ceb48335cf8a1d122615583a3a5f340bca323
Post-edit content hash: 41077b85383d39ee79cca9967a95bf1f65a5d3c7c4ed1674720fcd91c445c29e

Changes applied:
  - Outcome "Parked for Tom…" live board-status label: `awaiting-decision` → `deferred` for `accept-the-rewrite-price-tolerance-vocabulary`, matching that ticket's frontmatter and body (`It is deferred, not awaiting-decision`).
  - Dated correction after that sentence recording the prior mislabel and pointing at the authoritative `deferred` state.
  - Optional graph hygiene: added `accept-the-rewrite-price-tolerance-vocabulary` and `derive-how-rewrite-price-budgets-compose-across-a-program` to `related`.
  - Optional dated line-number correction for `docs/numerical-semantics.md` `tolerance` hits and supporting sentences (`:397`/`:416`/`:426`/`:816`/`:1003`; support at `:420`/`:430`); landing-time numbers left in place as history.

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none (report listed no docs/crates remainder; composition, acceptance, absolute-delegation, and flash-class sharpening already filed or in-record)

Verification:
  - files read:
    - audit report e7152def2e26_c99ac54950f2.md
    - tickets/reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance.md (pre/post)
    - tickets/accept-the-rewrite-price-tolerance-vocabulary.md (status: deferred; body defers vs awaiting-decision)
    - docs/numerical-semantics.md (rg -n tolerance + support anchors)
  - checks:
    - accept ticket `^status: deferred` confirmed
    - tolerance lines 397, 416, 426, 816, 1003; support at 420, 430
    - post-edit sha256 via shasum -a 256
    - live Parked sentence carries `filed deferred`; only historical mislabel remains inside the dated correction

Recommended next ledger state:
  integrated
