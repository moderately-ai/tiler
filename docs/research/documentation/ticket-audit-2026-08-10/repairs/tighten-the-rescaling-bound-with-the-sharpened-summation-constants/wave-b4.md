Ticket: tighten-the-rescaling-bound-with-the-sharpened-summation-constants
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/tighten-the-rescaling-bound-with-the-sharpened-summation-constants/2d3e0833f4d7_c99ac54950f2.md
Pre-edit content hash (from ledger): 2d3e0833f4d774cb753fcbd31af5548bb61dfa4bc91e3dd4b7785280e79d75e6
Post-edit content hash: b05cfe14a893dcd69c2dd6b32e38ad83bfb154c5063c8c22e79527e71ac62a9b

Changes applied:
  - Replaced false Measurement claim that sharpening is "worth roughly a factor of two in the first-order term" with the correct algebra: classical `gamma_h` and sharp `h*u` share first-order coefficients; sharpening drops only the `O((h*u)^2)` remainder and the `h*u < 1` side condition; relative classical-vs-sharp gap ~`h*u` (~3e-5 at binary32 `h = 511`) does not close the observed 20–600× looseness.
  - Added **Correction — 2026-08-10** documenting the prior false wording, the series expansion, and that Lange–Rump's factor-of-two is faithful-vs-RN, not classical-vs-sharp first-order.
  - Left status `deferred`, deps, related, scopes, tags, Trigger, and check log unchanged (report: metadata correct; no promote to todo).

Optional items skipped (with reason):
  - Optional dated reaffirmation of trigger not-fired at audit time: not required; 2026-08-05/09 entries remain accurate and report says no new check-log entry required beyond optional reaffirmation.

Residuals not applied (docs/crates/new tickets/authority):
  - none (report Exact files: ticket only; no docs/crates remainder; no new tickets).

Verification:
  - files read: full audit report; full ticket; certified-bounds anchors on `gamma` composition and "tighter constant … removing the second-order term"; house-style dated-correction samples in other tickets.
  - checks: post-edit `shasum -a 256` on ticket → `b05cfe14a893dcd69c2dd6b32e38ad83bfb154c5063c8c22e79527e71ac62a9b`; false "factor of two in the first-order term" claim removed from live Measurement; deferral still grounded on not closing observed looseness.

Recommended next ledger state:
  integrated
