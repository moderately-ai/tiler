Ticket: implement-first-quantized-backend-profile
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/implement-first-quantized-backend-profile/b7944a8c17b9_c99ac54950f2.md
Pre-edit content hash (from ledger): b7944a8c17b9ff914de658487b3621383172a034ba9c0a18598908a6d6137936
Post-edit content hash: 954b580a491fd17bf32dcfd817dfedf1dede62c3c6f7b99846defb4315d64823

Changes applied:
  - Removed `implement-workload-selected-quantized-parameter-maps` and `admit-a-caller-declared-target-profile` from `related` (both remain in `dependencies` only).
  - Populated `paths` with selection/corpus anchors: `docs/research/numerics/first-quantized-lm-profile.md`, `spikes/numerics/qwen3-weight-quantization-profiles/`, and the E-1 results directory `spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/` (all verified present on disk).
  - Rewrote opening activation gate from future-tense "Activate only after…" / "Before activation, revise…" into historical/consumed form stating selection landed 2026-07-31 and this ticket implements that fixed profile.
  - Graph-maintenance "All six are edges" bullet now names `widen-the-physical-vocabulary-for-per-axis-quantized-component-access` so the enumeration matches the count (six structural deps).
  - Added dated `## Fact audit — 2026-08-10` / `**Correction — 2026-08-10.**` (audit base `c99ac54950f2`) covering selection-consumed residual template, six-count repair, and related de-duplication.
  - Status left `todo` (product work undelivered; graph not ready).

Optional items skipped (with reason):
  - none (optional `paths` population applied as cheap graph hygiene once activation is treated as complete).

Residuals not applied (docs/crates/new tickets/authority):
  - Eventual product implementation (compiler/metal/artifact/runtime/reference vertical, dtype-ledger cells, pins) — out of wave B scope; remains open work on this ticket.
  - Parameter-map public constructor decision stays on `implement-workload-selected-quantized-parameter-maps` (Tom); not forked here.
  - Transitive deferred `realize-the-tiled-contraction-schedule-and-its-metal-emission` via fuse left as fuse ownership; no second fuse path filed.
  - Residual uncertainty on whether a non-tiled materializing path could stage without realize-tiled is fuse/selection ownership, not repaired here.

Verification:
  - files read:
    - full audit report `docs/research/documentation/ticket-audit-2026-08-10/reports/implement-first-quantized-backend-profile/b7944a8c17b9_c99ac54950f2.md`
    - full ticket pre/post edit `tickets/implement-first-quantized-backend-profile.md`
    - dependency `status:` lines for all ten hard deps (confirmed open chain: parameter-maps awaiting-decision; widen/fuse/runtime todo; six others done)
    - path anchors listed above exist on disk
  - checks:
    - `shasum -a 256 tickets/implement-first-quantized-backend-profile.md` → `954b580a491fd17bf32dcfd817dfedf1dede62c3c6f7b99846defb4315d64823`
    - `related` has no ids shared with `dependencies`
    - graph bullet names six backticked structural tickets including widen
    - opening no longer contains "Activate only after" / "Before activation, revise"

Recommended next ledger state:
  integrated
