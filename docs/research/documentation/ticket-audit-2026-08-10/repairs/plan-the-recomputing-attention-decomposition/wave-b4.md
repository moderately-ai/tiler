Ticket: plan-the-recomputing-attention-decomposition
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/plan-the-recomputing-attention-decomposition/a231ae5881a7_c99ac54950f2.md
Pre-edit content hash (from ledger): a231ae5881a78634f2f0c67144e70826eef0d63915105ee1e2a7dba9fa5ecd50
Post-edit content hash: 85eaf2860cf19564ff9a6368fd9cbbcfbd8743f2e11a49dd8b9525d354fe2c04

Changes applied:
  - User-visible outcome: replaced "long benchmark rows become feasible" with D-11 predicate language (lower transient requirement that a target profile can accept where materialized residency rejects or stays `Unknown`).
  - "What it is, precisely" online-form Inference: multi-dimension refusal now names distributivity (ADR 0080 / ADR 0095 / numerical-semantics) and elementary-function identity (ADR 0101 online-softmax rescaling fold); dropped reassociation as the sole second ground; multi-dimension naming rule cited (ADR 0080 item 5; ADR 0101 decision 6).
  - Dated correction under that Inference: L4 proposal-era two-permission account (distributivity + reassociation) superseded for refusal vocabulary by ADR 0101 (2026-08-06).
  - Evidence prerequisite stage-handoff Fact: scoped to program-stage ordering / pass-boundary-is-dispatch-boundary; clause that it does not decide kernel-internal synchronization; CooperativeWorkgroup / SynchronizationPoint admitted; D-B application sentence labeled Inference.
  - Non-goals: online-form rejection aligned to the same multi-dimension wording (distributivity + elementary-function identity; ADRs 0080, 0095, 0101); distributivity-permission owner noted as closed.
  - Optional related hygiene: added `scope-causal-structure-aware-attention-schedules` and `reconcile-the-first-attention-planning-record-with-landed-fusion-roles-and-budgets`.
  - Status left `todo` (close condition unmet; no metadata change required for graph readiness).

Optional items skipped (with reason):
  - none (both optional related edges applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required. Product implementation of D-B remains open work on this ticket when dispatched; wave B is ticket prose only.

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/plan-the-recomputing-attention-decomposition/a231ae5881a7_c99ac54950f2.md
    - tickets/plan-the-recomputing-attention-decomposition.md (pre- and post-edit)
    - tickets/plan-the-materialized-attention-decomposition.md (program-stage vs kernel-sync Fact correction)
    - docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md (decision 2 dual consumption; decision 6 multi-dimension refusal; Consequences flash-class two named reasons)
    - docs/research/program-planning/first-attention-program-vertical.md (D-11 transient budget gap; feasibility predicate 1)
    - crates/tiler-ir/src/program/model.rs (anchor: pass boundary *is* the dispatch boundary)
  - checks:
    - ADR 0101: online-softmax rescaling fold consumes both distributivity and elementary-function identity; multi-dimension refusal must name all missing dimensions.
    - model.rs still carries PartialReduction Data-edge / pass-boundary quote.
    - plan-the-materialized already separates program-stage Data from CooperativeWorkgroup kernel sync.
    - L4 D-11: no target profile declares a transient memory limit → D-A at B1-d is Unknown not proved infeasible.
    - shasum -a 256 post-edit ticket → 85eaf2860cf19564ff9a6368fd9cbbcfbd8743f2e11a49dd8b9525d354fe2c04

Recommended next ledger state:
  integrated
