Ticket: measure-b1-d-peak-residency-on-a-named-host
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/measure-b1-d-peak-residency-on-a-named-host/77fede757ac7_c99ac54950f2.md
Pre-edit content hash (from ledger): 77fede757ac78ce2da1252562e08db432c9fe8e2a62701c79750985b09f41232
Post-edit content hash: cb70abbdaa14661792e075f899ce19ddf5dc3ef1336a8a52aeff7adb886f8dc1

Changes applied:
  - First Fact: B1-section quote now matches L1 literal wording including "the selected layout carrier to land and its residency formula to be measured" (exclusion-table half unchanged).
  - Second Fact: L6 peak figures named as **historical rejected-candidate** Inference, not current physical-residency authority; "nothing has measured" scoped to resident-process peaks.
  - Inference: bare "rejected compact-allocation candidate" → **rejected singular dense-allocation candidate**; Why "rejected dense candidate" left as already-aligned family name.
  - Added **Correction — 2026-08-10.** under Why summarizing the three terminology/authority fixes.
  - Metadata left unchanged (status todo, deps/related/scopes/tags/paths graph-true per audit).

Optional items skipped (with reason):
  - none (Repair required listed no optional bullets).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket's prose. Eventual named-host measurement, harness delivery, L1 exclusion citation, and survivor full-row recomputation table remain product work owned by this ticket / harness dependency, out of wave B ticket-only repair.
  - Residual uncertainty from audit (which named host; KV-only vs weights+pool+transient+logits stated sum) not resolved here — product delivery choices, not false Facts.

Verification:
  - files read:
      - docs/research/documentation/ticket-audit-2026-08-10/reports/measure-b1-d-peak-residency-on-a-named-host/77fede757ac7_c99ac54950f2.md (full)
      - tickets/measure-b1-d-peak-residency-on-a-named-host.md (full, pre/post)
      - docs/research/program-planning/first-metal-lm-workload.md (B1 Inference + exclusion table anchors via search)
      - docs/research/program-planning/complete-model-ingestion-and-execution.md (Historical inference for rejected singular dense-allocation candidate)
      - docs/research/runtime/dynamic-kv-physical-layout.md (compact-allocation exact-live policy control name)
      - tickets/design-model-ingestion-and-complete-execution.md (Historical rejected-candidate arithmetic)
  - checks:
      - L1 B1: `needs the selected layout carrier to land and its residency formula to be measured on a named host first, and it belongs to L8` present
      - L1 exclusion: `Contexts beyond 8,320 tokens` / `A residency measurement on a named host, under L8` present
      - Corpus name `rejected singular dense-allocation candidate` / L6 `Historical rejected-candidate arithmetic` confirmed
      - Bare `compact-allocation exact-live policy` is dynamic-kv measurement control, distinct candidate
      - shasum -a 256 post-edit: cb70abbdaa14661792e075f899ce19ddf5dc3ef1336a8a52aeff7adb886f8dc1

Recommended next ledger state:
  integrated
