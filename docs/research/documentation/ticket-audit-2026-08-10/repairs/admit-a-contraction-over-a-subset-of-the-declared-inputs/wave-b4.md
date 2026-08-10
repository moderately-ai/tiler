Ticket: admit-a-contraction-over-a-subset-of-the-declared-inputs
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-contraction-over-a-subset-of-the-declared-inputs/e2b96392f7d2_c99ac54950f2.md
Pre-edit content hash (from ledger): e2b96392f7d2b02adad4bce0c969e43dc5633f3ca15417d92351f35f41278350
Post-edit content hash: 975b3711ccb3e3d51f7df6997ba96c6cb4abe761a5a9a50665ef1e85e4d3834b

Changes applied:
  - Trigger prose: separated (a) multi-output binary contraction beside an independent output retaining a skipped declaration from (b) single-output `rms_norm(matmul(a, b), w)` retaining the third declaration via the staged consumer; dropped "exactly that shape" equating them; named both as subjects of `contraction-input-arity`.
  - Appended `## Fact audit — 2026-08-10` restating the two subjects, live refusal under `contraction-input-arity`, fixture two-input spelling dodge, and optional ADR 0087 "binary family" wording note (reserved multi-operand / fifth structural rule, not a second family key).
  - Metadata left unchanged (status todo, dependency, related, scopes coherent per report).

Optional items skipped (with reason):
  - none (optional ADR 0087 wording note included in the dated correction as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none for this audit repair; product implementation (IR map, physical consumers, identity determination, residual arity/distinct-operand behaviour, perturbations) remains the open ticket work and is out of scope for wave B4 prose repair. No new remainder tickets required by the report.

Verification:
  - files read:
    - tickets/admit-a-contraction-over-a-subset-of-the-declared-inputs.md (full, pre + post)
    - audit report e2b96392f7d2_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/request.rs anchors: `return mismatch("contraction-input-arity")`, "Two declared inputs rather than three", "because `contraction-input-arity` requires exactly two declarations"
    - crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs and recognized_chain_depth_boundary.rs two-input dodge comments
    - crates/tiler-compiler/tests/contraction_direct_path.rs retained-third-input and one-input refusal rows under `contraction-input-arity`
    - docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md fifth structural rule `no index in more than two operands`
  - checks:
    - status remains todo; no metadata graph change required
    - sha256 post-edit: 975b3711ccb3e3d51f7df6997ba96c6cb4abe761a5a9a50665ef1e85e4d3834b

Recommended next ledger state:
  integrated
