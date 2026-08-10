Ticket: share-one-structured-kernel-interpreter
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/share-one-structured-kernel-interpreter/2b2f998e39cc_c99ac54950f2.md
Pre-edit content hash (from ledger): 2b2f998e39cc8acf8cd1863af19f61d115258d6ba8dde4d95875af14ace34f68
Post-edit content hash: 22cdf8616955765fd937b85a22d25831fc519088ca1a1070cae4e3254dd9e822

Changes applied:
  - Decision packet close-out: replaced "move the complete IR machine" with shared-authority language that must preserve IR barrier-containing-loop flatten (Seed/Iterate/Yield/Exit + Rendezvous split) and compiler multi-buffer, bf16, and op vocabulary surfaces, then delete both private copies without a third temporary.
  - User-visible outcome: clarified that "weaker" means barrier-nesting / multi-round executability only; pipeline machine is richer on multi-buffer, bf16, and op vocabulary.

Optional items skipped (with reason):
  - none (optional "weaker" clarity applied as cheap same-ticket prose honesty).

Residuals not applied (docs/crates/new tickets/authority):
  - Compiler `KirMachine` present-tense comments claiming every barrier at block depth zero (source rot; out of ticket-owned decision; fixable in post-decision implementation pass).
  - Tom ownership decision (Option A vs B) remains open; status stays `awaiting-decision`.
  - Causal "occupied lane" process-history clause left as Inference (unverifiable at base; narrative only).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/share-one-structured-kernel-interpreter/2b2f998e39cc_c99ac54950f2.md
    - tickets/share-one-structured-kernel-interpreter.md
    - rg `struct KirMachine` under crates/ → two hits (compiler pipeline tests, IR kernel tests)
    - rg compiler pipeline tests surfaces: `barrier_segments`, `declared_buffers`, `Bf16Canonicalization`, `IndexSubtract`, `F32Divide`, `F32Exp`, `F32Rsqrt`, `CanonicalizeBf16Nan`
    - rg IR kernel tests: `contains_barrier` flatten path
  - checks:
    - metadata unchanged (status awaiting-decision; deps []; related pair; scopes; tags) per report "none"
    - post-edit sha256: 22cdf8616955765fd937b85a22d25831fc519088ca1a1070cae4e3254dd9e822

Recommended next ledger state:
  integrated
