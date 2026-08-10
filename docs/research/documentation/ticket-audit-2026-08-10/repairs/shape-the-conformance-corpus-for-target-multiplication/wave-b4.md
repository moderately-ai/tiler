Ticket: shape-the-conformance-corpus-for-target-multiplication
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/shape-the-conformance-corpus-for-target-multiplication/0054f5809799_c99ac54950f2.md
Pre-edit content hash (from ledger): 0054f58097992ea804f341655576304ccf5b18152ee7f812a0bd6b6249aa05a9
Post-edit content hash: d36dbf60f05d55d2d55a61f62d2f346a7a2343f646e0fa555318cb3d20c2e573

Changes applied:
  - Direction item 2: retired present-tense "from_realization currently has no caller anywhere in crates/" and the non-existent correctness-and-testing heading "What no capability yet checks"; restated live contract (production callers in bf16_vertical::conformance_of and publication proof; residual is Unstated from strict()/new())
  - Dated **Correction — 2026-08-10.** under Direction items documenting the retired no-caller claim; singular-oracle design point unchanged
  - Fact 1 profile-key enumeration: distinguished sole production TargetProfileKey (`tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`), measured/normative producer identities, offline role identities, and test-only `.v2` rekey; singular-family conclusion retained
  - Direction item 1 optional precision: live retained_record six-field compare with xcode deliberately excluded
  - Metadata unchanged (status awaiting-decision; deps; related; scopes)

Optional items skipped (with reason):
  - reverse related edge on publish-the-backend-provider-conformance-suite — optional graph hygiene on another ticket; not load-bearing; wave B edits only this ticket

Residuals not applied (docs/crates/new tickets/authority):
  - Tom still decides Option A vs B (awaiting-decision correctly parked; not wave B product work)
  - post-decision implementation (case model, executor, F32/BF16 population, later provider-suite publish under Option A) remains authorized follow-on, not audit-repair remainder

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/shape-the-conformance-corpus-for-target-multiplication/0054f5809799_c99ac54950f2.md
    - tickets/shape-the-conformance-corpus-for-target-multiplication.md
    - crates/tiler-conformance/src/bf16_vertical.rs (from_realization call)
    - crates/tiler-conformance/src/publication/proof.rs (from_realization call)
    - crates/tiler-conformance/src/retained_record.rs (six-field compare; xcode not compared)
    - crates/tiler-build/src/metal_declaration.rs (profile_key, MEASURED_PRODUCER, NORMATIVE_PRODUCER, offline roles, .v2 rekey)
    - docs/correctness-and-testing.md (Semantic authority; "Each of those clauses is now false"; Unstated residual)
  - checks:
    - rg from_realization under crates/tiler-conformance → production callers at bf16_vertical and publication/proof
    - rg MEASURED_PRODUCER|NORMATIVE_PRODUCER|profile_key|offline- under metal_declaration.rs → identity kinds match repaired Fact 1
    - post-edit ticket no longer asserts present-tense no-caller; carries Correction — 2026-08-10
    - shasum -a 256 of ticket file → d36dbf60f05d55d2d55a61f62d2f346a7a2343f646e0fa555318cb3d20c2e573

Recommended next ledger state:
  integrated
