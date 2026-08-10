Ticket: ingest-the-checkpoint-as-f32-program-inputs
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/ingest-the-checkpoint-as-f32-program-inputs/dbf33aab3e06_c99ac54950f2.md
Pre-edit content hash (from ledger): dbf33aab3e0608d89ad8e41c003207734acd5cb24ccc512884cd61a4883f74a5
Post-edit content hash: 90696eaa2a72ee7f93c8a3f48c83ba972ba67c0efcc87660e062b121428eaf8a

Changes applied:
  - Decision paragraph: replaced "on every token" with pass-based wording ("on every forward pass (once per layer-program execution)") consistent with the 252 / nine-pass arithmetic and L6's "once per forward pass" sentence.
  - Required content: added non-finite census (NaN + infinite, refuse when non-zero) and subnormal count on the same digests pass; noted pinned-revision measured zeros; clarified that fixture `weights.widened.sha256` is ticket-9 oracle surface and does not close this ticket — consumer load path must produce and gate digests.
  - Closes when: require retained non-finite and subnormal counts, non-finite guard watchable on a substituted fixture, and consumer-path widened digest (fixture host.tsv alone insufficient).
  - Dated correction — 2026-08-10: records prior understatement of non-finite/subnormal obligations and the "every token" imprecision; notes L6 I-A residual phrasing outside this ticket's scopes.
  - Metadata left unchanged (status, deps, related, scopes already graph-correct per audit).

Optional items skipped (with reason):
  - Coordinated edit of L6 I-A "on every token" in complete-model-ingestion-and-execution.md — out of ticket scopes for wave B; noted in the dated correction and residuals.

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/program-planning/complete-model-ingestion-and-execution.md I-A still says "on every token" (optional coordinated research-record repair; lower severity because 252 figure remains correct).
  - Product delivery (new prototypes member, Cargo.toml/Cargo.lock, ticketsplease.toml scope entry, consumer README/gitignore, watched StorageScalarMismatch and non-finite/subnormal/digest gates) remains undelivered; status stays `todo`.

Verification:
  - files read:
    - tickets/ingest-the-checkpoint-as-f32-program-inputs.md (pre + post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/ingest-the-checkpoint-as-f32-program-inputs/dbf33aab3e06_c99ac54950f2.md
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (I-A "on every token" / "once per forward pass")
    - docs/research/program-planning/model-level-qualification.md (L8 ownership of non-finite check; weights.widened.*)
  - checks:
    - rg confirmed L6 I-A still carries "on every token" and follow-on "once per forward pass — nine times"
    - rg confirmed L8 assigns non-finite check ownership to this ticket id
    - shasum -a 256 post-edit ticket → 90696eaa2a72ee7f93c8a3f48c83ba972ba67c0efcc87660e062b121428eaf8a

Recommended next ledger state:
  integrated
