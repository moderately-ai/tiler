Ticket: conform-the-bf16-vertical-end-to-end
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/conform-the-bf16-vertical-end-to-end/fef6243adc11_c99ac54950f2.md
Pre-edit content hash (from ledger): fef6243adc11575afa9b60ca3cbc8183679e727ed1658e6bbc71b5bc7a1b0b43
Post-edit content hash: 053bee1025b23ff6bab15ec37ac0d928ff13532a4041676d803e848333b5ec55

Changes applied:
  - Renamed live type claim `MeasuredHalf::{Ran, Unavailable, Failed}` → `Measured::{Ran, Unavailable, Failed}` on the evidence bullet (type is `Measured<T>` in `measurement.rs`).
  - Added `## Corrected 2026-08-10` covering (1) crate-local `the_unsafe_site_population_is_the_two_named_ones` is historical at `8e995e5c`, live pin is `crates/tiler/tests/workspace_unsafe_sites.rs` / `the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones` with both sites still in `device_buffer.rs`; (2) `MeasuredHalf` → `Measured<T>`; (3) navigation cells delivered via ledger Fact citing this ticket at `b7c01815`; compile/artifact/routing remainder restated as undeclared BF16 contraction on the authoritative profile (not recognizer unowned), with follow-up declared not owed as a hanging bullet on this closed ticket.
  - Status/deps/scopes left unchanged (`done`).

Optional items skipped (with reason):
  - Optional separate ticket for request-path BF16 conformance after ledger BF16 numerical rows: not filed (report: optional; leave no unfiled hanging remainder — stated as not owned by this closed ticket in the correction block).

Residuals not applied (docs/crates/new tickets/authority):
  - none required; Exact files listed ticket only.

Verification:
  - files read:
    - tickets/conform-the-bf16-vertical-end-to-end.md (full, pre/post)
    - audit report fef6243adc11_c99ac54950f2.md (full)
    - crates/tiler-conformance/src/measurement.rs (`pub(crate) enum Measured<T>`)
    - crates/tiler/tests/workspace_unsafe_sites.rs (`the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones`)
    - docs/dtype-support.md (BF16 Backend execution / Conformance evidence Fact citing this ticket)
    - ADR 0079 supersession note for retired crate-local census (via rg)
  - checks:
    - `rg 'MeasuredHalf' tickets/conform-the-bf16-vertical-end-to-end.md` — only inside struck wording of the 2026-08-10 correction
    - `rg 'MeasuredHalf|enum Measured' crates/tiler-conformance` — only `enum Measured`
    - `rg 'the_unsafe_site_population_is_the_two_named_ones' crates/tiler-conformance` — empty (historical name survives only in docs/tickets)
    - `shasum -a 256 tickets/conform-the-bf16-vertical-end-to-end.md` → post-edit hash above

Recommended next ledger state:
  integrated
