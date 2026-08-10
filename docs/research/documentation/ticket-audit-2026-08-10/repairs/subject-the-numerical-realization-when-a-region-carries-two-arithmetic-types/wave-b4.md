Ticket: subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types/072ecde48afb_c99ac54950f2.md
Pre-edit content hash (from ledger): 072ecde48afbfef36b5a4c5f0c964c554fc11f9b7ee0439e27e14bdef7fd0c61
Post-edit content hash: b897922337d91a1f8b069fd8926c390317cebb37b26278a61bd00145c5b35016

Changes applied:
  - Rewrote "Why it was declined" bullet 1: replaced stale "no caller / every construction site is strict() or new()" with current facts — non-test callers in tiler-conformance (bf16_vertical, publication proof), subject passed as ArithmeticType from region/witness, deliberately not stored on NumericalRealization.
  - Added **Correction — 2026-08-10.** under Why declined: 2026-08-07 "no caller" premise false after give-the-realization…; arm A stands; trigger unchanged.
  - Replaced bare line citations model.rs:1333 and builder.rs:664 with searchable anchors (`region_arithmetic_type`, `verify_pointwise_bf16` NaN gate + `verify_accumulation_width`).
  - Added related edge: give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.
  - Appended Trigger check log 2026-08-10 **not fired** with recheck command.
  - status left deferred; dependencies unchanged.

Optional items skipped (with reason):
  - Expanding scopes for armed work: report says expand when trigger fires, not now.

Residuals not applied (docs/crates/new tickets/authority):
  - none (repair was ticket-only; no new remainder tickets required).

Verification:
  - files read: ticket; audit report; crates/tiler-ir/src/schedule/model.rs (region_arithmetic_type); crates/tiler-reference/src/conformance.rs (from_realization subject-as-argument docs); grep from_realization under crates/ (bf16_vertical, publication/proof production sites).
  - checks: region_arithmetic_type still one-type exhaustive match; from_realization non-test callers present; subject argument docs match repair prose.

Recommended next ledger state:
  integrated
