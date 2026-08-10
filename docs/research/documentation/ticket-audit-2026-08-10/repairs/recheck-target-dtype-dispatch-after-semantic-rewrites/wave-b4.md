Ticket: recheck-target-dtype-dispatch-after-semantic-rewrites
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/recheck-target-dtype-dispatch-after-semantic-rewrites/9f09d92b4ad7_c99ac54950f2.md
Pre-edit content hash (from ledger): 9f09d92b4ad74d3dfb3cf4032968222955f7d9cf0b61dded9f6ce09a97d7c132
Post-edit content hash: f579a641d1c1d44fe4a5b9e6f60c765db2ee74faf2297f729c9b71abde08ae88

Changes applied:
  - Activation trigger: replaced f32-only ambient claim with type-set preservation language (admitted rewrites preserve each value's exact resolved type; BF16 admitted; CSE clones `resolved_type`).
  - 2026-08-04 log: retired stale `region.rs:433,497,539` line citations; replaced with `RuleRef::builtin(REGION_FORMATION_RULE)` / `REGION_CANDIDATE_RULE` anchors; kept normalize rule anchors with note that 179/247/290 still hold.
  - Added 2026-08-10 trigger-check log entry: not fired; records activation-body and region-citation repairs.

Optional items skipped (with reason):
  - none (report listed no optional bullets; metadata/status/deps/related/scopes left unchanged as required).

Residuals not applied (docs/crates/new tickets/authority):
  - none for wave B ticket prose. Eventual product work remains deferred: `readmit_candidate` dtype recheck + mutation fixtures when a rewrite mutates the resolved-type set (Exact files for implementation only; not in scope for this wave).

Verification:
  - files read:
    - tickets/recheck-target-dtype-dispatch-after-semantic-rewrites.md (full, pre/post)
    - audit report 9f09d92b4ad7_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/region.rs (grep: REGION_* builtins at 626, 690, 732)
    - crates/tiler-compiler/src/normalize.rs (grep: NORMALIZE_* builtins still at 179, 247, 290)
    - crates/tiler-compiler/src/rewrite.rs (ordered-reassociate-*-f32 identities present)
  - checks:
    - pre-edit sha256 matches ledger: 9f09d92b4ad74d3dfb3cf4032968222955f7d9cf0b61dded9f6ce09a97d7c132
    - post-edit sha256: f579a641d1c1d44fe4a5b9e6f60c765db2ee74faf2297f729c9b71abde08ae88
    - status remains deferred; no crates/docs product edits

Recommended next ledger state:
  integrated
