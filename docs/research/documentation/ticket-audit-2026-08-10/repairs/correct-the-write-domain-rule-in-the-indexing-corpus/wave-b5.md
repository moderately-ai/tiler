Ticket: correct-the-write-domain-rule-in-the-indexing-corpus
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-write-domain-rule-in-the-indexing-corpus/15226ab71a9b_c99ac54950f2.md
Pre-edit content hash (from ledger): 15226ab71a9bcfc89665a9f3813084e51b031e029c8c618f4bbc138e1e21055c
Post-edit content hash: 381e02981e3d4717e0eebfb12e2543e35398b632c91275085d6a5bdd2acff0ab

Changes applied:
  - Added `## Fact audit — 2026-08-10` with **Correction — 2026-08-10.**: Why's present-tense equality-rule Facts marked historical filing premises (research list + open-questions already state subset rule); Outcome landing-base line pins dated as aged with durable symbol/phrase anchors; residual stale pins in research **Correction — 2026-08-06** named; status stays `done`; metadata/graph left unchanged (report: none required).

Optional items skipped (with reason):
  - In-place second dated note under research Correction — 2026-08-06 after pin refresh: optional if pins fixed in place; Class C wave is ticket-only, so docs pin refresh left residual rather than applied.
  - New remainder ticket for pin refresh: report says none required if accepted as tiny corpus repair; no new id filed.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/indexing/concatenate-fusion-role-and-lowering.md` **Correction — 2026-08-06**: replace stale current-state line pins (`builder.rs:1337-1343` write-domain, `builder.rs:1909-1927` multi-root output, `program/verify.rs:203` MultipleWriters) with durable anchors (`IndexRegionBuilder::write` / `prepare_access` Parallel-only / `InvalidWriteDomain`; `Several roots may name one output tensor`; `KernelProgramDiagnostic::MultipleWriters` at verify return site). Keep verified `proof.rs:329` partition dispatch and `proof.rs:271` owns_alone gate (re-verify if they move). Prefer phrase/symbol anchors over new hard lines. Class C default ticket-only; docs residual product debt.

Verification:
  - files read:
    - full audit report `15226ab71a9b_c99ac54950f2.md`
    - full ticket `tickets/correct-the-write-domain-rule-in-the-indexing-corpus.md` (pre/post)
    - `docs/research/indexing/concatenate-fusion-role-and-lowering.md` correction and refusal list
    - `docs/open-questions.md` Q-SHAPE-006 block (grep + pressure bullet)
    - `crates/tiler-ir/src/index/builder.rs` write doc / prepare_access InvalidWriteDomain / multi-root output doc
    - `crates/tiler-ir/src/index/builder/proof.rs` owns_alone, decide_partition_by_interval, dispatch
    - `crates/tiler-ir/src/program/verify.rs` MultipleWriters return
  - checks:
    - `rg -n 'InvalidWriteDomain' crates/tiler-ir/src/index/builder.rs` → doc + return at prepare_access Write branch (~1628)
    - `rg -n 'Several roots may name one output tensor' crates/tiler-ir/src/index/builder.rs` → ~2197
    - `rg -n 'fn decide_partition_by_interval|owns_alone' crates/tiler-ir/src/index/builder/proof.rs` → dispatch :329, owns_alone gate :271, fn :1193
    - `rg -n 'MultipleWriters' crates/tiler-ir/src/program/verify.rs` → return :209
    - `rg -n 'builder\.rs:1337-1343|builder\.rs:1909-1927' docs/research/indexing/concatenate-fusion-role-and-lowering.md` → still in Correction — 2026-08-06 (residual)
    - post-edit `shasum -a 256` ticket → 381e02981e3d4717e0eebfb12e2543e35398b632c91275085d6a5bdd2acff0ab

Recommended next ledger state:
  integrated
