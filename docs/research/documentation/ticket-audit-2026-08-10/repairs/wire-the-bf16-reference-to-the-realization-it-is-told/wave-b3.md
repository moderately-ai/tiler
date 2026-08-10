Ticket: wire-the-bf16-reference-to-the-realization-it-is-told
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/wire-the-bf16-reference-to-the-realization-it-is-told/c81b695e3656_c99ac54950f2.md
Pre-edit content hash (from ledger): c81b695e3656e4fc01411c99d33d3f2bdb2bae2944df14c2efe864d11ec79bf1
Post-edit content hash: 24a52c371aeb40a59a2c2733a0098c963eba4de27706f211ad1fa29d4b09fca3

Changes applied:
  - related: added give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject and route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge for residual traceability (both status done).
  - **Correction — 2026-08-10.** under Worker outcome "Left open, deliberately": subject-check residual closed on those two tickets; evaluate now uses conformance_for(ArithmeticType::Bf16); present-tense unchecked/unreachable/no-caller wording is historical only.
  - **Correction — 2026-08-10.** after Outcome residual paragraph: same residual-closure + API rename so delivery-time present tense is not re-read as live Fact.
  - Pre-work body: retired stale line citations registry.rs:199 and model.rs:1333 in favor of symbol/path anchors; noted conformance() → conformance_for evolution.

Optional items skipped (with reason):
  - none (optional related edges and line-citation retirement applied as cheap hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - docs/correctness-and-testing.md still spells ReferenceEvaluationRequest::conformance in the paragraph that credits this ticket (doc drift after rename to conformance_for; outside ticket-only wave).
  - Watched-failing 42-disagreement log and dual-base identity sha256 not re-measured (Outcome narrative only; not a ticket-prose repair).
  - make full counts (2,951 / 1,031) not re-verified.

Verification:
  - files read:
    - tickets/wire-the-bf16-reference-to-the-realization-it-is-told.md
    - audit report c81b695e3656_c99ac54950f2.md
    - crates/tiler-reference/src/registry.rs (conformance_for at L223)
    - crates/tiler-ir/src/schedule/model.rs (region_arithmetic_type at L1373)
    - crates/tiler-reference/src/bf16.rs (conformance_for at evaluate; module header cites it)
    - give-the-realization… and route-the-bf16-vertical… frontmatter status: done
  - checks:
    - rg conformance_for / fn conformance in registry.rs
    - rg region_arithmetic_type in model.rs
    - rg status on both residual successor tickets
    - shasum -a 256 on edited ticket

Recommended next ledger state:
  integrated
