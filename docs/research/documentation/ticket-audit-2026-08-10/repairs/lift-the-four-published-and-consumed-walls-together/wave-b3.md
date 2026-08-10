Ticket: lift-the-four-published-and-consumed-walls-together
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/lift-the-four-published-and-consumed-walls-together/0730d1e4d09f_c99ac54950f2.md
Pre-edit content hash (from ledger): 0730d1e4d09f06b811067006d47bb0c03ce3ebfa3866e263bc2ada511b837bc6
Post-edit content hash: 8bbffb5b40c191d23303c818fe4a93b37172f12362bea824de97ebbc452001ee

Changes applied:
  - Struck false present-tense `UncoveringStage` "two *declared accounts*" claim; **Correction — 2026-08-10.** records two accounts at this ticket's land and three live accounts after staged realization.
  - Marked the gate-prose Proposal **discharged** (integrator/carrier landed published-and-consumed row in `docs/correctness-and-testing.md`); retained drafting as close-time handoff only.
  - Struck false present-tense accept-node park at `awaiting-decision`; **Correction — 2026-08-10.** records `accept-the-kernel-program-publishing-copy-surface` `status: done` (Tom 2026-08-06), Accepted boundary labels, and `docs/ir.md` acceptance paragraph.
  - Added `### Audit correction — 2026-08-10` consolidating: accept closed, gate-prose discharged, `PROGRAM_DOMAIN` live `v11` with Outcome pin "New" as v10 intermediate ledger values, UncoveringStage three accounts.

Optional items skipped (with reason):
  - none (optional UncoveringStage three-account note applied with the required correction block).

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report Exact files expected only this ticket; metadata unchanged; no new remainder ticket; no docs/crates product path edits in this wave.

Verification:
  - files read:
    - tickets/lift-the-four-published-and-consumed-walls-together.md (pre- and post-edit)
    - audit report 0730d1e4d09f_c99ac54950f2.md
    - tickets/accept-the-kernel-program-publishing-copy-surface.md (status: done)
    - crates/tiler-ir/src/program/model.rs (PROGRAM_DOMAIN v11)
    - crates/tiler-ir/src/program/verify.rs (`This profile admits exactly three accounts`)
    - docs/correctness-and-testing.md (published-and-consumed / multi-output gate prose presence)
  - checks:
    - accept ticket frontmatter `status: done`
    - `const PROGRAM_DOMAIN: &[u8] = b"tiler.kernel-program.v11\0";` in model.rs
    - three-account doc on `verify_stage_accounts`
    - multi-output paragraph still names published-and-consumed flip material
    - post-edit sha256 via `shasum -a 256`

Recommended next ledger state:
  integrated
