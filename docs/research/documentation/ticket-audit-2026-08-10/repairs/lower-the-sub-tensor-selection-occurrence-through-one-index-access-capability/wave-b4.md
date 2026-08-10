Ticket: lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability/9fd1f7eaf800_c99ac54950f2.md
Pre-edit content hash (from ledger): 9fd1f7eaf80034ea725dd8d13c8adf7fe3bce03351783d5a15f4ea7c9a159c54
Post-edit content hash: 1b46a763ebef204a7142ec61611e891c4819a16e3b52bb6eea4975b4d1f309b2

Changes applied:
  - related: removed duplicate `admit-the-sub-tensor-selection-family` (kept only under dependencies).
  - User-visible outcome: narrowed "no owner named anywhere in the corpus" to the delivery-graph bare M5 cell / no ticket link in that document.
  - Why Fact 1: restated so delivery-graph O-06 M5 and owners bullet still lack an owner link / still read bare *owed*, while the support-matrix Sub-tensor selection row already names this ticket as the M5 owner between R5 and R6.
  - Optional adjacent note: delivery-graph O-06 M4 still reads *owed* while matrix R5 and the fusion-role ticket are done (out of close condition).
  - Explicit non-goals second bullet: replaced reindex/broadcast "delivered and never resolved" peer claim; stated those families resolve and compile via `LogicalAccess::ReindexBijection` / `BroadcastReplication`; residual for slice is missing selection/window LogicalAccess with **no live ticket owner** (concat has `admit-the-concatenate-family-into-the-scheduled-region-vocabulary`; slice has no counterpart); kept non-goal that this ticket does not lift the request boundary or claim a VerifiedKernel.

Optional items skipped (with reason):
  - none (optional O-06 M4 lag note applied as cheap adjacent hygiene on the same O-06 framing).

Residuals not applied (docs/crates/new tickets/authority):
  - No new remainder ticket filed for "admit a LogicalAccess (or equivalent) spelling for sub-tensor selection / window maps so a slice program clears `operation-set`" — report requires file-or-link; no concrete id was supplied and wave B forbids inventing ticket ids. Ticket body now states the gap as unowned rather than assigning it to done structural-families.
  - Product implementation still open: capability + provider in `crates/tiler-compiler/src/governed.rs`, offset-drop perturbation tests, delivery-graph O-06 M5 cell + owners update in `docs/research/semantic-graph/operation-family-delivery-graph.md` (Exact files; not wave-B scope).

Verification:
  - files read:
    - full audit report `9fd1f7eaf800_c99ac54950f2.md`
    - full ticket (pre/post)
    - `docs/research/semantic-graph/operation-family-delivery-graph.md` O-06 semantic/physical rows + owners bullet `live for the literal-offset form`
    - `docs/roadmap.md` Sub-tensor selection row (names this ticket for M5; R5 for F32 literal-offset)
    - `crates/tiler-compiler/src/governed.rs` (no slice/GovernedSlice; reindex/broadcast providers present)
    - `crates/tiler-ir/src/schedule/model.rs` (`ReindexBijection`, `BroadcastReplication`; no window/selection map)
    - `tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md` status done
    - `tickets/admit-the-concatenate-family-into-the-scheduled-region-vocabulary.md` (parallel remainder pattern; slice excluded explicitly)
    - ticket inventory under `tickets/*slice*` / `*sub-tensor*` (no slice scheduled-region admit ticket)
  - checks:
    - `related` no longer lists family-admission id
    - Why/non-goals no longer claim corpus-wide owner absence or structural-families ownership of slice request-boundary residual
    - status left `todo` (M5 capability still undelivered)

Recommended next ledger state:
  integrated
