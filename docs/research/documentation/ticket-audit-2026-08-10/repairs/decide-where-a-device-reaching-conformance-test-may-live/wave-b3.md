Ticket: decide-where-a-device-reaching-conformance-test-may-live
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-where-a-device-reaching-conformance-test-may-live/5a0176966fd9_c99ac54950f2.md
Pre-edit content hash (from ledger): 5a0176966fd9ff5e30dd71747ca0843793ee1693fc972bfd1688a4a85d23ce20
Post-edit content hash: 9edc3a00c029f10b32762a115ec35d798168fd2dfde673dad4eda0011d0f3975

Changes applied:
  - After the present-tense "exactly one workspace member can reach a device" Fact under "Why this node exists", added **Superseded — 2026-08-10.** retaining the pre-decision census as historical context, naming `crates/tiler-conformance` as the decided gated home, and noting the live pure-`metal` census (workspace pin, serial-sum-run, tiler-conformance, spikes) plus greppable-pattern false positives on `tiler-metal.workspace`.
  - Removed residual `needs-tom` tag from frontmatter on this closed decision (cheap label hygiene).
  - Added **Note — 2026-08-10.** under the 2026-08-07 scope-set correction: the three-byte-identical census is pinned to `3e0074d5`; `retain-the-selected-semantic-candidate-for-the-conformance-oracle` has since gained `contracts/decisions` and is `awaiting-decision`.

Optional items skipped (with reason):
  - none (both optional bullets applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report Exact files expected only this ticket; no docs/crates remainder and no new remainder ticket. Decision outcome already durable in ADR 0106 / admit ticket.

Verification:
  - files read:
    - tickets/decide-where-a-device-reaching-conformance-test-may-live.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-where-a-device-reaching-conformance-test-may-live/5a0176966fd9_c99ac54950f2.md (full)
    - tickets/retain-the-selected-semantic-candidate-for-the-conformance-oracle.md (frontmatter: scopes + status)
  - checks:
    - pure-`metal` Cargo.toml census returns workspace pin, serial-sum-run, tiler-conformance, spikes
    - retain ticket frontmatter: `scopes: [implementation/compiler, contracts/decisions]`, `status: awaiting-decision`
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
