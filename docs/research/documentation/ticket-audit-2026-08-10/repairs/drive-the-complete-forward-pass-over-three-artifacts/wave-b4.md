Ticket: drive-the-complete-forward-pass-over-three-artifacts
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/drive-the-complete-forward-pass-over-three-artifacts/cf80416bb05b_c99ac54950f2.md
Pre-edit content hash (from ledger): cf80416bb05bf8c0869f77b7d103b002ce658dadbd1a97ea16ecd66533e77491
Post-edit content hash: 13bd3f5e2edb193c6849f6f47c80dbc9710c662e32b72c56876e86c8aa9f1a22

Changes applied:
  - scopes: `[implementation/runtime, implementation/frontend]` → `[implementation/candle]` (consumer driver beside candle-metal-adapter; matches peer ordinal ticket 2026-08-09 correction and ticketsplease.toml candle glob).
  - related: added `define-the-widening-relation-over-a-symbolic-broadcast-extent` and `decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode` so the identity pin is graph-visible (kept as related, not hard deps, after conditionalizing the close pin).
  - Required behaviour bullet 1: 30 executions unconditional; exactly-3 identities across prefill+decode conditional on L6 D-19 and properties (a)(b)(c); until then report observed count with attribution (T=1 divergence vs S specialization), per L8 conditional pin.
  - Required behaviour bullet 6: ordinal obligation is call-site pairing on this ticket; five-class diagnostic surface remains `name-the-execution-ordinal-in-model-level-failures`.
  - Closes when: same conditional identity pin; post-commit clause is call-site ordinal pairing; points ordinal composition to ticket 10.
  - Dated correction 2026-08-10 under Required behaviour / Closes when: L6 2026-08-05 Whole-model composition correction, L8 conditional pin, and scope move.

Optional items skipped (with reason):
  - none (related D-19 / decide-prefill-decode edges applied as cheap graph hygiene with the conditionalized pin).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required for this wave; parent design-ticket historical dep-7 sentence (Fact 18) is out of ticket scope and not listed as required repair on this id.
  - product work remains: no consumer driver, open deps (ingest, deliver-symbolic, integrate-decode), D-19 still open — not wave B prose.

Verification:
  - files read:
    - full audit report cf80416bb05b_c99ac54950f2.md
    - full ticket (pre- and post-edit)
    - ticketsplease.toml implementation/candle glob
    - tickets/name-the-execution-ordinal-in-model-level-failures.md (scope correction + ordinal split)
    - docs/research/program-planning/model-level-qualification.md (L8 conditional pin)
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (L6 thirteen-not-three / D-19 correction anchors via grep)
    - ticket id existence for define-the-widening-relation and decide-whether-one-decoder-layer-graph
  - checks:
    - sha256 post-edit: 13bd3f5e2edb193c6849f6f47c80dbc9710c662e32b72c56876e86c8aa9f1a22
    - scopes now only implementation/candle
    - related includes both D-19 graph tickets
    - no crates/ or other tickets edited

Recommended next ledger state:
  integrated
