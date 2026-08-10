Ticket: project-only-the-final-position-logits
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/project-only-the-final-position-logits/ae224aef8695_c99ac54950f2.md
Pre-edit content hash (from ledger): ae224aef8695ed3a5abb3e3a94025fa7b792b7cee5182aa5ffb14540b9919abe
Post-edit content hash: 662b57078b0f23c47dda4fe53d37c1fda743818da582c1f5e6dfe375377cbcdf

Changes applied:
  - Why-this-exists closing sentence: detached "is the family this depends on" from the rotary third-trigger clause; live prose now states third trigger is rotary-only (`related`), and the family dependency is `admit-the-sub-tensor-selection-family` / `tiler::slice-f32@1` (frontmatter `dependencies`).
  - Dated **Correction — 2026-08-10** for that mis-attached clause (frontmatter was always correct).
  - Dated **Correction — 2026-08-10** for inherited off-by-4096 B1-d full-logits figure from L6: full-T `4,978,638,848`, saving `4,978,031,104`, all-positions D-B peak `10,895,491,080`; final-position peak `5,917,459,976` left standing; original Inference digits left as L6 inheritance rather than silent rewrite.
  - Frontmatter status/dependencies/related left unchanged (report: graph edges already right).

Optional items skipped (with reason):
  - None required on this ticket; optional L6 delivery-table / "family the corpus does not define" note is residual product debt outside ticket scopes (see Residuals).

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/program-planning/complete-model-ingestion-and-execution.md: residency/logits digits and delivery ticket 12 waits-on still disagree with post-reclassify graph (report: repair belongs to L6 / design-model-ingestion scopes).
  - docs/roadmap.md Slice trigger digits if kept in lockstep with corrected full-T figure.
  - docs/research/shapes/sequence-extending-tensor-family.md historical figure if corrected rather than dated.
  - Residual uncertainty from audit (slice residual-before-projection vs logits-after; literal vs symbolic T) left for implementer; not prose-repair scope.

Verification:
  - files read:
    - tickets/project-only-the-final-position-logits.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/project-only-the-final-position-logits/ae224aef8695_c99ac54950f2.md
  - checks:
    - `python3 -c` recompute: `8192 * 151936 * 4 = 4978638848`; `151936 * 4 = 607744`; saving `4978031104`; all-positions peak `10895491080`; final-position peak `5917459976`
    - `shasum -a 256 tickets/project-only-the-final-position-logits.md` → post-edit hash above
    - frontmatter unchanged: dependencies include admit-the-sub-tensor-selection-family; rotary only in related; status todo

Recommended next ledger state:
  integrated
