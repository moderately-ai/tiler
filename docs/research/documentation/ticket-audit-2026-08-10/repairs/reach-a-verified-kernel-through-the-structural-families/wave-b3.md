Ticket: reach-a-verified-kernel-through-the-structural-families
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reach-a-verified-kernel-through-the-structural-families/1f1ab4fc0f4e_c99ac54950f2.md
Pre-edit content hash (from ledger): 1f1ab4fc0f4ef9b2653552c54884db725421bb168bf01e347e735393fd5d288f
Post-edit content hash: 2fbb83528db54889deebdbcebbf5539d9893a25b118b21afdb1e80a0ffc7cfa3

Changes applied:
  - Added **Correction — 2026-08-10** under Outcome 2026-08-06 rung residual subsection: emit-the-structural-region-on-metal is done; IndexSubtract is emitted under crates/tiler-metal; structural goldens exist; support-matrix structural row is R6 (offline translation bound, R7 unmet). Historical "this ticket left R5 because no backend was asked" left intact as this ticket's resolution.
  - Annotated Outcome lead "row stays at R5" and Required delivery R5 resolution as historical close, not live matrix residual, with dated correction pointing at related emit discharge.
  - Corrected 2026-08-05 inventory note: NormalizedOutput live variants are five (SerialSum, Pointwise, Contraction, Epilogue, Staged); sole_output is not itself #[cfg(test)] — only the shape accessors wrapping it are.

Optional items skipped (with reason):
  - none (optional NormalizedOutput / sole_output inventory notes applied as cheap hygiene on the same ticket)

Residuals not applied (docs/crates/new tickets/authority):
  - none required; report Exact files listed only this ticket; matrix already R6; no remainder ticket; R7 is product/support-matrix planning outside this close condition

Verification:
  - files read:
    - tickets/reach-a-verified-kernel-through-the-structural-families.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/reach-a-verified-kernel-through-the-structural-families/1f1ab4fc0f4e_c99ac54950f2.md
    - crates/tiler-compiler/src/request.rs (NormalizedOutput enum; sole_output + #[cfg(test)] accessors)
    - crates/tiler-metal/src/emit.rs (IndexSubtract arm present)
    - tickets/emit-the-structural-region-on-metal.md (status: done)
    - docs/roadmap.md (structural row R6 for the two admitted families)
  - checks:
    - rg NormalizedOutput variants: five arms including Epilogue and Staged
    - sole_output at request.rs without #[cfg(test)]; serial_sum/pointwise/contraction accessors are cfg(test)
    - rg IndexSubtract under crates/tiler-metal/ hits emit.rs, tests.rs, golden_compilation.rs
    - roadmap structural cell: R6 for the two admitted families
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
