Ticket: decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access/ad90d8594fc2_c99ac54950f2.md
Pre-edit content hash (from ledger): ad90d8594fc2d13733e81424ce7e407e0e592c44d2ed425581677257d828eb7f
Post-edit content hash: 0e11a4d8353086026bd9e4bb965c8c995c5407562be85f588a0bc8fd4aaca922

Changes applied:
  - Removed tag `needs-tom`; left `status: done`.
  - Reframed opening decision prose as filing-time: inherit + workspace `forbid` problem statement; dated **Correction — 2026-08-10** that the crate restates lints with `unsafe_code = "deny"` (no workspace inheritance).
  - Past-tense "only construction" Fact; dated **Correction — 2026-08-10** that `device_buffer.rs` also crosses the device boundary; named `write_bytes` / `read_bytes` with host→device and device→host roles.
  - Noted admit-era open-question manifest comment is historical; decision text is live.
  - Optional Closes-when discharge: dated correction under Decided obligations naming the two sites, what each does, and that the open-question comment obligation is already discharged in tree.

Optional items skipped (with reason):
  - none (optional site-population one-liner applied as cheap same-ticket hygiene discharging Closes when).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report (Exact files: ticket only; no remainder ticket; safe-wrapper-elsewhere stays intentionally undecided architecture).

Verification:
  - files read:
    - tickets/decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access/ad90d8594fc2_c99ac54950f2.md
    - crates/tiler-conformance/Cargo.toml (deny + Decided comment; no lint workspace inherit)
    - crates/tiler-conformance/src/device_buffer.rs (write_bytes / read_bytes population)
    - prototypes/serial-sum-run/Cargo.toml (deny)
  - checks:
    - `rg` on conformance Cargo.toml: `unsafe_code = "deny"`, Decided comment present
    - `rg` on device_buffer.rs: `write_bytes` / `read_bytes` sites
    - post-edit sha256: 0e11a4d8353086026bd9e4bb965c8c995c5407562be85f588a0bc8fd4aaca922
    - ticket no longer carries `needs-tom`

Recommended next ledger state:
  integrated
