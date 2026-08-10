Ticket: resolve-semantic-shape-inference-over-symbolic-extents
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/resolve-semantic-shape-inference-over-symbolic-extents/69cda00f02eb_c99ac54950f2.md
Pre-edit content hash (from ledger): 69cda00f02eb0a77928f0c011f2c535108ddd7c1508a30d0b28a9d248fe307fc
Post-edit content hash: 7de717a4070782885f2ace6930f5603fa79fe2a8930d730fd41a27e154c2f5c3

Changes applied:
  - Dated **Correction — 2026-08-10** under the Unsupported-cases bullet that claimed `OBLIGATION_DOMAIN` and peer identity domains were unguarded/unpinned: present-tense claim retired; 3181-pass measurement retained as landing-time historical; pins now live in `crates/tiler-ir/src/domains.rs` via done ticket `pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate` for obligation v2, graph v3, and index-region v11; introducing those IR domain pins is no longer left to Tom.
  - Optional: tightened the absolute "nothing outside `tiler-ir` constructs a symbolic program" clause to production/out-of-crate providers while noting `tiler-artifact` test fixture construction via `try_standard_with_shape_environment` + `input_sourced`.

Optional items skipped (with reason):
  - none (optional construction wording was cheap same-ticket prose and was applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave; Tom public-boundary acceptance remains open on the Decision packet (status correctly stays `awaiting-decision`); no crates/docs edits; no new remainder tickets.

Verification:
  - files read:
    - full audit report `docs/research/documentation/ticket-audit-2026-08-10/reports/resolve-semantic-shape-inference-over-symbolic-extents/69cda00f02eb_c99ac54950f2.md`
    - full ticket `tickets/resolve-semantic-shape-inference-over-symbolic-extents.md`
    - `crates/tiler-ir/src/domains.rs` (PINNED_IDENTITY_DOMAINS includes the three named spellings)
    - `tickets/pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md` status `done`
    - `crates/tiler-artifact/src/program/tests.rs` constructs symbolic program via `try_standard_with_shape_environment`
  - checks:
    - status/dependencies/related/scopes left unchanged (report: none required)
    - shasum -a 256 of ticket after edit → 7de717a4070782885f2ace6930f5603fa79fe2a8930d730fd41a27e154c2f5c3

Recommended next ledger state:
  integrated
