Ticket: define-the-model-weight-binding-manifest
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/define-the-model-weight-binding-manifest/bdf559f18567_c99ac54950f2.md
Pre-edit content hash (from ledger): bdf559f1856728bd31264142152e8dad4795a3c6e6b0a0f310e7241b9e9a5690
Post-edit content hash: a87ac2e78e81e14b0c060b52e0bb56c4844b2b94597c17951879102e20357d28

Changes applied:
  - Why this exists: replaced false product `57! · 56!² · 28!⁴` with `57! · 56!³ · 28!³` from the ticket's own eight-class table (three size-56, three size-28).
  - Why this exists: widened opening bind-check summary past "shape, dtype, rank, or symbol" to include operand count, adapter capability, and dense storage-length on the dispatch path (optional recommended item).
  - Why it belongs to the consumer: replaced "once at model-state creation" with "once when the consumer builds its bound weight set"; noted ADR 0021 is analogy not weight-binding scope.
  - Required content: rephrased map as total/injective over checkpoint's 310 tensor names (or fully qualified layer+role slots), each carrying interface key + expected shape + stored scalar; stated layer key reuse so bare interface keys are not the injective domain; aligned with fixture README phrasing.
  - Added `## Fact audit — 2026-08-10` dated correction block for the false factorial, model-state wording, map domain, and bind-check precision.
  - Metadata unchanged (status todo; deps/related/scopes graph-correct per report).

Optional items skipped (with reason):
  - none; the recommended bind-check widening was applied as cheap same-ticket prose hygiene.

Residuals not applied (docs/crates/new tickets/authority):
  - Same false `57! · 56!² · 28!⁴` product still in (a) `docs/research/program-planning/complete-model-ingestion-and-execution.md`, (b) `tickets/design-model-ingestion-and-complete-execution.md`, (c) `docs/research/program-planning/model-level-qualification.md`, (d) `tickets/design-model-level-qualification-and-optimization.md` — report asked to file/extend a narrow remainder; no concrete remainder ticket id was supplied, and wave B forbids inventing ids or editing docs/other tickets here. Recorded as blocked residual product debt (cross-site formula re-infection).
  - Undelivered outcome (manifest, header gate, permuted-map failure, altered-digest stop) remains this ticket's open product work — not wave-B scope.
  - Open design choice (manifest "expected stored scalar" = checkpoint BF16 vs post-widen F32) left open as in residual uncertainty; not a false Fact.

Verification:
  - files read:
    - audit report bdf559f18567_c99ac54950f2.md (full)
    - tickets/define-the-model-weight-binding-manifest.md (full pre/post)
    - crates/tiler/src/expansion.rs (BindError variants from bind_region)
    - spikes/program-planning/qwen3-conformance-fixture/README.md (total, injective map from checkpoint tensor name)
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (model-state supersession anchors)
    - rg for remaining `56!²` sites in tickets/
  - checks:
    - table Count 56 appears thrice, Count 28 thrice → exponents ³ and ³
    - no remaining `56!²` on this ticket; corrected product present
    - no "model-state creation" on this ticket
    - shasum -a 256 → post-edit hash above

Recommended next ledger state:
  integrated
