Ticket: retire-the-corrected-softmax-fact-quotations
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/retire-the-corrected-softmax-fact-quotations/6079ac66c8d2_c99ac54950f2.md
Pre-edit content hash (from ledger): 6079ac66c8d21de92f91fae4474ae9d005b6beb1ce09672a63d19b0db547c1a4
Post-edit content hash: 293696a58133cfceea6e801f1fd0792062938fce140f913495061616763103d9

Changes applied:
  - Required: added `## Outcome` naming landing commit `12d72e20`, the two dated sites (elementary Part 9; admit-the-softmax-family), sibling ticket residuals already dated 2026-08-10, and residual undated docs hits under the old close-condition population.
  - Required: narrowed User-visible outcome from corpus-wide "no document quotes … as reassociation" to the two-site dating that actually landed; dated Correction records the overstated claim.
  - Required: narrowed Closes when to the two-site dating + `tkt lint`; dropped the substring-grep close condition that cannot distinguish corrected-value hits from undated old-string hits; dated Correction explains why.
  - Required: Fact audit — 2026-08-10 keeps status `done` under the narrowed two-site scope without reopen.
  - Ticket residual sites from Repair required prose item 1: verified already discharged on current tree — `name-the-elementary-identity-rewrite-dimension` Outcome has `**Correction — 2026-08-10.**`; `correct-the-online-single-pass-softmax-fold-legality-fact` Why has HISTORICAL labels + Correction; no re-edit.

Optional items skipped (with reason):
  - none applicable beyond graph; related list already names name-the-elementary and correct-the-online; no cheap related addition without inventing a remainder id.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/numerics/elementary-identity-rewrite-dimension.md` Open axes bullet still present-tense "A registered fact says the online single-pass softmax form is a reassociation…" (Class C / wave ticket-only: do not edit docs/).
  - ADR 0101 Open questions still present-tense "a registered definition fact in the tree states that the online single-pass softmax form is a reassociation" (needs `contracts/decisions` scope or a connected remainder; report listed no concrete remainder id — not filed in this wave).
  - No crates/ edits; identity step already landed under correct-the-online / `28fe26a8`.

Verification:
  - files read:
    - full audit report 6079ac66c8d2_c99ac54950f2.md
    - full tickets/retire-the-corrected-softmax-fact-quotations.md (pre/post)
    - name-the-elementary Outcome + Correction; correct-the-online Why HISTORICAL + Correction
    - elementary Part 9 Acted-on dating; admit-the-softmax-family superseded parenthetical
    - elementary Open axes residual bullet; ADR 0101 Open questions residual sentence
    - crates/tiler-ir/src/semantic/softmax.rs SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM registration (corrected multi-clause string)
    - git show 12d72e20 --stat; rev-parse 12d72e20 / 28fe26a8
  - checks:
    - `shasum -a 256 tickets/retire-the-corrected-softmax-fact-quotations.md` → 293696a58133cfceea6e801f1fd0792062938fce140f913495061616763103d9
    - ticket has `## Outcome` and no corpus-wide live UVO claim
    - residual docs greps still hit Open axes + ADR 0101 as listed
    - sibling ticket Corrections present; crates still corrected (not-a-reassociation-of-the-sum-but-a-horner-…)

Recommended next ledger state:
  integrated
