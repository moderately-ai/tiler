Ticket: restore-an-executable-artifact-assembly-example
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/restore-an-executable-artifact-assembly-example/6b2e46e8a6ac_c99ac54950f2.md
Pre-edit content hash (from ledger): 6b2e46e8a6acb7791c722bb2fb5678959c21e787dbe48394bddb9c5435b48354
Post-edit content hash: 363169993b926c2b96474af6ae7f03c0794f26067637a44b59a5f80859d5f0a6

Changes applied:
  - ## Fact — outcome, per example: rewrote bare StrictSerialSum rationale — serial non multi-pass admits ContributorTensor::DeclaredDomain (Intermediate **or** first input); not chosen because reduction topology is longer than PointwiseF32 with two interface inputs; FusedMultiplyAddSerialSum still noted as five-op fixture territory
  - **Correction — 2026-08-10.** under that paragraph marks Intermediate-only refusal as retired wording (multi-pass Final only) and cites live DeclaredDomain.admits rule
  - Discharged close check: replaced stale `builder.rs:1922` with anchor "Ordinary transactional call site" in index/builder.rs; dated correction records the drifted line number so greps for `:1922` land on the marker
  - Optional population-table hygiene: dated note that `:164`/`:83`/`:104` were coverage-binding-commit inventory, not current base; live ignore population is the one unrelated builder illustration
  - Metadata unchanged (status done, deps [], related, scopes stand)

Optional items skipped (with reason):
  - none (optional population-table line-number note applied as cheap hygiene on this ticket)

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required ticket prose only; no docs/crates edits; no remainder tickets

Verification:
````text
  - files read: full audit report; full ticket pre-edit; schedule/builder.rs DeclaredDomain => Intermediate || FIRST_INPUT; StrictSerialSum serial arm uses DeclaredDomain; multi_pass Final Exactly(Intermediate); sole crates/ ```ignore at index/builder.rs Ordinary transactional call site
````
  - checks: shasum -a 256 post-edit; present-tense Intermediate-only refusal no longer live claim; 1922 only inside dated correction; metadata unaltered

Recommended next ledger state:
  integrated
