Ticket: prove-the-c1-stateful-attention-vertical
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/prove-the-c1-stateful-attention-vertical/73e6470b8801_c99ac54950f2.md
Pre-edit content hash (from ledger): 73e6470b8801b17907263f9ff78f5991b00b350b860ba383add628e1bcf85ae7
Post-edit content hash: 80f1c70a59998d201e88f13f077f8ceea2fd75a97ece59d69470731460618da8

Changes applied:
  - scopes: [implementation/candle, implementation/runtime, research/runtime] → [implementation/candle]; added dated Scope correction — 2026-08-10 citing supersede consumer-fixture disposition (no runtime type, no L5 research-record edit); modeled on execute-the-stateful-prefill-path / execute-the-decode-step-path.
  - Artifact identity: replaced unconditional "single artifact identity across all nine" with conditioned claim — eight decode steps (T=1, S=11…18) share one identity (L5 invariant 2); prefill (T=10) sharing that identity is conditional on D-19 / symbolic broadcast-result extents (2026-08-05 L5 narrowing under decide-whether-one-decoder-layer-graph); record measured count rather than nine-as-one.
  - Normative reference: named as tiler-reference evaluation of the same decode-shaped block program under the same numerical contract on the stated host; qwen model-logit fixture and prefill-only attention-block probe excluded as substitutes.
  - Variant selected: qualified as packaged multi-variant routing selection from bind-repeated / the route; not deferred Metal tiled contraction body (realize-the-tiled-contraction-schedule-and-its-metal-emission still deferred).
  - Added Correction — 2026-08-10 summarizing (1)–(4); Closes when now names the reference and retains conditioned identity/variant/byte record.
  - status left todo (outcome undelivered; hard dep test-the-autoregressive-state-failure-cases still open).

Optional items skipped (with reason):
  - related: add bind-repeated-invocations-over-caller-retained-tensors and/or integrate-the-autoregressive-decode-loop — audit marked optional clarity only; identity/variant bullets already cite bind-repeated inline; not required for readiness.

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation of the nine-execution Metal driver remains out of wave B (crates/prototypes under implementation/candle when work proceeds).
  - No remainder ticket filed: identity conditioned in place; optional future host measurement of attention-block occurrence counts at T=1 vs T=10 with cache inputs is residual research, not a blocker (report residual uncertainty).

Verification:
  - files read:
    - tickets/prove-the-c1-stateful-attention-vertical.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/prove-the-c1-stateful-attention-vertical/73e6470b8801_c99ac54950f2.md
    - tickets/execute-the-stateful-prefill-path.md (Scope correction + 2026-08-10 identity pattern)
    - tickets/execute-the-decode-step-path.md (Scope correction; tiled-variant qualification)
    - docs/research/runtime/autoregressive-state-and-kv-cache.md (L5 invariant 2; stands for C not T)
    - ticketsplease.toml (scope path maps)
    - tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md (status: deferred)
  - checks:
    - scopes now match prefill/decode candle-only
    - L5 anchors: `eight decode steps at C1 must produce exactly one artifact identity`; `stands for \`C\` and not for \`T\``
    - live unconditional "single artifact identity across all nine" removed (only quoted in Correction as retired wording)
    - shasum -a 256 post-edit: 80f1c70a59998d201e88f13f077f8ceea2fd75a97ece59d69470731460618da8

Recommended next ledger state:
  integrated
