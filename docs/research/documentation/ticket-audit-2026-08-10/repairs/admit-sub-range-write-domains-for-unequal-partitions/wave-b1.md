Ticket: admit-sub-range-write-domains-for-unequal-partitions
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-sub-range-write-domains-for-unequal-partitions/687b89a95f8d_c99ac54950f2.md
Pre-edit content hash (from ledger): 687b89a95f8ddc20824ce2bcaa9e233a98092ae16eafe4396f751f933902b408
Post-edit content hash: f524824db2cfda00a242e5f341e6d33d2ea1960b73cff9f4d357ccbb10720a20

Changes applied:
  - Why-this-exists: replaced stale live line cite `builder.rs:1308-1310` with searchable anchor on `prepare_access` / `role != DomainRole::Parallel` → `InvalidWriteDomain`; marked full-parallel equality enforcement as historical (landing-time) vs live reduction-role refuse.
  - What-the-work-is: struck live `oracle.rs:1420-1427` equality premise cite; replaced with filing-time history plus current root-point / `output_plans` subset-domain anchors (`DomainWalk::new(self.domain(access.domain())?)`).
  - Added `## Fact audit — 2026-08-10` with dated correction: Outcome "exactly one `.domain()` outside index/" was landing-tree blast radius only; live oracle `output_plans` consumes write-root domains.
  - Same section: dated correction that interim oracle refuse-then-support completed (both filed oracle tickets `done`); Outcome oracle-boundary prose is landing-time history.

Optional items skipped (with reason):
  - Title clarification (sub-range → subset-domain): report labeled optional; not load-bearing for status/deps/scopes; Outcome already states subset admitted and sub-range annotation eliminated.

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required ticket-only prose/metadata fixes; no docs/crates edits; no new remainder.

Verification:
  - files read:
      - tickets/admit-sub-range-write-domains-for-unequal-partitions.md (full, pre/post)
      - report 687b89a95f8d_c99ac54950f2.md (full)
      - crates/tiler-ir/src/index/builder.rs (InvalidWriteDomain / prepare_access role check)
      - crates/tiler-reference/src/oracle.rs (root-point docs; output_plans; DomainWalk)
      - grep `.domain()` under crates/**/*.rs
  - checks:
      - stale cites `builder.rs:1308` and `oracle.rs:1420` absent from ticket after edit
      - sha256 post-edit: f524824db2cfda00a242e5f341e6d33d2ea1960b73cff9f4d357ccbb10720a20

Recommended next ledger state:
  integrated
