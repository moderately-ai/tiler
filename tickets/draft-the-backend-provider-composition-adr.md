---
id: draft-the-backend-provider-composition-adr
title: Draft the backend-provider composition ADR
status: todo
priority: p1
dependencies: [specify-the-consumer-neutral-backend-provider-composition-contract]
related: [draft-public-extension-seam-ownership-adr, multi-device-and-sharding-scope-gate]
scopes: [contracts/decisions, contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, decision, adr]
---
## User-visible outcome

One proposed ADR presents the concrete backend-provider participation model, including whether target-specific scheduling knowledge is typed profile data, provider code, or a checked combination, without treating the proposal as accepted.

## Implementation keys

- Derive the decision from the completed custom-Metal and CPU evidence rather than from interface aesthetics.
- Decide the intended participation model of `PhysicalImplementationProvider` and explicitly refine ADR 0078's deferred item.
- State the composition of target-profile authorities, physical providers, emitters/artifact producers, runtime adapters, and execution contexts.
- State whether partial providers may contribute specialized candidates while reusing another backend's emitter/runtime components, and how identity proves the selected composition.
- Preserve static linking, explicit immutable registries, re-verification, deterministic ambiguity, no silent override, no provider-authored proof, and no runtime source compilation.
- Separate build-time and runtime installation and define the artifact-carried join.
- State the initial single-device limit and leave multi-device/sharding to its existing activation gate.
- Label every fact, inference, proposal, and measurement, update the hand-maintained proposed-decision catalog, and create no implementation visibility changes.

## Closes when

A coherent proposed ADR exists with alternatives eliminated against correctness, performance, and maintainability; the affected contracts disclose its proposed status without asserting it; and one acceptance node structurally blocks every implementation ticket conditional on the decision.

## Graph maintenance

- Make `accept-the-public-backend-provider-composition-boundary` the sole acceptance node and keep it parked for Tom.
- Move the acceptance node from `todo` to `awaiting-decision` only after the complete decision packet exists.
- Point every conditional implementation ticket at the acceptance node, never directly at this drafting ticket.
- If more than one genuine product choice survives, split it into atomic decision records before asking Tom.
