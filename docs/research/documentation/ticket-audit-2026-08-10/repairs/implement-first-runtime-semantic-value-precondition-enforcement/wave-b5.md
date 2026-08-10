Ticket: implement-first-runtime-semantic-value-precondition-enforcement
Wave: B5
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/implement-first-runtime-semantic-value-precondition-enforcement/3c3550e80389_c99ac54950f2.md
Pre-edit content hash (from ledger): 3c3550e803895f929be7af4ff33c943c625b12b22059d67d61f0bf4c4f4c1e54
Post-edit content hash: d2066769dab6d922d679e9d0b553482075baad7c0df9d0d9e5c3236d4ee8fe17

Changes applied:
  - Corrected subject Fact 2: qualified "complete contract" as complete for currently admitted per-tensor strict-affine U4/U8; stated derive refuses non-per-tensor via `UnsupportedValueRepresentation::ParameterMap`; named maps ticket for selected per-axis admission.
  - Dated **Correction — 2026-08-10.** recording (a) per-tensor-only admission gap vs selected per-axis subject and (b) all-element scan Fact vs U8 proof-elision Inference, with raw anchors.
  - New **Vocabulary — ADR names vs runtime types**: `RoutingCommit` → `Preflight::commit` → `RoutedDispatch`; `EnforcementCommit` = missing committed-needs-conformance type-state this ticket adds.
  - Implementation keys / Runtime state / Adversarial / Closes when: parenthetical live-type maps on `RoutingCommit`; U8 proof-elision labelled Proposal (this ticket introduces) with current Fact that `check_bound_value` still scans all U8 content; Closes when labels Inference→implemented support contingent on carrier/width/kind honesty.
  - Optional related hygiene: added `implement-workload-selected-quantized-parameter-maps` to `related` plus Graph maintenance note (related only; hard chain stays through carry/fuse).

Optional items skipped (with reason):
  - none (optional related hygiene applied).

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation (Class E code residual — not ticket-record work): post-commit type-state / EnforcementCommit in tiler-runtime (`load/route.rs`, adapter), host scan before allocate_dispatch, proof-elision disposition in shared checker or plan, Metal/host adapter logical views. Exact files from audit remain product debt owned by this ticket after carry delivers static plans.
  - docs/ contracts (artifact-abi, numerical-semantics, runtime-execution-contract) only when implemented support upgrades Inference language — wave B1 forbids docs/crates edits; coordinate with reconcile ownership at implementation time.
  - No new remainder ticket: per-axis maps owned by implement-workload-selected-quantized-parameter-maps; static plans by carry; remainder after carry is this ticket itself.

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/implement-first-runtime-semantic-value-precondition-enforcement/3c3550e80389_c99ac54950f2.md (full)
    - tickets/implement-first-runtime-semantic-value-precondition-enforcement.md (full, pre and post)
    - crates/tiler-ir/src/semantic/conformance.rs (`if map != &ParameterIndexMap::per_tensor`, ParameterMap refusal, scan_logical_elements)
    - crates/tiler-ir/src/semantic/types.rs (`ParameterIndexMapKind` sole `PerTensor`)
    - crates/tiler-runtime/src/load/route.rs (`pub fn commit(self) -> RoutedDispatch`)
    - docs/numerical-semantics.md (all-element scan Fact / proof-elision Inference)
  - checks:
    - rg anchors: per-tensor qualification, Correction 2026-08-10, Vocabulary section, proof-elision Proposal/Fact split, related maps ticket, Closes when Inference→implemented
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
