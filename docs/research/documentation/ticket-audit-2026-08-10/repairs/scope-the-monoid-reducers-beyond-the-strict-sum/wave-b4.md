Ticket: scope-the-monoid-reducers-beyond-the-strict-sum
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-monoid-reducers-beyond-the-strict-sum/8c9548a7f3ce_c99ac54950f2.md
Pre-edit content hash (from ledger): 8c9548a7f3ce2ba46283ec19371202e0f683509383d35aef8e9f33aab4b21d2d
Post-edit content hash: 255bd5155d70ce812bfae148252804c4421e9bccb759acb1fbebd060849cac8d

Changes applied:
  - In `## What the work would be, when it starts`, replaced the causal clause that `empty_identity_bits` is absent "because no binary32 value is an identity for `Maximum`" with wording matching `ScalarProgram::StrictSerialMaximum` docs: empty-domain result never declared (no field; refuse empty domain), while padding neutrality of `-inf` is a separate fact that neither supplies nor weakens that declaration.

Optional items skipped (with reason):
  - Optional one-line dated correction under Facts/work noting matrix/ADR-boundary absolute identity wording refined by schedule docs — not required once the work-section sentence is fixed (report: "not required if the work-section sentence is simply fixed").

Residuals not applied (docs/crates/new tickets/authority):
  - Matrix row still opens with ADRs 0012/0022/0023/0025 as `implementation_status: not-started` while ADR 0012 and ADR 0022 are `partial` — matrix/ADR-boundary hygiene, out of ticket edit surface (report residual).
  - ADR 0022 implementation-boundary absolute "no binary32 value is neutral" wording and rotted line citation — docs residual, not this ticket.

Verification:
  - files read:
    - tickets/scope-the-monoid-reducers-beyond-the-strict-sum.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-monoid-reducers-beyond-the-strict-sum/8c9548a7f3ce_c99ac54950f2.md
    - crates/tiler-ir/src/schedule/model.rs (StrictSerialMaximum empty-domain / padding-neutrality doc block; `That refusal is not the claim that no binary32 value is neutral for this fold`; `0xff80_0000`)
  - checks:
    - shasum -a 256 tickets/scope-the-monoid-reducers-beyond-the-strict-sum.md → 255bd5155d70ce812bfae148252804c4421e9bccb759acb1fbebd060849cac8d
    - grepped model.rs for empty-domain identity omission vs padding identity of `-inf`

Recommended next ledger state:
  integrated
