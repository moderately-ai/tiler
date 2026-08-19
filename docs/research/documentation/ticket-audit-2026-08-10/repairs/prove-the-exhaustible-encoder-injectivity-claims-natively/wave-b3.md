Ticket: prove-the-exhaustible-encoder-injectivity-claims-natively
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/prove-the-exhaustible-encoder-injectivity-claims-natively/e744edd1ceef_c99ac54950f2.md
Pre-edit content hash (from ledger): e744edd1ceefef3cd9acd906e65ab8b0585175e5982a613b265256517498ac90
Post-edit content hash: 79e10cd9d7b85b4833f3624b8c4c317b104567e54d0c12ec42c43b84795cf1b3

Changes applied:
  - Outcome completeness: added ## Fact audit — 2026-08-10 naming kernel `push_index_arithmetic` and artifact `index_arithmetic_tag`/`from_tag` as size-1 exhaustible remainder omitted from the nineteen and from `every_governed_tag_table_round_trips`; withdrew silent completeness; kept status `done`.
  - Moved kernel `push_requirements` out of **Slices and vectors** into a fixed-width ordinals + finite tail bullet alongside almost-exhaustible `push_resources` / `push_numerical` discussion.
  - Replaced stale `codec/tests.rs`:541 and fixed "seven" with searchable anchor `fn every_governed_tag_table_round_trips` and accurate table list (notes index-arithmetic tables are outside that test).
  - Optional dated correction included in the same Fact audit block (IndexArithmetic parallel to ContributorOrder; artifact tags outside both nineteen and left-inverse walk).

Optional items skipped (with reason):
  - none (dated correction applied; no optional graph edge without a remainder ticket id)

Residuals not applied (docs/crates/new tickets/authority):
  - No new remainder ticket filed (would need a concrete ticket id decision). Residual owed work: one-encoder injectivity for kernel `push_index_arithmetic` and artifact `index_arithmetic_tag`/`from_tag` with `variant_count`-sized population pin (literal 1), either as a narrow related ticket or absorbed into `prove-the-governed-tag-tables-injective` owed-set classification.
  - Crates not edited (wave ticket-only): optional injectivity tests in `crates/tiler-ir/src/kernel/model.rs` and artifact index-arithmetic left-inverse/injectivity coverage.
  - Full independent census of every other `push_*`/`encode*` exhaustible leaf in index/semantic remains residual uncertainty from the audit (only IndexArithmetic confirmed miss of tested class).

Verification:
  - files read:
    - audit report e744edd1ceef_c99ac54950f2.md in full
    - tickets/prove-the-exhaustible-encoder-injectivity-claims-natively.md in full (pre-edit)
    - crates/tiler-ir/src/kernel/model.rs (push_index_arithmetic, push_requirements windows)
    - grep: IndexArithmetic / push_index_arithmetic / every_governed_tag_table_round_trips / index_arithmetic_tag under crates/
  - checks:
    - `push_index_arithmetic` present at kernel model; no injectivity hit for index_arithmetic under exhaustive helpers
    - `fn every_governed_tag_table_round_trips` present (codec/tests.rs); not line 541
    - `index_arithmetic_tag` / `from_tag` present on artifact program model
    - post-edit sha256: 79e10cd9d7b85b4833f3624b8c4c317b104567e54d0c12ec42c43b84795cf1b3

Recommended next ledger state:
  integrated
