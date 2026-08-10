Ticket: decide-whether-a-contraction-may-consume-more-than-two-operands
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-a-contraction-may-consume-more-than-two-operands/5f49a14adc00_c99ac54950f2.md
Pre-edit content hash (from ledger): 5f49a14adc00eef3333acce7c6276febf238a3df7b727de4bca1d46e6c328fcd
Post-edit content hash: dcc828ec359f5bcbe2fd78212bc9cf4ecfed57f85dea6f2b4330854a46795fc6

Changes applied:
  - Replaced stale line-number Fact citations with searchable anchors: roadmap reserved item 2 (`Whether a semantic contraction node may consume more than two operands` under `#### Decisions reserved for Tom`); structural rule (`for a multi-operand form, an index appearing in more than two operands`); Q-SEM-015 (`Still reserved from the framing, and the only one of the three left`); binary admission via `normalize_contraction` / exactly-two-operands guards in `crates/tiler-compiler/src/request.rs` plus support-matrix `R6 for a whole-program contraction occurrence` (dropped `docs/roadmap.md:421`); diagnostic code `contraction.rule.index-in-more-than-two-operands` without bare `:336`.
  - Updated independence quote to match live Q-SEM-015 wording (`and therefore ADR 0095's decline`).
  - Dropped bare sibling `:19` location; quote remains the locator.
  - Added 2026-08-10 trigger-check log entry (**not fired**), reconfirming refusal-only diagnostic, binary `normalize_contraction`, and no three-or-more shared-index workload.

Optional items skipped (with reason):
  - none (optional trigger-log refresh applied as cheap hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required no docs/crates edits, no new remainder tickets, no status/metadata change

Verification:
  - files read:
    - audit report `5f49a14adc00_c99ac54950f2.md`
    - full ticket pre-edit
    - `rg` anchors in `docs/roadmap.md`, `docs/open-questions.md`, `crates/tiler-ir/src/semantic/contraction.rs`, `crates/tiler-compiler/src/request.rs`
  - checks:
    - reserved multi-operand phrase present under Decisions reserved for Tom
    - structural multi-operand bullet and R6 matrix row present
    - Q-SEM-015 trigger still reserves multi-operand as only open of three
    - diagnostic `contraction.rule.index-in-more-than-two-operands` + refusal tests only
    - post-edit ticket has no `file:NNN` citations

Recommended next ledger state:
  integrated
