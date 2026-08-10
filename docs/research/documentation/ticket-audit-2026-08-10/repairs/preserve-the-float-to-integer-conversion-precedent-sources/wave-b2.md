Ticket: preserve-the-float-to-integer-conversion-precedent-sources
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/preserve-the-float-to-integer-conversion-precedent-sources/e83eeeda0572_c99ac54950f2.md
Pre-edit content hash (from ledger): e83eeeda0572824e74b47064a186d7b2149ff7b4e35f08ac3ce8fadb17a7f552
Post-edit content hash: 95a3c1325288f0eb1899a967eed519bacde56eedb4b4a2faacdafd3c92da5b4e

Changes applied:
  - Opening Fact: replaced "sole `evidence` record for ADRs 0010 and 0041" with sole for ADR 0041 and co-evidence (with `tiler.research.numerics.dtype-resolution-precedents`) for ADR 0010.
  - Inference: lesson quote now attributed to mature-operation taxonomy (where the sentence lives) while still linking the preservation record as what enforces the practice.
  - Optional related graph hygiene: added `preserve-the-pytorch-conversion-platform-variation-source` to frontmatter `related:`.

Optional items skipped (with reason):
  - none (optional related addition applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by this audit (PyTorch remainder already split and done; no docs/crates/ADR edits).

Verification:
  - files read:
    - tickets/preserve-the-float-to-integer-conversion-precedent-sources.md
    - audit report e83eeeda0572_c99ac54950f2.md
    - docs/decisions/0010-typed-conversion-contracts.md (evidence frontmatter)
    - docs/decisions/0041-separate-float-to-integer-conversion-families.md (evidence frontmatter)
    - docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md (lesson quote via rg)
    - tickets/preserve-the-pytorch-conversion-platform-variation-source.md (related back-link)
  - checks:
    - ADR 0010 evidence: dtype-resolution + float-to-integer (two ids)
    - ADR 0041 evidence: float-to-integer only
    - lesson sentence only in mature-operation-and-signature-taxonomy.md under docs/
    - pytorch sibling related: [preserve-the-float-to-integer-conversion-precedent-sources]
    - post-edit sha256: 95a3c1325288f0eb1899a967eed519bacde56eedb4b4a2faacdafd3c92da5b4e

Recommended next ledger state:
  integrated
