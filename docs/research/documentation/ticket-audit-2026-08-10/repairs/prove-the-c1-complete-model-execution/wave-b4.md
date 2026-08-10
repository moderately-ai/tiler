Ticket: prove-the-c1-complete-model-execution
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/prove-the-c1-complete-model-execution/319fd52ef889_c99ac54950f2.md
Pre-edit content hash (from ledger): 319fd52ef88986dfb0a3485907fba65293bf08d36cc0c006b443dee01424ba36
Post-edit content hash: 7eb5151f4d608910ad6baf94a753c6ca7eddae3d2a2bbfdf70df946608f84566

Changes applied:
  - scopes: `[implementation/runtime, research/program-planning]` → `[implementation/candle, research/program-planning]` (L6 places the complete-model driver beside `prototypes/candle-metal-adapter`; candle scope maps to `prototypes/candle-*/**`; no named tiler-runtime change for this proof).
  - related: added `define-the-widening-relation-over-a-symbolic-broadcast-extent` (D-19, awaiting-decision) and `decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode` (done; L6 correction carrier) so the identity pin's condition is graph-visible; not hard deps because Closes when is now conditional.
  - What must hold "The counts": 270 unconditional; exactly-3 identities conditional on L6 D-19 / properties (a)(b)(c); until D-19, report observed count with attribution; fourth identity within one forward pass still fails unconditionally.
  - State-closes bullet: "selected runtime pool" → "consumer's retained pool under the selected physical layout"; kept 147,456 / 112 arithmetic.
  - Closes when: execution count 270 is the fail-able gate; identity count uses L8 conditional discipline (not unconditional ≠ 3).
  - Dated correction 2026-08-10 under What must hold pointing at L6 2026-08-05 Whole-model composition correction and L8 conditional pin; notes consumer pool ownership.

Optional items skipped (with reason):
  - Optional related for `define-the-model-level-conformance-corpus` / `build-the-model-level-measurement-harness` (report: harness integration, not required for ticket-body truth).

Residuals not applied (docs/crates/new tickets/authority):
  - none; report Exact files listed only this ticket; no new remainder ticket; D-19 already open.

Verification:
  - files read:
    - tickets/prove-the-c1-complete-model-execution.md (full, pre/post)
    - audit report 319fd52ef889_c99ac54950f2.md (full)
    - ticketsplease.toml (implementation/runtime vs implementation/candle globs)
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (anchors: prototypes/candle-metal-adapter; thirteen, not three; Correction 2026-08-05)
    - docs/research/program-planning/model-level-qualification.md (pinned conditionally on L6's D-19)
    - docs/research/runtime/dynamic-kv-physical-layout.md (The **consumer** owns them)
    - tickets/define-the-widening-relation-over-a-symbolic-broadcast-extent.md (status awaiting-decision)
    - tickets/decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md (status done)
    - tickets/prove-the-c1-stateful-attention-vertical.md (scopes include implementation/candle)
  - checks:
    - scopes no longer authorize only tiler-runtime for the model driver
    - three-identity close is conditional; 270 remains unconditional
    - status remains todo (not delivered; open hard deps unchanged)

Recommended next ledger state:
  integrated
