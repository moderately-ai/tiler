Ticket: add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral/d2dd2cc2f693_c99ac54950f2.md
Pre-edit content hash (from ledger): d2dd2cc2f693b9b729f4a6ce6cef2c6b8d3cf2627dade3d47cf2b6d92475c838
Post-edit content hash: c02c49b03510b9f6bc954cb8ef25341f1d3d75fcd06b3c8c6efaa165d249e2a9

Changes applied:
  - User-visible outcome: reframed present-tense "Today its triggers are sized… refuses" as historical "When this ticket opened, that deferral's triggers were sized… refused" so the body no longer asserts a live false Fact; Outcome left as delivery authority (report optional-but-recommended tense repair; required as strike of false present-tense Fact).
  - Why this exists: one-line bridge that the identity-curve constant moved 710 → 719 under the publishing-copy / `v10` step before the fold, and that ceiling arithmetic uses 719 (report optional constant-bridge).

Optional items skipped (with reason):
  - none — both optional prose items from the report were applied (cheap same-ticket hygiene; no product decision).

Residuals not applied (docs/crates/new tickets/authority):
  - Target ticket `decide-whether-executable-coverage-evidence-folds-as-a-digest` trigger-4 parenthetical still says "about ~46 once the manifest digest decision lands" while that ticket's own 2026-08-06 correction replaced the figure with 50/51 in *Why this exists* — repair belongs on the target if pursued (report out-of-scope note).
  - Outcome hash `453aef62` not reopened; delivery effect already verified via target body (report residual).
  - Live post-0103/0104 embedding-crossing (~148/149 at ×2) not restated on this ticket; historical pre-fold numbers remain correctly historical after the tense reframe.

Verification:
  - files read:
    - tickets/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral/d2dd2cc2f693_c99ac54950f2.md (full)
    - tickets/decide-whether-executable-coverage-evidence-folds-as-a-digest.md (status + trigger 4 line)
    - crates/tiler-compiler/src/request.rs (semantic_operations: 62 in DeterministicBudgets::governed)
  - checks:
    - pre-edit shasum -a 256 matched ledger hash d2dd2cc2f693b9b729f4a6ce6cef2c6b8d3cf2627dade3d47cf2b6d92475c838
    - post-edit shasum -a 256 = c02c49b03510b9f6bc954cb8ef25341f1d3d75fcd06b3c8c6efaa165d249e2a9
    - target status done; trigger 4 text present; governed semantic_operations still 62
    - metadata unchanged (status done, deps [], scopes [], shared_scopes project/tickets)

Recommended next ledger state:
  integrated
