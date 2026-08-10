Ticket: correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical/c549fccdd133_c99ac54950f2.md
Pre-edit content hash (from ledger): c549fccdd133285b6f2d009bb7075609ee0787e302d7345bf10a622c63181081
Post-edit content hash: 88a171ec63f3300dbdbcf249a326516933a401cde180dc1540ed7ffb5541f193

Changes applied:
  - Struck the unstruck opening clause equating `Unknown { "unproven-reassociation" }` with "the surviving contraction wall and a different region"; dated 2026-08-10 correction points to Worker finding / Outcome and names the true BF16 contraction wall (`unrealized-contraction` / `ArithmeticContraction`) and the explicit assertion site (`a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`).
  - Softened Outcome "reached only at" to "explicitly asserted at" and noted production reachability is broader than the named checkable site (optional uniqueness-language tighten from the audit).

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required ticket prose only; no docs/crates/new tickets; metadata already fine)

Verification:
  - files read:
    - audit report `c549fccdd133_c99ac54950f2.md` (entire)
    - ticket body (entire, pre- and post-edit)
    - re-checked anchors: `push_reduction_obligations` disjunction in `fusion_legality.rs`; `"unproven-reassociation"` assert site; `unrealized-contraction` contraction-wall test
  - checks:
    - `shasum -a 256` on ticket → post-edit hash above
    - `rg` confirms struck clause + "explicitly asserted at" present; no live unstruck wall-equating claim remains

Recommended next ledger state:
  integrated
