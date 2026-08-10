Ticket: carry-the-elementary-numerical-dimensions-in-the-region-realization
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-the-elementary-numerical-dimensions-in-the-region-realization/766e974fe5db_c99ac54950f2.md
Pre-edit content hash (from ledger): 766e974fe5dbc37413b20d3a7768b33f46034026b02968e1b694c16bbb1c5c98
Post-edit content hash: 59d9c4a7eaedd106d98cf860a277b6618ad46c83fb8a7254c3d8dda98f7e227e

Changes applied:
  - Replaced `session::NumericalContract::RelaxedF32` with `session::NumericalContract::RELAXED_F32` in the refuse-everywhere Fact; also fixed bare `` `RelaxedF32` `` in the Fired trigger to `` `RELAXED_F32` `` (same identifier error).
  - Backend-local Fact now names `precise::exp`, `precise::rsqrt`, and the `/` operator (three-family emission), still recording `MetalNumericalRequirement::PreciseFp32Functions`.
  - Refuse-everywhere consequence now says "no elementary family occurrence" instead of "no activation".

Optional items skipped (with reason):
  - Optional 2026-08-10 dated correction block: skipped; required prose rewritten in place before any worker brief, as the report allows.

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation after Tom accepts the decision packet (NumericalRealization widen, encoders, capability rows, profile declarations, identity rebaseline) — out of wave B1 scope; ticket remains awaiting-decision.
  - Residual design note from the report (region_proposal asymmetry for already-realized dimensions) stays as implementation decision under accepted shape, not a missing ticket.

Verification:
  - files read:
    - tickets/carry-the-elementary-numerical-dimensions-in-the-region-realization.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/carry-the-elementary-numerical-dimensions-in-the-region-realization/766e974fe5db_c99ac54950f2.md
    - crates/tiler-compiler/src/session.rs (grep RELAXED_F32)
    - crates/tiler-metal/src/emit.rs (grep precise::exp / precise::rsqrt / F32Divide / PreciseFp32Functions)
  - checks:
    - `rg RelaxedF32|no activation` on ticket: empty after edit
    - `shasum -a 256` post-edit: 59d9c4a7eaedd106d98cf860a277b6618ad46c83fb8a7254c3d8dda98f7e227e

Recommended next ledger state:
  integrated
