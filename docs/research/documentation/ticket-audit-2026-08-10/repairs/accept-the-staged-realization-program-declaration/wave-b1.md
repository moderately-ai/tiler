Ticket: accept-the-staged-realization-program-declaration
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-staged-realization-program-declaration/d8ac062da905_c99ac54950f2.md
Pre-edit content hash (from ledger): d8ac062da90523c3a908bb203e20f1e25cfbb538cd759bb9ce5622926dc5c380
Post-edit content hash: f929fca4bd57e301dcc5d59b2f62184d3c3fe8ab0c9f453d1d314b676651d104

Changes applied:
  - Exact surface: rephrased KernelProgramBuildError packing to one new variant (`DuplicateStagedRealization`) and two reused (`SelfDependency`, `CoverageOutOfRange`).
  - Evidence: replaced "six tests" with "five tests" and enumerated the five named `#[test]` functions in `crates/tiler-ir/src/program/tests.rs`.

Optional items skipped (with reason):
  - Optional 2026-08-10 dated correction note: prose repaired in place on live Evidence/Exact-surface Facts; report says dated note not required when repaired in place.

Residuals not applied (docs/crates/new tickets/authority):
  - none (acceptance complete; report lists no docs/crates/remainder work)

Verification:
  - files read:
    - tickets/accept-the-staged-realization-program-declaration.md
    - audit report d8ac062da905_c99ac54950f2.md
    - crates/tiler-ir/src/program/tests.rs (staged-realization test names via grep)
    - crates/tiler-ir/src/program/builder.rs, error.rs (insertion error variants via grep)
  - checks:
    - five staged-realization `#[test]` fns present by name (helpers are not tests)
    - `push_staged_realization` returns SelfDependency, CoverageOutOfRange, DuplicateStagedRealization; only DuplicateStagedRealization is the new variant

Recommended next ledger state:
  integrated
