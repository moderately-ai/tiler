Ticket: remove-the-scalar-lowering-family-from-the-compiler
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-scalar-lowering-family-from-the-compiler/03fa78289deb_c99ac54950f2.md
Pre-edit content hash (from ledger): 03fa78289debe31e4540bc1dbaa5d1e1ad1a382f830b0808bc8f9d602651ca02
Post-edit content hash: abb1e3c89eae46e7e5a505aa48d694bcc749d70e920bca9755f24b199a3f90cb

Changes applied:
  - Outcome reserved-types paragraph: added **Correction — 2026-08-10.** that the claim `tag()` gained a comment recording tag `2` as spent/recyclable is false at source; landed comment only asserts IndexAccess tag `1` is durable and must not be renumbered (no "spent"/"recyclable"/"tag 2" in capability.rs).
  - Outcome evaluation-order / labelled-draft census sentence: added **Correction — 2026-08-10.** that the exclusive "only ScalarArithmetic::new" present-tense census is false after later draft surfaces (physical_provider, tiler-ir gather/law); evaluation-order acceptance itself remains as claimed.
  - Metadata: none (status done, dependencies, related, scopes stand per report).

Optional items skipped (with reason):
  - none (optional labelled-draft census note applied as cheap hygiene on this same ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket's removal population. Out-of-scope ADR 0105 / open-questions debt named in Outcome was already discharged at audit base by later work; no remainder tickets reopened.
  - Workspace nextest counts (2891→2889) and make-full green: historical Outcome measurements not re-run (read-only residual from audit).

Verification:
  - files read: ticket full; audit report full; capability.rs tag doc via rg (Index access is `1` and stays `1`; no spent/recyclable/tag 2); crates/ labelled-draft|awaiting a boundary decision census (physical_provider, target ScalarArithmetic, tiler-ir gather/law).
  - checks: shasum -a 256 of ticket post-edit.

Recommended next ledger state:
  integrated
