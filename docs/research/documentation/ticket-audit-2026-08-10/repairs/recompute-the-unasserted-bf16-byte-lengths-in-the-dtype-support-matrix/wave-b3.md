Ticket: recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix/baf32cab65aa_c99ac54950f2.md
Pre-edit content hash (from ledger): baf32cab65aa1d7cd2f74174ff3861ea8e6fa5d5afb6e25048aea06dd20333ed
Post-edit content hash: 2090030a0a54e2a076a40b0a0a70d11e63cfbc68739e11b6e4ee66f462fca32b

Changes applied:
  - Frontmatter `related`: added `replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin` and `pin-the-differing-identity-positions-beside-the-carrier-positions-constant` beside the existing envelope-related edge (optional graph hygiene from the report).
  - `## Out of scope`: struck "six" → live **nine**; labeled the trailing quartet as "the four lengths"; appended (`1 + 4 + 4 = 9`) so the count matches the enumerated list (identity length + four offsets + four lengths).
  - Added **Correction — 2026-08-10.** naming the miscount and the sibling's 2026-08-08 `1 + 4 + 4` repair as the matching authority.

Optional items skipped (with reason):
  - none (optional related edges applied as cheap graph hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/dtype-support.md` still carries present-tense "nothing asserts it either" for the four-byte identity difference after `DIFFERING_IDENTITY_POSITIONS` landed (post-pin doc drift; not this ticket's original close condition; wave B ticket-only).
  - `docs/artifact-abi.md` "What is left unpinned" still claims identity length equality and four positions unasserted (same residual class; product doc path out of wave B).
  - No new remainder ticket filed (report: none required for the original absolute-length defect).

Verification:
  - files read:
    - tickets/recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/recompute-the-unasserted-bf16-byte-lengths-in-the-dtype-support-matrix/baf32cab65aa_c99ac54950f2.md (full)
    - tickets/replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md (frontmatter + six→nine Fact; confirmed sibling count `1 + 4 + 4 = 9` and status `done`)
    - tickets/pin-the-differing-identity-positions-beside-the-carrier-positions-constant.md (id exists; related edge target)
  - checks:
    - Count of Out of scope list: identity `48,584` + offsets `3,104`/`3,106`/`47,898`/`47,899` + lengths `90,806`/`45,457`/`73,556`/`36,832` = 9
    - `shasum -a 256` of ticket after edit → `2090030a0a54e2a076a40b0a0a70d11e63cfbc68739e11b6e4ee66f462fca32b`

Recommended next ledger state:
  integrated
