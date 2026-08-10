Ticket: admit-lane-typed-values-and-masked-memory-into-the-kernel-ir
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-lane-typed-values-and-masked-memory-into-the-kernel-ir/bad7c5384741_c99ac54950f2.md
Pre-edit content hash (from ledger): bad7c5384741e02e8d300ecc3d0995b0d467067eee3c783870bbec15a77c7a3e
Post-edit content hash: 86a0a46380c7490ea9058bf896dfbd506a9911cde985ac68789ec5d7c8d1686f

Changes applied:
  - Kept `status: blocked` (schedule-vocabulary dependency still `awaiting-decision`).
  - Added tags `decision` and `needs-tom` (match schedule and target-profile siblings).
  - Expanded `scopes` with `implementation/artifact`, `implementation/metal`, and `contracts/decisions` for total-map / refusal-arm fanout and in-ticket Decision packet authority.
  - Added `related: admit-subgroup-typed-values-and-collectives-into-the-kernel-ir` (lane-identity coordination reverse edge; optional hygiene).
  - Replaced false Implementation key that claimed both `KernelType` and `Builtin` are not `#[non_exhaustive]` with the verified split: `KernelType`/`UnaryOp` total maps vs `Builtin` `#[non_exhaustive]` additive landing.
  - Added Implementation keys for append-only identity encoding discipline and in-scope mechanical artifact/metal match/refusal arms (emission of lane programs still non-goal).
  - Added `## Decision packet — 2026-08-10` with recommended public shapes under ADR 0075 and dated Builtin fact repair.
  - Added `## Board release path`: on schedule dependency completion, release to `awaiting-decision` not `todo`/`ready`.

Optional items skipped (with reason):
  - none (optional related reverse edge applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation of lane/mask types, masked memory, lane-index builtin, and arithmetic widening remains future work on this ticket after ADR 0075 acceptance (Exact files under crates/ listed by the audit are not wave-B edits).
  - No new remainder ticket filed (report: none required if scopes and Decision packet repaired here).

Verification:
  - files read:
    - full audit report `bad7c5384741_c99ac54950f2.md`
    - full ticket pre-edit
    - `crates/tiler-ir/src/kernel/model.rs` (KernelType not non_exhaustive; Builtin is)
    - sibling Decision packets: `admit-vector-lane-bindings-into-the-schedule-vocabulary.md`, `declare-cpu-vector-realization-facts-in-the-target-profile.md`
    - subgroup ticket related edge confirming reverse coordination
  - checks:
    - pre-edit sha256 == bad7c5384741e02e8d300ecc3d0995b0d467067eee3c783870bbec15a77c7a3e
    - post-edit sha256 == 86a0a46380c7490ea9058bf896dfbd506a9911cde985ac68789ec5d7c8d1686f
    - dependency still `awaiting-decision`; this ticket still `blocked`
    - `rg` confirms Builtin `#[non_exhaustive]` at model.rs

Recommended next ledger state:
  integrated
