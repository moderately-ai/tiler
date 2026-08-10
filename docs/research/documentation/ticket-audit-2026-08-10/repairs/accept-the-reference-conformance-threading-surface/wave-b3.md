Ticket: accept-the-reference-conformance-threading-surface
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-reference-conformance-threading-surface/6aa331eecc9f_c99ac54950f2.md
Pre-edit content hash (from ledger): 6aa331eecc9ff0a6aabcbc545ff179ede6794a0dda6a617dee1a9971a21084e5
Post-edit content hash: eabb56e7db89d6ffba19d428025161cb2a6e7b738631fa469eac43185dc7295a

Changes applied:
  - Replaced live parking framing ("Only Tom closes… parks at `awaiting-decision`") with closed-after-acceptance wording; parking state kept as historical open-period context only.
  - Corrected "eleven" → "twelve" additive public items (list unchanged; count matches the twelve named items).
  - Annotated that acceptance-day `ReferenceEvaluationRequest::conformance` was later replaced by `conformance_for(ArithmeticType)` under `give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`, under the ticket's non-stabilization clause.
  - Added **Correction — 2026-08-10.** and short `## Fact audit — 2026-08-10` so the acceptance-day inventory remains reconstructible.
  - Left frontmatter unchanged (`status: done`, empty dependencies/related, `scopes: [contracts/decisions]`).

Optional items skipped (with reason):
  - none; the report's optional inventory annotation (item 3 under prose) was applied because it is load-bearing for grepping the accepted inventory and is cheap same-ticket prose.

Residuals not applied (docs/crates/new tickets/authority):
  - none; Exact files expected only this ticket; no remainder ticket; no docs/crates edits.

Verification:
  - files read:
    - tickets/accept-the-reference-conformance-threading-surface.md (pre/post)
    - reports/.../6aa331eecc9f_c99ac54950f2.md (full)
    - tickets/give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md (conformance → conformance_for replacement claim)
    - crates/tiler-reference/src/{evaluate,contraction,registry,silu,rms_norm,softmax}.rs (grep for public under/conformance/_under surface)
  - checks:
    - twelve named items: ReferenceEvaluator::{under,conformance}, Request::conformance (historical inventory name), strict_partial_sums_under, strict_partitioned_sum_under, five family `*_f32_under`, StagedStrictTensorContractionF32::{under,conformance}
    - live Request public accessor is `conformance_for` at registry.rs; no public `fn conformance` on ReferenceEvaluationRequest
    - `shasum -a 256` post-edit hash recorded above

Recommended next ledger state:
  integrated
