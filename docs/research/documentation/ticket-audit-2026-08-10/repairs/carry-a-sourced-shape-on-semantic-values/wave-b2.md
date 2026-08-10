Ticket: carry-a-sourced-shape-on-semantic-values
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/carry-a-sourced-shape-on-semantic-values/2c8551c739b6_c99ac54950f2.md
Pre-edit content hash (from ledger): 2c8551c739b67ed4dc3be5269bcf5377c489aa487411d22d9d9ef037638eec8c
Post-edit content hash: 20e1c24f0535baadca04a78bc25f58701ebe2c6a89a1b27400a63f6cac7c4999

Changes applied:
  - Opening "## Why this exists" Fact struck as a live claim with **Correction — 2026-08-10.**: filing-time gap only; live tree carries `SourcedShape` on `ValueFact`/`ValueData`, `ValueDefinition` never held a shape, `SemanticProgram::shape` returns `Result<&SourcedShape, HandleError>`
  - Same correction notes Outcome pin table and landing-time `ValueFact::shape` → `&Shape` boundary are historical, not tip-of-tree

Optional items skipped (with reason):
  - none (optional landing-vs-tip Outcome note applied inside the dated correction)

Residuals not applied (docs/crates/new tickets/authority):
  - none required for this ticket's delivered unit; residual public-boundary acceptance remains Tom's (ADR 0075); further symbolic result-inference acceptance lives on `resolve-semantic-shape-inference-over-symbolic-extents`

Verification:
  - files read: full audit report; full ticket; greps on `crates/tiler-ir/src/semantic/{operation,program,precondition,conformance}.rs` confirming `ValueFact`/`ValueData` hold `SourcedShape`, `ValueDefinition` is Input|OperationResult only, `SemanticProgram::shape` → `Result<&SourcedShape, HandleError>`, `ValueFact::shape`/`ValueRef::shape` → `&SourcedShape`
  - checks: shasum -a 256 post-edit ticket; status/dependencies/related/scopes left unchanged (report: none)

Recommended next ledger state:
  integrated
