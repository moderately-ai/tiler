Ticket: drive-an-external-physical-implementation-provider-through-compilation
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/drive-an-external-physical-implementation-provider-through-compilation/9fca49de2589_c99ac54950f2.md
Pre-edit content hash (from ledger): 9fca49de25890d69044902a3c03198473534e9d9f0706b1226b3a074c6829842
Post-edit content hash: 1c77cbc955657ce3b4878c241776029b121be14b0c681edb2f1209e99a331259

Changes applied:
  - Retitled `## What blocks this today` to `## What blocked this until the Outcome (historical, pre-landing)`; added a historical banner; past-tensed visibility/installation/observability claims so a done-ticket reader cannot treat private modules or missing installation as live work.
  - Outcome fixture census: replaced bare "seven tests" with "seven tests at this landing (`550e3ab6`); nine after disclose added two disclosure tests".
  - Graph maintenance offered/selected note: marked selected-half-only / lowering-only `offered_providers` conflation as true-at-landing; recorded supersession by done disclose ticket's `Compilation::offered_physical_providers` without deleting the historical hand-off.

Optional items skipped (with reason):
  - separate standalone `**Correction — 2026-08-10.**` block: in-place historical reframing of the three required items already discharges the optional dated-note form; no further retirement-by-correction needed.

Residuals not applied (docs/crates/new tickets/authority):
  - none — Repair required lists no docs/crates edits and no new remainder ticket; region-vocabulary / ADR 0090 additivity limitation and `accept-the-installed-physical-provider-public-surface` remain out of this close as already stated.

Verification:
  - files read:
    - tickets/drive-an-external-physical-implementation-provider-through-compilation.md (full, pre/post)
    - audit report 9fca49de2589_c99ac54950f2.md (full)
    - crates/tiler-compiler/tests/external_physical_provider.rs (#[test] census → 9)
    - crates/tiler-compiler/src/session.rs (offered_physical_providers / selected_physical_providers / with_physical_providers)
    - crates/tiler-compiler/src/lib.rs (pub mod physical_provider)
    - crates/tiler-compiler/src/pipeline.rs (compile_with_physical_providers)
    - tickets/disclose-offered-and-selected-physical-provider-sets-separately.md (status: done)
  - checks:
    - fixture has 9 #[test] functions at current tree
    - `Compilation::offered_physical_providers` and `PlanAlternative::selected_physical_providers` both present
    - metadata (status/dependencies/related/scopes) left unchanged per report

Recommended next ledger state:
  integrated
