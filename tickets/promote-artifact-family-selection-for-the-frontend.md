---
id: promote-artifact-family-selection-for-the-frontend
title: Promote artifact-family selection for the frontend
status: todo
priority: p1
dependencies: [prototype-artifact-family-delivery, admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [prototype-inline-proc-macro-frontend, generate-cfg-gated-artifact-family-delivery]
scopes: [implementation/frontend, implementation/metal-aot, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The frontend can state the accepted artifact-family delivery policy through one reviewed typed request without duplicating the crate-private `ArtifactFamilySelection` or teaching the proc macro Apple tool-discovery logic.

## Implementation keys

Review the construction sites and promote the smallest existing `ArtifactFamilySelection`, `ArtifactDeliveryPolicy`, `SelectedFamily`, `FamilyRequirement`, and validation/error surface needed by the frontend. Preserve canonical ordering, explicit `FallbackOnly`, duplicate/empty refusal, per-family deployment minimum and MSL standard, and the accepted one-envelope/N-payload identity. Do not move generated consumer-`cfg` mapping into `tiler-metal-aot`; `generate-cfg-gated-artifact-family-delivery` remains frontend-owned.

If exposing the type from `tiler-metal-aot` would make the facade depend on driver/tool discovery or leak backend-only policy into a consumer-neutral frontend boundary, move the dependency-neutral request vocabulary to the lowest existing owner that both sides may depend on rather than copying it. Preserve `tiler-metal-aot`'s empty dependency closure and reject any second canonical encoder.

## Public boundary for Tom

Present the exact owner/module/type/constructor/reader/error path and frontend call site before acceptance. This review does not reopen the already accepted family selection semantics or one-envelope/N-payload decision.

## Closes when

The frontend can construct and inspect one canonical selection, the AOT driver consumes the same value, no duplicate encoding or Apple host inference exists, public dependency direction is reviewed, mutation tests prove empty/duplicate/order/version checks can fail, and targeted tests/Clippy plus `make full` pass.

## Graph maintenance

- Follow facade admission explicitly because the reviewed packet includes the frontend call site; do not rely on frontend-scope collision for prerequisite order.
- Keep generated consumer-`cfg` mapping in `generate-cfg-gated-artifact-family-delivery` and Apple tool discovery in the AOT owner.
- Release the proc-macro proof only after one dependency-neutral canonical request is available without duplicating its encoder.
