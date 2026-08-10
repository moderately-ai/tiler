Ticket: replace-the-digest-note-s-restated-domain-count-with-its-type
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/replace-the-digest-note-s-restated-domain-count-with-its-type/9642f5a29aaa_c99ac54950f2.md
Pre-edit content hash (from ledger): 9642f5a29aaa72a581c1a42f433062738759285dac3c42dc5e46ce0998cb81ed
Post-edit content hash: 9531ae55948b3b6457e05ccaf4d167337dbe5b04e81e8c102444ca1a1c92c5e5

Changes applied:
  - Refreshed Beware/history anchors from stale `so the check covered 8 of 11` to live `it covered 8 of 11` and noted parallel `it covered 8 of 18` with scope in domains.rs header.
  - Added **Correction — 2026-08-10** that worker-audit "8 of 18 … not in any tracked file at this base" is scoped to `670e7a31` only; post-disambiguation trees carry both figures.
  - Replaced false "Two of the twelve … metal-aot and digest" census with three direct non-`tiler-ir` deps: `tiler-metal-aot`, `tiler-digest`, `tiler-cache`; dated correction records the omission.

Optional items skipped (with reason):
  - Formal Outcome naming `8fda6b34` / `bdb085c9`: optional board convention only; status stays done and primary close condition already landed; not required by Repair required bullets.

Residuals not applied (docs/crates/new tickets/authority):
  - None for production code (report: no remainder tickets; Exact files is ticket-only).
  - Full independent re-grade of all 21 neighbouring header claims remains residual uncertainty from the audit, not a ticket prose fix.

Verification:
  - files read:
    - tickets/replace-the-digest-note-s-restated-domain-count-with-its-type.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/replace-the-digest-note-s-restated-domain-count-with-its-type/9642f5a29aaa_c99ac54950f2.md
    - crates/tiler-artifact/src/domains.rs (header: `it covered 8 of 11`, `it covered 8 of 18`)
    - crates/*/Cargo.toml tiler-ir dep scan; crates/tiler-cache/Cargo.toml (only tiler-artifact)
  - checks:
    - `rg 'it covered 8 of 11|it covered 8 of 18' crates/tiler-artifact/src/domains.rs` hits both
    - three non-ir crates among twelve others: cache, digest, metal-aot
    - ticket no longer contains live "Two of the twelve" as the asserted census
    - post-edit sha256: 9531ae55948b3b6457e05ccaf4d167337dbe5b04e81e8c102444ca1a1c92c5e5

Recommended next ledger state:
  integrated
