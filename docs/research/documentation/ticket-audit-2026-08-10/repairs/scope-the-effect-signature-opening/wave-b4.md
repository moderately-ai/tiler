Ticket: scope-the-effect-signature-opening
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-effect-signature-opening/de5a2f64bcb5_c99ac54950f2.md
Pre-edit content hash (from ledger): de5a2f64bcb527461f64bd3572432d17705762ba486282cd740905d874d47eb6
Post-edit content hash: 6623dbd7eae5fe257c069138911e2ebb167086746f3a07ae3ecf20c3ae622c70

Changes applied:
  - Why-deferred Fact: replaced "three encoders outside `tiler-ir`" with declaration-named total mappers (legality + fusion_legality outside; registry encoder inside) plus the fourth total match in index refinement.
  - Dated correction 2026-08-10: prior "outside `tiler-ir`" census was imprecise on count/location; fail-closed mechanism stands.
  - "What the work would be": encoder consequence sentence now names the three declaration mappers plus refinement, without the false outside-only census.
  - Explicit non-goals: "three encoders" rephrased to "every total encoder of the vocabulary".

Optional items skipped (with reason):
  - none (optional dated note applied as cheap prose hygiene on this ticket)

Residuals not applied (docs/crates/new tickets/authority):
  - Same "outside `tiler-ir`" phrase drift in roadmap matrix row and ADR 0020 implementation-boundary text (out of ticket file; coordinator/corpus scope per report).
  - Declaration doc on `OperationEffect` still says "three encoders outside this crate" while listing the in-crate registry encoder among them (crates/docs residual, not ticket repair).

Verification:
  - files read:
    - tickets/scope-the-effect-signature-opening.md (full)
    - audit report de5a2f64bcb5_c99ac54950f2.md (full)
    - crates/tiler-ir/src/semantic/operation.rs (`OperationEffect` declaration)
    - crates/tiler-compiler/src/legality.rs (`effect_tag`)
    - crates/tiler-compiler/src/fusion_legality.rs (`effect_tag`)
    - crates/tiler-ir/src/semantic/registry.rs (definition effect encode)
    - crates/tiler-ir/src/index/refinement.rs (subject.effect total match)
  - checks:
    - four total `OperationEffect::Pure => 1` match sites confirmed; only legality + fusion_legality outside `tiler-ir`
    - status left `deferred`; no metadata change required

Recommended next ledger state:
  integrated
