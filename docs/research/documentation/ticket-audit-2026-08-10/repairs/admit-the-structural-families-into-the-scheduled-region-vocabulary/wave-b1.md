Ticket: admit-the-structural-families-into-the-scheduled-region-vocabulary
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-the-structural-families-into-the-scheduled-region-vocabulary/787723541879_c99ac54950f2.md
Pre-edit content hash (from ledger): 787723541879544bb9d9474c4552f82f1dfaee65ce93e43da9564478d02e15ec
Post-edit content hash: 6d65f97da94d27f97e62982d2d0e0925fc9af22af0a6f7ee721c26e2f92f1285

Changes applied:
  - Why section: added **Correction — 2026-08-10** that the Facts/Inference are filing-time problem statement only; live tree carries `ReindexBijection` and `BroadcastReplication`; struck both present-tense Facts and the Inference as live claims; past-tensed the retained problem prose; struck the obsolete absence-reproduce line.
  - Outcome public-surface line: struck `awaiting-decision` pointer; dated correction that `accept-the-structural-region-access-vocabulary` is `status: done` / accepted 2026-08-06.
  - Optional precision applied: Outcome kernel Fact now names `tiler.kernel.v6` as domain at IndexSubtract landing; dated correction that current domain is `tiler.kernel.v7` with tag `0x0c` unchanged.

Optional items skipped (with reason):
  - none (the sole optional precision bullet was applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report for correctness; named residual behaviors (`access_domain_shape` None; fail-closed fusion over structural reads) remain intentional product posture, not ticket prose debt.

Verification:
  - files read:
    - tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-the-structural-families-into-the-scheduled-region-vocabulary/787723541879_c99ac54950f2.md (full)
    - tickets/accept-the-structural-region-access-vocabulary.md (status: done; ## Decided — accepted)
    - crates/tiler-ir/src/schedule/model.rs (grep: LogicalAccess, ReindexBijection, BroadcastReplication)
    - crates/tiler-ir/src/kernel/model.rs (grep: KERNEL_DOMAIN = tiler.kernel.v7; IndexSubtract => 0x0c)
  - checks:
    - rg confirms ReindexBijection/BroadcastReplication present in schedule/model.rs
    - rg confirms KERNEL_DOMAIN is tiler.kernel.v7 and IndexSubtract tag 0x0c
    - accept ticket frontmatter status: done; Decided — accepted heading present
    - shasum -a 256 of ticket after edit → 6d65f97da94d27f97e62982d2d0e0925fc9af22af0a6f7ee721c26e2f92f1285

Recommended next ledger state:
  integrated
