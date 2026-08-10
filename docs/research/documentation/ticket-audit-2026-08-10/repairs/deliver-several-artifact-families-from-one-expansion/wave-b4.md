Ticket: deliver-several-artifact-families-from-one-expansion
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/deliver-several-artifact-families-from-one-expansion/fdfc7bb8790e_c99ac54950f2.md
Pre-edit content hash (from ledger): fdfc7bb8790ea41792f2b63f657f46c87b851915472ffca001ad91108440c137
Post-edit content hash: be0a18d8cb5c6fadeb265223fef4bc5201de52b3273e8769645e0045a2fff4a9

Changes applied:
  - Required prose: ledger-source Fact re-anchored to `Both rows are transcribed from` / `2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`; 2026-07-31 path noted as retained lineage only; no-iOS-row substance kept.
  - Required prose: dropped pure line numbers (`line 51`, `line 133`, `numerical-behaviour.md:219`, `docs/artifact-abi.md:385`, `builder.rs:1254`) in favour of phrase anchors (`No iOS family…`, simulator admissibility sentence, `was withdrawn rather than made true`, `TargetProfileMismatch`).
  - Required prose: "### What landed" metal_plan bullet no longer claims the limitation test is "retained"; records replacement by `one_envelope_carries_one_payload_per_artifact_family` and `a_payload_at_another_familys_delivery_position_is_refused`.
  - Required dated correction: `## Fact audit — 2026-08-10` at base `c99ac54950f2` covering ledger rebind, line-number withdrawal, and absent historical limitation test.
  - Metadata: none (status/dependencies/related/scopes left as report required).

Optional items skipped (with reason):
  - none listed as optional beyond related:[] already correct

Residuals not applied (docs/crates/new tickets/authority):
  - none required; remaining product work is this ticket's Closes when after `first-authoritative-ios-metal-compile-declaration` leaves deferred (not a wave-B remainder filing)
  - board style question whether `blocked` vs `todo` while the hard dep is deferred — report left open; status unchanged as required

Verification:
  - files read: full audit report; full ticket; ledger bind sentence + inheritance refusal; numerical-behaviour simulator sentence; artifact-abi withdrawn-sentence paragraph; metal_plan positive/wrong-position tests; builder TargetProfileMismatch site
  - checks: rg for bound 2026-08-02 path and inheritance sentence; metal_plan has `one_envelope…` / `a_payload_at_another…` and no `a_second_artifact_family_cannot_yet_share`; shasum -a 256 post-edit

Recommended next ledger state:
  integrated
