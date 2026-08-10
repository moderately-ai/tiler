Ticket: widen-the-strategy-recognizer-past-the-f32-wall
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/widen-the-strategy-recognizer-past-the-f32-wall/39712c49a962_c99ac54950f2.md
Pre-edit content hash (from ledger): 39712c49a962aa99dc2e868de28a40587b75e624171afebdd79f504d38fa457b
Post-edit content hash: 74c6158944d3a27bd956b7a6d310900b6d6b207dc0e4e9dd46217b341f3c2a47

Changes applied:
  - Opening problem-statement Fact: marked historical (retired by Outcome); removed stale line citations (`request.rs:4206`, wall test lines) in favour of searchable anchors (`fn select_supported_strategy(`, delivery-time wall test names).
  - Mid-Outcome "shape is one occurrence" paragraph: past-tense delivery framing; dangling test name kept as delivery-time only; **Correction — 2026-08-10.** that multi-occurrence fusion wall is gone after establish-bf16-optimizer-legality, live successor `a_multi_occurrence_bf16_program_derives_its_own_fusion_legality`, narrower wall `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall`.
  - `bf16_scheduled_region` re-foundation sentence: dated correction that fusion boundary no longer keeps the fixture hand-assembled; live reason is stated realization.
  - Identity Fact: framed as delivery-time snapshot at `0b0b4bed`; **Correction — 2026-08-10.** live explain pin `request=7ba3d77a66f04638` and live `FIXED_CONTENT_BYTES = 65_313`.
  - "What was not done" → "at delivery"; struck live "one remaining boundary"; dated correction that legality and conformance successors are done.
  - Delivered Outcome identity / fourth-site sentences: delivery-time framing + dated pin and fusion corrections.
  - Section "The boundary that survives" → "survived at delivery"; present-tense wall claim past-tensed; delivery-time test name labelled; **Correction — 2026-08-10.** full fusion-legality survival strike with successor test and no-reopen.
  - Metadata unchanged (status done, deps empty, related four, scopes).

Optional items skipped (with reason):
  - none (optional pin framing and line-citation hygiene applied as cheap same-ticket prose).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report (exact new remainder: none; docs/crates not in wave B ticket scope).

Verification:
  - files read: full audit report; full ticket pre-edit; greps for successor test names and FusionNumericalCapabilities BF16 rows under crates; live explain pin and FIXED_CONTENT_BYTES in explain.rs / metal_plan.rs; sibling correction ticket style samples.
  - checks: shasum -a 256 post-edit; rg confirms no present-tense "still stops" / live dangling-only citation without delivery framing; successor fn name present in crates/tiler-compiler/tests/bf16_numerical_contract.rs.

Recommended next ledger state:
  integrated
