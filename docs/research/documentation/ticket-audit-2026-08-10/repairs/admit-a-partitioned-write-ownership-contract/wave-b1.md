Ticket: admit-a-partitioned-write-ownership-contract
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-partitioned-write-ownership-contract/b3f66015ae3c_c99ac54950f2.md
Pre-edit content hash (from ledger): b3f66015ae3c9047f1d01972345b018010a98cb2972b03632bc71b5bf53ad26c
Post-edit content hash: 6946b266f1837f9ac94222185654b1b93f163c534039286bd7bf118e984d90ba

Changes applied:
  - Outcome Fact 2 equal-share Inference: added **Correction — 2026-08-10.** marking the equal-share / extent-3+5-unspellable claim as historical at land; notes `admit-sub-range-write-domains-for-unequal-partitions` delivered subset-of-parallel domains (`status: done`) and unequal partitions (test `unequally_sized_contiguous_partitions_are_admitted_by_interval_reasoning`); preserves domain-vs-coverage reading of `InvalidWriteDomain` (now reduction-only).
  - Public boundary section: added **Correction — 2026-08-10.** marking "draft, for Tom" / "Not self-accepted" as historical; points at `accept-the-partitioned-write-ownership-proof-boundary` (`status: done`, Tom accepted all four parts on 2026-08-06).

Optional items skipped (with reason):
  - none (report listed no optional bullets; metadata already coherent).

Residuals not applied (docs/crates/new tickets/authority):
  - Sequence-extending research table still states two proof forms only — corpus drift outside this ticket (report residual uncertainty; not owned here).
  - Stale line numbers in Outcome site citations left as historical anchors (report residual; anchors used in audit).

Verification:
  - files read:
    - tickets/admit-a-partitioned-write-ownership-contract.md (full, pre- and post-edit)
    - audit report b3f66015ae3c_c99ac54950f2.md (full)
    - tickets/admit-sub-range-write-domains-for-unequal-partitions.md (status: done)
    - tickets/accept-the-partitioned-write-ownership-proof-boundary.md (status: done; Accepted by Tom on 2026-08-06)
    - crates/tiler-ir/tests/index_region.rs (presence of unequally_sized_contiguous_partitions_are_admitted_by_interval_reasoning)
  - checks:
    - shasum -a 256 tickets/admit-a-partitioned-write-ownership-contract.md → 6946b266f1837f9ac94222185654b1b93f163c534039286bd7bf118e984d90ba
    - two Correction — 2026-08-10 anchors present (equal-share; public boundary)

Recommended next ledger state:
  integrated
