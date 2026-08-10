Ticket: site-the-governed-digest-so-layered-identity-encoders-can-reach-it
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/site-the-governed-digest-so-layered-identity-encoders-can-reach-it/d155db325690_c99ac54950f2.md
Pre-edit content hash (from ledger): d155db325690b9bdf52eb9fa14113dc55d0546139c700dca887ca9063b8e1f28
Post-edit content hash: 91e450c4b1cb36c117055ca84f09872ff78bd7d567a3026ef48b31c694a39b24

Changes applied:
  - Outcome relocation-sweep `docs/artifact-abi.md` bullet: added **Correction — 2026-08-10.** retiring the live-as-stated `tiler.artifact-` vs `tiler.ir.` quantifier; standing argument is `tiler.artifact` or `tiler.proof-sidecar.` vs `tiler.ir.` (hyphen never covered e.g. route-requirement); linked same defect class as correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix; reproduce command against docs/artifact-abi.md.

Optional items skipped (with reason):
  - Past-tense framing of "Why this exists" opening Facts — report polish only; not required for live Facts.
  - New remainder ticket for the two test-local hand-written SHA-256 helpers (contraction_conformance.rs, contraction_profile_cells.rs) — report marks optional; status done does not require the edge; Outcome observation already states out-of-scope and "worth a narrow ticket" without inventing product scope.

Residuals not applied (docs/crates/new tickets/authority):
  - None required. Report Exact files: ticket only for the required prose correction. No docs/crates product residual from this repair. Optional second-hashing remainder remains unfiled (see optional skip).

Verification:
  - files read: audit report; full ticket; docs/artifact-abi.md no-prefix paragraph (opens `tiler.artifact` or `tiler.proof-sidecar.`); tickets/correct-the-coverage-graph-digest-domain-s-eight-count-and-hyphenated-artifact-prefix.md (defect class); sample wave repair note shape.
  - checks: `shasum -a 256` on ticket post-edit; `rg` confirms Correction and retired `tiler.artifact-` wording retained in historical Outcome phrase; standing artifact-abi spelling still present.

Recommended next ledger state:
  integrated
