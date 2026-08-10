Ticket: preserve-the-pytorch-conversion-platform-variation-source
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/preserve-the-pytorch-conversion-platform-variation-source/2bb7ecff4d19_c99ac54950f2.md
Pre-edit content hash (from ledger): 2bb7ecff4d1951f92d1c54d808f7c6a6f8b743a0fb80770b566e5e40262a0677
Post-edit content hash: e59e36ddcaf7be01e9ff05367e0d8357b28d2c6b6b7cfe4af0e30462e1549ff7

Changes applied:
  - Defect-section post-landing six-line grep characterization: replaced false "mutating a tensor shared with NumPy" clause with accurate enumeration of the three non-claim `_tensor_docs.py` hits (`index_put_` accumulate undefined-on-duplicate-indices at 2467; `put_` accumulate same at 3781; TorchScript view-dtype overload undefined behavior at 6133); kept six-line count, ScalarType.h pair, and claim line 5165; marked with **Correction — 2026-08-10.**

Optional items skipped (with reason):
  - none (optional dated-correction path was folded into the required prose fix as an inline **Correction — 2026-08-10.** rather than a free-standing frozen-ticket note)

Residuals not applied (docs/crates/new tickets/authority):
  - none (report listed ticket-only repair; metadata/deps/related/status left as-is; no docs/crates remainder)

Verification:
  - files read:
    - tickets/preserve-the-pytorch-conversion-platform-variation-source.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/preserve-the-pytorch-conversion-platform-variation-source/2bb7ecff4d19_c99ac54950f2.md
    - docs/research/numerics/sources/pytorch-v2.13.0/_tensor_docs.py (hit contexts at 2455–2474, 3770–3788, 6120–6138)
  - checks:
    - `grep -rniE 'platform|undefined|out of range|saturat' docs/research/numerics/sources/pytorch-v2.13.0/` → six lines: ScalarType.h:170, ScalarType.h:209, _tensor_docs.py:2467, 3781, 5165, 6133
    - post-edit `shasum -a 256 tickets/preserve-the-pytorch-conversion-platform-variation-source.md` → e59e36ddcaf7be01e9ff05367e0d8357b28d2c6b6b7cfe4af0e30462e1549ff7

Recommended next ledger state:
  integrated
