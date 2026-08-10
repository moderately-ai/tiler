Ticket: evaluate-write-roots-over-their-own-domains-in-the-oracle
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/evaluate-write-roots-over-their-own-domains-in-the-oracle/fd7f014c316c_c99ac54950f2.md
Pre-edit content hash (from ledger): fd7f014c316cf64cbd36715d26509b86b285a3dce56d77e8e9e60212f8896e67
Post-edit content hash: c4c1814f1873bce3674351fcec8ee894ced5b5afbe7e1a44e4b72fa0dc49e5a4

Changes applied:
  - Outcome free_dimensions Fact: added **Correction — 2026-08-10.** replacing stale `builder.rs:1258` / `:1447-1458` / `:1706-1713` with fragment anchors (`domain.iter().map(|d| d.index).collect()` for reads; operand free flat_map + evaluation insert for apply; free.remove for reduce). Left `proof.rs:79` and used_parallel as valid.
  - Outcome permitted-divergence-oracle.md Fact: added **Correction — 2026-08-10.** marking oracle.rs absolute lines historical-to-landing; live doc cites `IndexRegionEvaluator::under` by name and `from_realization` via conformance.rs. OOS leave-alone scope retained.
  - Metadata left unchanged (status/dependencies/related/scopes already coherent per report).

Optional items skipped (with reason):
  - Why section present-tense ParallelWalk architecture: report marks non-blocking; Outcome remains authoritative; house style leaves Why as filing-time problem statement.

Residuals not applied (docs/crates/new tickets/authority):
  - none required for delivered Closes-when; draft public-surface Tom review already stated on ticket; no docs/crates edit in wave B3.

Verification:
  - files read:
    - tickets/evaluate-write-roots-over-their-own-domains-in-the-oracle.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/evaluate-write-roots-over-their-own-domains-in-the-oracle/fd7f014c316c_c99ac54950f2.md
    - crates/tiler-ir/src/index/builder.rs (free_dimensions read/apply/reduce sites)
    - crates/tiler-ir/src/index/builder/proof.rs (ValueDimensionOutsideWriteDomain; used_parallel)
    - docs/research/reference/permitted-divergence-oracle.md (index-region oracle table row)
  - checks:
    - rg free_dimensions / domain.iter().map in builder.rs → read collect at free set site; apply flat_map + insert; reduce free.remove
    - rg ValueDimensionOutsideWriteDomain|used_parallel in proof.rs → present
    - rg IndexRegionEvaluator::under|from_realization in permitted-divergence-oracle.md → name-only under cite; from_realization on conformance.rs
    - shasum -a 256 ticket → c4c1814f1873bce3674351fcec8ee894ced5b5afbe7e1a44e4b72fa0dc49e5a4

Recommended next ledger state:
  integrated
