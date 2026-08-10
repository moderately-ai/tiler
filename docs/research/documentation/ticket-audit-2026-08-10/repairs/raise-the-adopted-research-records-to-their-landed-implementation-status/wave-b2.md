Ticket: raise-the-adopted-research-records-to-their-landed-implementation-status
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/raise-the-adopted-research-records-to-their-landed-implementation-status/84b7844f2c2e_c99ac54950f2.md
Pre-edit content hash (from ledger): 84b7844f2c2e9a440d700d95a10be2aebd68143ff565b3c78d3983b596a7e4b2
Post-edit content hash: 5ed01fb1b602e0fafa7cc348d1b7a22f49cc1062ca99a9457d299a49bc354791

Changes applied:
  - Added `## Outcome — 2026-08-05` recording landing `63b067ba`, all six frontmatters `spike-only` → `partial`, each with `## Implementation status` derived from the record's own decided behaviour and named crates (not adopting ADR status alone), none left at `spike-only`, and close commit `cc6a7fd5` status-only flip.
  - Added `**Correction — 2026-08-10.**` under Why this exists marking the six-record `spike-only` Fact as filing-time problem statement, with reproduce command for current `adopted`/`partial` frontmatter.
  - Corrected production-code table cell for `semantic-validation-enforcement.md` from sole `crates/tiler-runtime/` to primary `crates/tiler-ir/src/semantic/` (precondition/conformance) with `RoutingCommit`/`Preflight` in `crates/tiler-runtime/src/load/route.rs`.
  - Sharpened operation-extension-surface production-code cell from "the registered operation-extension surface" to `crates/tiler-ir/src/semantic/registry.rs`.
  - Left status/dependencies/related/scopes unchanged (`done`; related and scopes already correct per report).

Optional items skipped (with reason):
  - none (optional registry path sharpen applied; no other optional bullets).

Residuals not applied (docs/crates/new tickets/authority):
  - Line-number drift inside the six research `## Implementation status` sections (e.g. `ScheduledRegion`, `SemanticPreconditionStatus` line citations) — residual documentation drift on already-landed records; report says not repaired here; Exact files for this wave are ticket-only.
  - Thirteen other adopted/partially-adopted research records still at `spike-only` — outside this ticket's named six and Closes when; report forbids opening a remainder on this ticket for that population.

Verification:
  - files read:
    - entire audit report `…/84b7844f2c2e_c99ac54950f2.md`
    - entire ticket (pre- and post-edit)
    - Implementation status anchors in `docs/research/runtime/semantic-validation-enforcement.md` and `docs/research/extensions/operation-extension-surface.md`
  - checks:
    - `rg -n '^(disposition|implementation_status):'` on the six named research paths → all `adopted` / `partial`
    - `git log` on `63b067ba` → 2026-08-05, message "Let six adopted records say which of their own behaviours shipped"
    - ticket contains `## Outcome — 2026-08-05`, dated correction, corrected table cells
    - `shasum -a 256` ticket → `5ed01fb1b602e0fafa7cc348d1b7a22f49cc1062ca99a9457d299a49bc354791`

Recommended next ledger state:
  integrated
