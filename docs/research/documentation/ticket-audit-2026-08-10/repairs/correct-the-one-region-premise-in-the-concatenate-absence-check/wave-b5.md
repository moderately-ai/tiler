Ticket: correct-the-one-region-premise-in-the-concatenate-absence-check
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-one-region-premise-in-the-concatenate-absence-check/0b27b9a09c96_c99ac54950f2.md
Pre-edit content hash (from ledger): 0b27b9a09c969f25c68dcf591fce0d64165879cd4257082d14a0999dc90812c8
Post-edit content hash: ab55cf76752d01c38455b44c98651b4684cc8c5af35f1f1f082f734817b02912

Changes applied:
  - Kept status `done`, empty dependencies, scopes unchanged (subject close condition still holds).
  - Frontmatter `related`: added `restate-the-fusion-role-table-census-in-the-indexing-records`.
  - Outcome: added **Correction — 2026-08-10** after the Out of scope paragraph — freezes present-tense remainder claims (slice absence, eleven keys, three-key arm) as landing-time relative to `cfe906cc`; states live fifteen `roles.insert` sites, slice registered CoordinateRelation, four-key contraction arm; notes concatenate checks 1–2 / nine-key body inventory are also live-stale; points ownership at the new remainder; explicitly does not reopen check 5.
  - Filed remainder `tickets/restate-the-fusion-role-table-census-in-the-indexing-records.md` (status `todo`, scope `research/indexing`): both indexing fusion-role records' role-table census drift, frontmatter on the sub-tensor record, checks 1–2 on both, concatenate body nine-key inventory; check 5 out of scope; related to this ticket, the slice role admission, and the concatenate role admission.

Optional items skipped (with reason):
  - None. Optional note that concatenate checks 1–2 are live-stale was folded into the required dated correction (cheap graph hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/indexing/sub-tensor-selection-fusion-role.md` — checks 1–2, `:110` eleven-key restatement, frontmatter disposition/implementation_status: owned by the filed remainder (not executed in wave B5).
  - `docs/research/indexing/concatenate-fusion-role-and-lowering.md` — checks 1–2 live counts and body "maps nine operation keys" inventory: owned by the same remainder; check 5 left alone.
  - Delivery-graph / matrix cell prose outside those two files: remainder non-goal (file separately if still lagging after restatement).

Verification:
  - files read:
    - tickets/correct-the-one-region-premise-in-the-concatenate-absence-check.md (full, before and after edit)
    - audit report 0b27b9a09c96_c99ac54950f2.md (full)
    - docs/research/indexing/sub-tensor-selection-fusion-role.md (frontmatter + checks + `:110` span)
    - docs/research/indexing/concatenate-fusion-role-and-lowering.md (nine-key body Fact + checks 1–2 span)
    - crates/tiler-compiler/src/fusion_legality.rs (roles.insert count; CoordinateRelation arm)
    - tickets/admit-a-fusion-role-for-the-sub-tensor-selection-slice.md (Outcome Out of scope flag; prior wave-b3 note)
  - checks:
    - `grep -c 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` → 15
    - arm membership: reindex || broadcast || concatenate || slice (four keys)
    - sub-tensor frontmatter still `disposition: "pending"` / `implementation_status: "not-started"`
    - `shasum -a 256` on parent ticket after edit → post-edit hash above
    - remainder file hash (record only): 403de0fbd877cf28bb62b0d8eb2b16c31c61471d16860cf5741558c62cf0aa8a

Recommended next ledger state:
  integrated
