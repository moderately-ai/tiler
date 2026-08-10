Ticket: derive-the-payload-carrying-enum-populations-in-the-injectivity-module
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-payload-carrying-enum-populations-in-the-injectivity-module/18f6d636c703_c99ac54950f2.md
Pre-edit content hash (from ledger): 18f6d636c703b072a60a3feb0cc9bd773c898c4331f4f3c64713abb5bbaf63ed
Post-edit content hash: 18f6d636c703b072a60a3feb0cc9bd773c898c4331f4f3c64713abb5bbaf63ed

Changes applied:
  - none (report: exact metadata none; exact dated correction none; ticket file none required for board correctness; status/deps/related/scopes stay)

Optional items skipped (with reason):
  - Optional one-line Outcome residual note on this ticket if a repair ticket is opened — not mandatory; no new remainder id filed in wave B, so no note added.
  - Optional treatment of residual `numerics/tests.rs` rustdoc as out-of-scope adjacent debt vs in-scope prose close condition — wave B3 is ticket-only, so crates residual is recorded below rather than edited.

Residuals not applied (docs/crates/new tickets/authority):
  - `crates/tiler-ir/src/numerics/tests.rs` rustdoc on `every_behaviour_round_trips_and_consumes_exactly_its_own_width`: replace the clause that claims every one of the five spaces is walked "from its own variant count" with wording that matches `all_behaviours` (three fieldless `variant_count` spaces + two exhaustive outer-arm census sums). Anchor still live: ``[`all_behaviours`] walks from its own variant count. Twelve values, so all 66``. Wave B3 is ticket-only; crates path out of scope. Report marks this optional if treated as adjacent micro-debt; implementation close is otherwise solid.

Verification:
  - files read:
    - audit report 18f6d636c703_c99ac54950f2.md (full)
    - tickets/derive-the-payload-carrying-enum-populations-in-the-injectivity-module.md (full)
    - crates/tiler-ir/src/numerics/tests.rs (rg around all_behaviours / every_behaviour_round_trips)
  - checks:
    - residual anchor still present at `every_behaviour_round_trips_and_consumes_exactly_its_own_width` rustdoc
    - frontmatter status `done`, deps `[]`, related `[derive-the-artifact-numerical-and-fenced-space-populations]`, scopes unchanged
    - post-edit `shasum -a 256` ticket → 18f6d636c703b072a60a3feb0cc9bd773c898c4331f4f3c64713abb5bbaf63ed (unchanged; no ticket prose required)

Recommended next ledger state:
  integrated
