Ticket: accept-the-governed-reciprocal-square-root-scalar-key
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-governed-reciprocal-square-root-scalar-key/8519fb813825_c99ac54950f2.md
Pre-edit content hash (from ledger): 8519fb8138256071ac360b5cd17dc78a17ea5332f99d20f7b8863e319b16e7ed
Post-edit content hash: 56020560503ac927e9596e9282a11566f04bb1924036f91d29eb6d7c27791459

Changes applied:
  - Excluded: struck present-tense false claim that no realization law names the key / no region can carry it; restated that realization is owned by the widen ticket and has landed (`StagedRootMeanSquareScaleF32` applies `rsqrt_f32_scalar_op`); kept true exclusions (no bf16 sibling; no square-root key).
  - Accepted 2026-08-06: replaced `(tenth key, exp/divide shape)` with admission-time eleventh-key / elementary-unary / exp-shape wording aligned with the surface section and pre-admission ten-key census.
  - Added **Correction — 2026-08-10.** noting Excluded went stale after the 2026-08-09 code-half correction, the tenth-key ordinal conflict, and that later `maximum-f32` (twelfth key) does not change the admission-time ordinal.

Optional items skipped (with reason):
  - related edge to `widen-the-staged-realization-law-to-the-registered-elementary-families`: report marks it optional non-blocker already named by prose link; Repair required says none for metadata/scopes.

Residuals not applied (docs/crates/new tickets/authority):
  - none — repair is ticket-prose only; no docs/crates edits or remainder filing required.

Verification:
  - files read:
    - tickets/accept-the-governed-reciprocal-square-root-scalar-key.md (full, pre/post)
    - audit report 8519fb813825_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/law.rs (grep: StagedRootMeanSquareScaleF32 applies rsqrt_f32_scalar_op)
    - crates/tiler-ir/src/index/scalar.rs (grep: rsqrt registration among builder.register blocks)
    - crates/tiler-ir/src/index/mod.rs (rsqrt re-export)
  - checks:
    - `rsqrt_f32_scalar_op` used in law emit path at apply_one for StagedRootMeanSquareScaleF32
    - multiple builder.register in standard() including rsqrt and later maximum-f32
    - false Excluded present-tense sentence removed from ticket body
    - Accepted no longer says "tenth key"

Recommended next ledger state:
  integrated
