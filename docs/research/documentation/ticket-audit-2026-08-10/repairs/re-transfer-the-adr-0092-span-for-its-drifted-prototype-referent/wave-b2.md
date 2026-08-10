Ticket: re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent/f3696aa09e63_c99ac54950f2.md
Pre-edit content hash (from ledger): f3696aa09e63bced918a04e6776e86f4acbe4a0191075ab5cd14a89fc83e93c2
Post-edit content hash: c5464f764276051d235dc4553726752be80ec6c33fa71a4b16fb40d3f95badc2

Changes applied:
  - What closes this: **Correction — 2026-08-10.** withdrawing "cited by AGENTS.md as the standing convention" — AGENTS.md has only the verbatim-landable transfer sentence; the link-condition convention is the research record's own (record already said so on 2026-08-08).
  - Outcome prototype Fact: **Correction — 2026-08-10.** re-pin by symbols `binding_apple_enumerator` / `probe_apple_families` and `observed_apple_family` (supportsFamily closure); mark `:1161-1319` and `:658-661` as non-live base-bound windows.
  - Outcome cmp block: **Correction — 2026-08-10.** mark ADR:46/span:378 and ADR:64/span:394 as delivery-era pins that have rotted; subject anchors (`The pattern is available to every backend`, `Publish the family vocabulary and let each consumer observe the device itself`); audit-base pins ADR:48↔REC:389 and ADR:72↔REC:405; whole-span 28/0 still holds.
  - Outcome fold paragraph: **Correction — 2026-08-10.** same AGENTS.md standing-convention withdrawal as under What closes this.
  - Metadata: none (status stays done; graph edges fine per report).

Optional items skipped (with reason):
  - none (report listed no optional bullets).

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required ticket-only dated corrections; no docs/crates edits, no new remainder tickets (item-6 re-transfer already has its own closed ticket).

Verification:
  - files read:
    - tickets/re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent.md (full, pre- and post-edit)
    - audit report f3696aa09e63_c99ac54950f2.md (full)
    - docs/decisions/0092-…md (item 8 / alternatives entry anchors at current lines 48 and 72)
    - docs/research/runtime/backend-scoped-route-requirement-answers.md (span item 8 / alternatives at current lines 389 and 405)
    - prototypes/serial-sum-run/src/proof.rs (binding_apple_enumerator, probe_apple_families)
    - prototypes/candle-metal-adapter/src/adapter.rs (observed_apple_family supportsFamily closure)
    - AGENTS.md (verbatim-landable only; no span/repoint/standing convention)
  - checks:
    - subject anchors present and matching between ADR and research span
    - prototype symbols present; no pair-table claim re-opened
    - four dated corrections present beside the false live pins/attributions

Recommended next ledger state:
  integrated
