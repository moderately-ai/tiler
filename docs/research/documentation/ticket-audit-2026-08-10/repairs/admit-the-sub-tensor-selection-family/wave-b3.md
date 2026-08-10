Ticket: admit-the-sub-tensor-selection-family
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-the-sub-tensor-selection-family/762bd092af43_c99ac54950f2.md
Pre-edit content hash (from ledger): 762bd092af4305301f9c9934f6ee7b21ea9b40031ed158fa47e3e1ac69409754
Post-edit content hash: 2375baf7562baa7f2cc333f8760b8783bac24e2ec4e544edde8fc106bcb96844

Changes applied:
  - Prefaced "Why this is its own ticket" with **Correction — 2026-08-10** stating filing-time Facts are not live; live matrix is R5 for F32 literal-offset; IndexNode half retired under 2026-08-09 correction.
  - Struck first Fact (matrix R1 / no key) as pre-delivery problem statement; reindex refusal remains live.
  - Struck third Fact (IndexNode cannot express `t + C`) as pre-delivery problem statement; forward-referenced 2026-08-09 correction and semantic `symbolic-window` refusal / decide-the-source-bearing-slice-offset-boundary.
  - Outcome: replaced "Nine refusals" with twelve `SliceSelectionError::diagnostic_code` arms (eleven named + `result-shape`), noting schema/type/operands codes outside the enum (**Correction — 2026-08-10**).
  - Outcome support-matrix paragraph: kept landing R1→R4 as close-day delivery; **Correction — 2026-08-10** that fusion-role ticket moved live cell to R5.
  - Outcome compiler-seating paragraph: past-tense at close; **Correction — 2026-08-10** that `UNPLANNED_OPERATIONS` and `CoordinateRelation` fusion role have landed; absolute explain pin not pinned from this ticket.
  - Metadata unchanged (status done, empty dependencies, related list coherent).

Optional items skipped (with reason):
  - Separate top-level `## Fact audit — 2026-08-10` block for items 2–3: applied as inline **Correction — 2026-08-10** markers next to the false counts/claims instead; 2026-08-09 IndexNode correction left intact and not restated as unfixed.

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report Exact files: ticket prose only. No docs/crates edits. No new remainder tickets; symbolic boundary already owned by decide-the-source-bearing-slice-offset-boundary.

Verification:
  - files read: full audit report; full ticket (pre/post); `SliceSelectionError::diagnostic_code` arms in crates/tiler-ir/src/semantic/slice.rs (twelve codes); docs/roadmap.md Sub-tensor selection row (R5); crates/tiler-compiler policy UNPLANNED + fusion_legality CoordinateRelation; LinearTermData.coefficient SourcedIndexInteger; admit-a-fusion-role-for-the-sub-tensor-selection-slice status done.
  - checks: shasum -a 256 of ticket post-edit; count of diagnostic_code match arms = 12 including result-shape; matrix and seating greps.

Recommended next ledger state:
  integrated
