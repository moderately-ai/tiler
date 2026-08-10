Ticket: carry-the-device-executed-value-proof-into-the-conformance-crate
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/carry-the-device-executed-value-proof-into-the-conformance-crate/dcb8945b5ffb_c99ac54950f2.md
Pre-edit content hash (from ledger): dcb8945b5ffb7778b8a2f6750c999dc95f71eacf7bf166f59d642a44a96f4f77
Post-edit content hash: f7a504f7ba6d5168ce8b697fd0fb0d0b4e69a9f3ef7c5932f86f0913851cad60

Changes applied:
  - Outcome landing census: dated "17 → 47 tests" as 2026-08-07 / `0f948637` only; not a live count.
  - Outcome pins: rephrased "Pins unmoved" as landing-time only; **Correction — 2026-08-10.** with live metal_plan pins `39e765637a7e014a…` / `7e00d9fa0ce90749…` / 65_313.
  - Outcome unmet-obligations heading: "at landing"; ambient-input / `TILER_CONFORMANCE_ARTIFACT_BASE` paragraph past-tensed; **Correction — 2026-08-10.** that produce-the-conformance-envelope… is done and the ambient input is retired via publication/envelope.
  - Outcome lint-drift risk: past-tense restatement risk; **Correction — 2026-08-10.** that stop-the-conformance-crate-s-lint-table… is done and "first member" is false (prototype diverged first).
  - status/metadata left unchanged (done correct; no dependency edge missing).

Optional items skipped (with reason):
  - none — optional "17 → 47" dating applied as cheap house-style hygiene on the same Outcome block.

Residuals not applied (docs/crates/new tickets/authority):
  - Loader fixture compiler-free rewrite onto adapter_route assembler remains a possible separate ticket if desired; not required for this repair (report: not a false close).
  - Prototype retirement still Tom's fork — not a repair of this ticket.
  - Worker notes 2026-08-07 ambient-input narrative left as dated landing history (report Repair required scoped to Outcome unmet-obligations present tense).
  - No docs/crates edits (wave B ticket-only).

Verification:
  - files read:
    - audit report dcb8945b5ffb_c99ac54950f2.md (full)
    - tickets/carry-the-device-executed-value-proof-into-the-conformance-crate.md (full, pre and post)
    - crates/tiler-conformance/src/publication.rs (header: ambient input used to gate; in-process publish)
    - crates/tiler-conformance/src/envelope.rs (TILER_CONFORMANCE_ARTIFACT_BASE retired)
    - crates/tiler-build/src/metal_plan.rs (ARTIFACT_IDENTITY / CACHE_SUBJECT / FIXED_CONTENT_BYTES)
    - tickets/produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate.md (status: done)
    - tickets/stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace.md (status: done; first-member correction)
  - checks:
    - shasum -a 256 tickets/carry-the-device-executed-value-proof-into-the-conformance-crate.md → f7a504f7ba6d5168ce8b697fd0fb0d0b4e69a9f3ef7c5932f86f0913851cad60
    - no present-tense "still gated on TILER_CONFORMANCE_ARTIFACT_BASE" as live Outcome claim
    - no live "Pins unmoved 7a2bfe51… / 65,294" without landing-time / correction framing
    - "first member" claim corrected in Outcome lint-risk paragraph

Recommended next ledger state:
  integrated
