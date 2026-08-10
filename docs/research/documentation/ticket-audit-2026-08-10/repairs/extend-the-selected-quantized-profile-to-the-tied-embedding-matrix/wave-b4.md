Ticket: extend-the-selected-quantized-profile-to-the-tied-embedding-matrix
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/extend-the-selected-quantized-profile-to-the-tied-embedding-matrix/1a50570e1203_c99ac54950f2.md
Pre-edit content hash (from ledger): 1a50570e120352d646570d963e998b93dcdb16f361e79f9d1cdf3418bf93d0c7
Post-edit content hash: 48bc58af13cedb867d35a59c4497e7ac8c258bf594e1d4d1cdbaf2132cf57e45

Changes applied:
  - § Activation boundary: replaced "neither exists" with precise residuals — `implement-first-quantized-backend-profile` still open (no executable selected quantized path), and `tiler::gather-f32@1` admits only `tiler::f32@1` sources so a compound/quantized tied matrix cannot be a gather operand.
  - § Why it is separate Fact sentence: replaced "which the profile does not cover and which Tiler cannot express" with "which the selected profile does not cover and which the admitted F32 gather cannot take as a compound source (ADR 0107 / `tiler::gather-f32@1`)."
  - Trigger check log: added 2026-08-10 **not fired** entry (gather done F32-only; backend profile still todo; compound gather source unadmitted).
  - Status left `deferred` (no metadata change required).

Optional items skipped (with reason):
  - Dependency list change: report says none required when activation prose names the residual as compound source rather than family absent; list retained.

Residuals not applied (docs/crates/new tickets/authority):
  - L7 first-quantized-lm-profile.md still states gather "has not delivered" in one proposal paragraph — research drift outside ticket scopes.
  - Compound-source gather vs this ticket's outcome ownership left as residual uncertainty (no split; report says no split required solely by this audit).
  - Product work (IR compound-source admission, backend profile, C1 through Tiler) out of wave B scope.

Verification:
  - files read:
    - tickets/extend-the-selected-quantized-profile-to-the-tied-embedding-matrix.md (full, pre/post)
    - audit report 1a50570e1203_c99ac54950f2.md (full)
    - dependency status greps: implement-first-quantized-backend-profile=todo, admit-an-indirect-gather-family-for-tied-embedding-lookup=done, reclassify-language-model-work-as-a-conformance-track=done
    - crates/tiler-ir/src/semantic/gather.rs anchors: gather_f32_op, SourceNotF32, tiler::gather-f32@1 F32-only source
  - checks:
    - shasum -a 256 tickets/extend-the-selected-quantized-profile-to-the-tied-embedding-matrix.md → 48bc58af13cedb867d35a59c4497e7ac8c258bf594e1d4d1cdbaf2132cf57e45

Recommended next ledger state:
  integrated
