Ticket: derive-the-multi-round-two-level-reduction-composition
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-multi-round-two-level-reduction-composition/a8cec39338ef_c99ac54950f2.md
Pre-edit content hash (from ledger): a8cec39338eff0e83bba285ac1207d6d40be17351d94c665ba782eddb6a77680
Post-edit content hash: df155341c827cd9b9596914543fc9e4703ae290895aabc8d93d67edcb84a530c

Changes applied:
  - Kept `status: done`; expanded `related` with `accept-adr-0100-multi-round-reduction-composition`, `catalogue-adr-0100-and-the-multi-round-composition-record`, and `correct-the-extrema-familys-identity-ground-and-name-its-padding-identity` (optional graph hygiene from Repair required).
  - Retitled `## Why this is live` to `## Historical premise (carrier-time)` and past-tensed the filing-time Facts/Inference so a done ticket does not read as open work.
  - Added `## Outcome` covering: research path + `research_status: complete`; drafted then-accepted ADR 0100 and acceptance ticket at `e10adb74`; ADR 0096 open-question update / decision-8 supersession; five public-boundary items left for Tom; deferrals left on research/ADR (including unfired round-dependent span); derivation bases `1d918b67` / `d9bd49ef`; composition leaf-order and identity summary.
  - Added dated `**Fact audit — 2026-08-10.**` noting the missing-Outcome close at `7d49e639` and naming residual docs debt (ADR 0096 proposed-tense durable result; research disposition/pending catalog).

Optional items skipped (with reason):
  - none; optional related expansion applied as cheap graph hygiene on this ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - docs/decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md — durable-result paragraph still asserts ADR 0100 is `proposed` / "nothing here is decided" (Class C: ticket-only; out-of-ticket load-bearing for close-condition prose honesty).
  - docs/research/scheduling/multi-round-two-level-reduction-composition.md — status prose + `disposition: "pending"` lag acceptance.
  - docs/research/README.md — catalog row still "pending"; should move to adopted pattern used by one-round two-level sibling.
  - No new remainder ticket filed (report: none required for this mandate; deferrals already owned).

Verification:
  - files read:
    - tickets/derive-the-multi-round-two-level-reduction-composition.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-multi-round-two-level-reduction-composition/a8cec39338ef_c99ac54950f2.md (full)
    - tickets/accept-adr-0100-multi-round-reduction-composition.md (Outcome / acceptance provenance)
    - tickets/catalogue-adr-0100-and-the-multi-round-composition-record.md (Outcome)
    - docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md (accepted frontmatter, decisions, deferrals, 2026-08-09 corrections)
    - docs/decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md (open-questions durable-result "proposed" residual)
    - docs/research/scheduling/multi-round-two-level-reduction-composition.md (research_status complete, disposition pending, Public-boundary items 1–5, Deferrals)
    - git log for d9bd49ef, e10adb74, 7d49e639, 1d918b67
  - checks:
    - `rg -n '## Outcome|Historical premise'` on ticket after edit
    - ADR 0100 `decision_status: "accepted"`; research `research_status: "complete"`; disposition still `"pending"` (residual)
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
