Ticket: accept-the-multi-reader-index-realization-retention
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-multi-reader-index-realization-retention/6f32a9c5065b_c99ac54950f2.md
Pre-edit content hash (from ledger): 6f32a9c5065b4d8205239bd6e05ca5ee9696ded374f845df106378b9bf2a22dd
Post-edit content hash: 7990a9d1f31c261fa8e86ad92ff18d77a65569f23de2522c46f5c45c38a011cc

Changes applied:
  - Rephrased the `retained_through()` surface bullet: one-reader equality remains; the population clause is historically scoped to acceptance (2026-08-06) instead of present-tense "today", with an explicit note that registered laws may since emit multi-reader chains (softmax staging example).
  - Appended **Correction — 2026-08-10.** stating the retired present-tense population claim is false once a multi-reader law is registered and that only that claim, not the equality rule, was wrong.

Optional items skipped (with reason):
  - none — the report's optional dated correction was applied (cheap house-style hygiene on this same ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required only ticket prose; metadata was already coherent; no remainder filing or docs/crates edits.

Verification:
  - files read:
    - tickets/accept-the-multi-reader-index-realization-retention.md (full, before and after)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-multi-reader-index-realization-retention/6f32a9c5065b_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/law.rs — softmax pin tuples `(1, 2, 3, ...)` / `(1, 3, 3, ...)` confirm multi-reader retained_through past consumer for producer 1
    - crates/tiler-ir/src/index/sequence.rs — retained_through field/accessors present
  - checks:
    - present-tense phrase `every record any registered law produces today` no longer stands as a live Fact in the ticket
    - equality rule for one-reader records left intact
    - status/deps/related/scopes unchanged (report: metadata none)
    - post-edit sha256 via `shasum -a 256` on the ticket path

Recommended next ledger state:
  integrated
