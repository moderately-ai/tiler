Ticket: refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause/761eac4e887f_c99ac54950f2.md
Pre-edit content hash (from ledger): 761eac4e887f592e1cc38a9a81a8b89278a17102fd16b73e9f67a7e5dcd70456
Post-edit content hash: 38e15df885329123cfe48a26d80b74f7f32be21e917a22b16ca3afb0983ae56d

Changes applied:
  - Added `## Fact audit — 2026-08-10` / `**Correction — 2026-08-10.**` noting that Outcome "labelled draft" / accept parked language is historical after **Accepted — 2026-08-07** on `accept-the-softmax-realization-law` (`status: done`); live roadmap Softmax cell and `law.rs` Draft-boundary comments still present-tense; and that the R6-needs "each is its own ticket" claim asserted two walls without filing discoverable owner tickets. Frontmatter left `status: done` (no required metadata change).

Optional items skipped (with reason):
  - none — report's only optional ticket item was the dated correction; applied as cheap same-ticket hygiene.

Residuals not applied (docs/crates/new tickets/authority):
  - docs/roadmap.md — Softmax cell still present-tense labelled draft / accept parked; R6-needs still says each remaining wall "is its own ticket" without concrete ids; wave B ticket-only.
  - crates/tiler-ir/src/index/law.rs — `StagedSoftmaxF32` Draft boundary docs still claim labelled draft awaiting decision after acceptance.
  - File or connect two Softmax R6 implementation tickets (four-stage lowering provider; `physical::staged_plan` arm) and link them from the roadmap — report requires filing but assigns no concrete ids; blocked residual for coordinator id assignment.
  - docs/research/program-planning/first-metal-lm-workload.md Softmax row — still pre-law `operation-set` / two-prerequisite clause (out of original scopes; same defect class).
  - crates/tiler-compiler/tests/softmax_recognizer_boundary.rs module docs — multi-reader carry wall language stale; optional hygiene only.

Verification:
  - files read:
    - tickets/refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause.md (full, before and after)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-roadmap-softmax-cells-remaining-prerequisite-clause/761eac4e887f_c99ac54950f2.md (full)
    - tickets/accept-the-softmax-realization-law.md — `status: done`, **Accepted — 2026-08-07**
    - docs/roadmap.md Softmax cell — confirmed still "labelled draft" / parked and "which is why each is its own ticket"
    - crates/tiler-ir/src/index/law.rs — confirmed `labelled draft awaiting Tom's decision at` on StagedSoftmaxF32
  - checks:
    - accept ticket done + Accepted 2026-08-07 re-verified
    - ticket search for Softmax four-stage lowering / staged_plan owner remains empty of dedicated R6 wall tickets (per report)
    - frontmatter status/deps/related/scopes unchanged
    - post-edit sha256 via `shasum -a 256` on the ticket path

Recommended next ledger state:
  integrated
