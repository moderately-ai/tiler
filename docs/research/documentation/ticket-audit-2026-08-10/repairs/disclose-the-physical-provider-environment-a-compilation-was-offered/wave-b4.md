Ticket: disclose-the-physical-provider-environment-a-compilation-was-offered
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/disclose-the-physical-provider-environment-a-compilation-was-offered/c8cde8ac6c8e_c99ac54950f2.md
Pre-edit content hash (from ledger): c8cde8ac6c8e38c1aa4aa4368f3335bd414b523dd3d0ddfa8c4107784ba52d21
Post-edit content hash: 0ea1aa25bd55c75383d8db4c61f0c22ae7a301e7439ca2a5cbcd339a701b8cff

Changes applied:
  - related: added `disclose-offered-and-selected-physical-provider-sets-separately` and `accept-the-installed-physical-provider-public-surface`; kept existing three; status left `awaiting-decision`; scopes left unchanged (already artifact/build/contracts).
  - User-visible outcome rewritten to residual CompilationEnvironment subject (Option A whole tagged environment vs Option B lowering-only docs); compiler disclosure marked complete/not reopened.
  - Why section: struck/retired key-1 doc-defect Fact; replaced "exactly one physical provider / nothing can vary it / not observably wrong today" with multi-provider install live + assemble still lowering-only; restated ADR 0072 without claiming unused environment rows are packaged identity or that SemanticRegistrySnapshotIdentity is false of artifact bytes; marked ADR 0090 name attribution and accessor ownership claims false/imprecise with pointers to composition record and disclose-offered ticket.
  - Corrected 2026-08-08 block: dropped outstanding "coordinator should re-scope" language; noted frontmatter scopes already match residual crates.
  - Added ## Fact audit — 2026-08-10 covering key 1 still discharged, multi-provider falsifying premise, selected physical never reaching select_provider (Option A may need both answers), scopes already correct, related graph update, status stays awaiting-decision.
  - Implementation keys annotated discharged / still load-bearing / open; Decision packet reworded off implementation/compiler and notes Option A selected-physical packaging residual + post-decision implementation ticket constraint; Closes when marks first clause already true.

Optional items skipped (with reason):
  - none required as optional in Repair required; residual product encoding (Option A/B implementation) is Tom-gated and out of wave B scope.

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-artifact CompilationEnvironment type docs / encoding (post Option A or B).
  - crates/tiler-build/src/plan_artifact.rs assemble path (post Option A).
  - docs/artifact-abi.md / identity pins if Option A encoding moves.
  - no new remainder ticket filed (report: none new if Tom answers here; Option A needs one identity-step implementation ticket after decision — not wave B).

Verification:
  - files read: audit report; ticket pre-edit; greps on plan_artifact.rs (CompilationEnvironment::new from offered_providers; select_provider only on selected_capabilities); session.rs / external_physical_provider.rs (with_physical_providers, offered_physical_providers); tiler-artifact model/codec tests (unused environment not in bytes; SelectedProvider compilation-request environment wording).
  - checks: shasum -a 256 post-edit ticket = 0ea1aa25bd55c75383d8db4c61f0c22ae7a301e7439ca2a5cbcd339a701b8cff; status still awaiting-decision; scopes unchanged.

Recommended next ledger state:
  integrated
