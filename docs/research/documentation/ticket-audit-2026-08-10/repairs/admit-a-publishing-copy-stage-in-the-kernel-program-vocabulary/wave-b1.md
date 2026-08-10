Ticket: admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-publishing-copy-stage-in-the-kernel-program-vocabulary/72f1c7cc1365_c99ac54950f2.md
Pre-edit content hash (from ledger): 72f1c7cc1365d5034a0f762b1efa4647507b8db5c98f20932f5a15986ca40789
Post-edit content hash: 79f505ef0218e6be1a1cb2b4e27fa477436d0d03ad3106fcea49c550775846ac

Changes applied:
  - Marked Why-section empty-coverage Fact historical (pre-widening `verify_partial_reductions` / split-only quote retained as measurement text); noted live `verify_stage_accounts` admits split, publishing-copy, and staged-realization accounts.
  - Marked `MaterializesAndPublishes` compiler-route Fact historical for the attribution arm; **Correction — 2026-08-10** that the variant is gone and publishing copies assemble; kept `CoverDuplicationAdmission::Forbidden` as still live.
  - Softened Outcome close "gate still refuses" to past-at-close wording scoped to this ticket; added `## Fact audit — 2026-08-10` covering empty-coverage site, MaterializesAndPublishes retirement, positive successor gate row, and wall-4 table site-name drift (table left historical).
  - Optional: added `lift-the-four-published-and-consumed-walls-together` to frontmatter `related` for discoverability.
  - Marked the design Inference delivered/obsolete as open work (same successor).

Optional items skipped (with reason):
  - none (successor `related` entry applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report (Exact files: ticket only; no remainder tickets; no docs/crates edits).
  - Outcome narrative still contains past-tense discovery phrasing such as "UncoveringStage is unreachable today" in the stop paragraph; not listed under Repair required, left as historical discovery voice under the Fact audit.

Verification:
  - files read: assigned audit report; full ticket; `crates/tiler-ir/src/program/verify.rs` (`verify_stage_accounts` admits three accounts; `verify_publishing_copies` / `verify_staged_realizations`); greps for `MaterializesAndPublishes` and `a_published_and_consumed_intermediate_compiles_and_agrees` under `tiler-compiler`.
  - checks: empty-coverage gate at `verify_stage_accounts` with `publishing_copies` and `staged_realizations` arms; positive conformance test name present; MaterializesAndPublishes only in retirement docs/comments.

Recommended next ledger state:
  integrated
