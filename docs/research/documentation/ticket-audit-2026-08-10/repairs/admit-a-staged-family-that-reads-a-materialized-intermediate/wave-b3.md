Ticket: admit-a-staged-family-that-reads-a-materialized-intermediate
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-staged-family-that-reads-a-materialized-intermediate/ddcf12616840_c99ac54950f2.md
Pre-edit content hash (from ledger): ddcf1261684006eacc2a63648dbab7921a944538c17798989a99f37129dc6211
Post-edit content hash: eab7aaa820f70521fc2bd3e7d6c7145eafc3b701689cbc69fb8bfb00a3c20d70

Changes applied:
  - Fact corrections (2026-08-07 block): reclassified `plan_elementwise`'s `leaves.staged.is_none()` guard as chain **width** / unordinalled Intermediate, owned with `admit-a-scheduled-region-that-reads-two-materialization-edges` (second-read ticket for one-value-twice); kept **depth** solely as `staged-operand-depth` / `StagedOperandAdmission` under `admit-a-recognized-chain-more-than-one-materialization-boundary-deep`.
  - Fact corrections: renamed residual contraction declared-count rule from `input-arity` to `contraction-input-arity`; noted program-wide zero-input still uses `input-arity`; linked naming + widening remainder pair.
  - Outcome "A conflation…" paragraph: same width vs depth residual-wall map and `contraction-input-arity` name.
  - Appended `## Fact audit — 2026-08-10` so the 2026-08-07 "repair" is not live residual-wall authority.
  - Optional related frontmatter: remainder tickets already named in prose (`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`, `admit-a-scheduled-region-that-reads-two-materialization-edges`, `admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`, `name-the-contraction-operand-arity-wall-and-separate-its-rule`, `admit-a-contraction-over-a-subset-of-the-declared-inputs`).

Optional items skipped (with reason):
  - none (related edges applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report; historical "Where the refusal is" present-tense wall description remains historical/superseded by Outcome (audit Fact 1 HISTORICAL, not a required repair). No docs/crates edits in this wave.

Verification:
  - files read:
    - tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md (full, pre + post)
    - audit report ddcf12616840_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/request.rs anchors: plan_elementwise Folded comment ("chain *width* rather than depth"), StagedOperandAdmission docs (width neighbour vs `staged-operand-depth`), `return mismatch("contraction-input-arity")` in `normalize_contraction`, residual `input-arity` only for zero-input program check
    - tickets/admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md (2026-08-08 width correction)
  - checks:
    - rg confirmed `contraction-input-arity` at normalize_contraction and contraction_direct_path tests
    - sha256 post-edit: eab7aaa820f70521fc2bd3e7d6c7145eafc3b701689cbc69fb8bfb00a3c20d70

Recommended next ledger state:
  integrated
