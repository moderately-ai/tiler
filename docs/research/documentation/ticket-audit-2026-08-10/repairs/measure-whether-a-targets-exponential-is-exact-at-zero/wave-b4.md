Ticket: measure-whether-a-targets-exponential-is-exact-at-zero
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/measure-whether-a-targets-exponential-is-exact-at-zero/95f1d229eb58_c99ac54950f2.md
Pre-edit content hash (from ledger): 95f1d229eb5883946e3374bd0d5fac5997242928b9fde541dc29a5f9e14c5414
Post-edit content hash: e9105202a8c6690b3061282e2bfe07c5b411a6b8e400a97187ed263e539332a1

Changes applied:
  - Kept `dependencies: []` (audit option b); rewrote 2026-08-09 trigger log from "The named dependency is `done`" to "The related ticket `expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate` is `done`" so graph vocabulary matches frontmatter (`related` only).
  - Rewrote Why-this-exists **Inference** to drop the imprecise "roughly half of the elementary evaluations and half the rescale roundings" slogan: exactness removes charges only on path steps whose rescale argument is zero; worst-case path under strict max jumps may drop far less than half of path-based `E = 1 + D`; worker must re-derive any sharpened form; kept conservative first-order price `D * (u + eps_exp)`.

Optional items skipped (with reason):
  - Optional dated correction block for the half-`E` gloss: skipped because the Inference was rewritten cleanly in place on a `todo` ticket with no landed Outcome to preserve (audit preferred path).
  - Optional hard `dependencies` edge to expose: skipped; audit preferred (b) and empty dependencies remain consistent with a self-contained post-trigger measurement.

Residuals not applied (docs/crates/new tickets/authority):
  - Tree-fold open axis and "Experiment remaining" still say this ticket is deferred / unfired on `eps_exp` retrieval (research-record drift under `research/numerics`); docs edit out of wave B1 scope.
  - Product measurement work itself (device bits, bound sharpening) remains open `todo` — not this repair.

Verification:
  - files read:
    - tickets/measure-whether-a-targets-exponential-is-exact-at-zero.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/measure-whether-a-targets-exponential-is-exact-at-zero/95f1d229eb58_c99ac54950f2.md (full)
    - tickets/expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md (frontmatter: status done)
    - docs/research/numerics/tree-fold-online-softmax-bound.md (anchors: path `E = 1 + D`, first-order `D * (u + eps_exp)`, open axis "roughly half of `E`")
  - checks:
    - expose status: done; this ticket related lists expose; dependencies remain []
    - post-edit sha256: e9105202a8c6690b3061282e2bfe07c5b411a6b8e400a97187ed263e539332a1

Recommended next ledger state:
  integrated
