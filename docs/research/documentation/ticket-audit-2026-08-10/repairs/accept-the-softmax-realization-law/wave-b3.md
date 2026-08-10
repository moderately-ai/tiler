Ticket: accept-the-softmax-realization-law
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-softmax-realization-law/30a5bc28549a_c99ac54950f2.md
Pre-edit content hash (from ledger): 30a5bc28549a546cdbb90d20ff993c70d8351583a2db658a61133016dbcae545
Post-edit content hash: 00ca51928d1372ab25ea1e85eca2d8890162b65dc54d36c4e8d73a0f4c662489

Changes applied:
  - Added `## Current-state correction — 2026-08-10` stating that Tom's 2026-08-07 acceptance did **not** flip the in-source `**Draft boundary.**` on `IndexRealizationLaw::StagedSoftmaxF32`; rustdoc still claims "labelled draft awaiting Tom's decision" / "the label is what an acceptance flips", which is false after acceptance; accepted surface/registration/tag/identity unchanged; records unfiled remainder owed for accepted-boundary language (or removing the awaiting-decision claim) without reopening the surface.

Optional items skipped (with reason):
  - none — report listed no optional ticket-prose items; metadata already sound (status/deps/related/scopes).

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-ir/src/index/law.rs — `StagedSoftmaxF32` Draft boundary docs still await flip; wave B ticket-only (out of scope).
  - Remainder ticket — report requires filing or connecting a narrow remainder for the draft-boundary docs flip, but gives no concrete id; blocked residual for coordinator id assignment (not filed here). Sibling `PartitionedConcatenate` leftover draft boundary noted as same class, out of this ticket.
  - Public-boundary documentation defect only; no law vocabulary / identity / architecture change required for registration or realization correctness.

Verification:
  - files read:
    - tickets/accept-the-softmax-realization-law.md (full, before and after)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-softmax-realization-law/30a5bc28549a_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/law.rs — confirmed `**Draft boundary.**` / `labelled draft awaiting Tom's decision at` / `the label is what an acceptance flips` still present on `StagedSoftmaxF32`
  - checks:
    - Fact 12 (false live draft label after acceptance) re-verified via `rg` on law.rs
    - status/deps/related/scopes left unchanged per report
    - post-edit sha256 via `shasum -a 256` on the ticket path

Recommended next ledger state:
  integrated
