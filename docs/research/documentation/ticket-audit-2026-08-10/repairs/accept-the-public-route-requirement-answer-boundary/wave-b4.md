Ticket: accept-the-public-route-requirement-answer-boundary
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-public-route-requirement-answer-boundary/fe121ddf15ac_c99ac54950f2.md
Pre-edit content hash (from ledger): fe121ddf15ac867b259e071a7c323bd0bfc47c6002fae798a71ced14f2dca4f4
Post-edit content hash: 5dd0256901ba82996f664f1bd804148c350c0b4162d6720fe43d590efe9a4f25

Changes applied:
  - Required: withdrew the 2026-08-08 claim that research record sites `:257` and `:317` still carry the flat quotation and `:389` as live evidence; **Correction — 2026-08-10** states those live sites use the frontend-qualified anchor and keep the flat form only in dated retired notes.
  - Required: replaced measurement-boundary present-tense `none compiles` paraphrase with ADR 0092 live derived-at wording; named observation half landed (`observe_highest_gpu_family`, `AppleGpuFamilyConstant`) and decision half still uncompiled.
  - Required: "released three implementation tickets" → two (with accept-adr-0093 execution correction); clarified "none of its seven items has a ticket" as no per-item implementation ticket / this node is the packet.
  - Required: Ripens when and 2026-08-07 / 2026-08-09 trigger-log phrasing now prefer "no dispatching consumer that needs the decoder" over bare "no consumer dispatches" / "no dispatching consumer has arrived".
  - Optional hygiene: added 2026-08-10 **not fired** trigger-log line with recheck commands.

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none from Repair required (report listed ticket-only prose; status/deps/related/scopes unchanged and correct as `deferred`)
  - seven public-boundary items remain Tom's until trigger fires; no product/docs/crates work in this wave
  - `f7e57bd` acceptance hash provenance left as ticket prose (report residual uncertainty; acceptance corroborated by ADR 0092 + accept-adr-0093)

Verification:
  - files read: full audit report; full ticket pre-edit; ADR 0092 measurement/status prose; research record architecture-sentence sites; accept-adr-0093 "two, not three"; applicability.rs observation half presence; crates absence of decide_metal_route_requirement / MetalRouteRequirementAnswer; tiler-build absence of RouteRequirement
  - checks: shasum -a 256 post-edit; rg RouteRequirement in tiler-build empty; rg decision-half names in crates empty

Recommended next ledger state:
  integrated
