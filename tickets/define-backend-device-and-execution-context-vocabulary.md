---
id: define-backend-device-and-execution-context-vocabulary
title: Define backend, device, and execution-context vocabulary
status: todo
priority: p1
dependencies: [correct-stale-public-compiler-boundary-authorities]
related: [draft-public-extension-seam-ownership-adr, multi-device-and-sharding-scope-gate]
scopes: [contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, documentation, architecture]
---
## User-visible outcome

A reader and future API author can distinguish a backend, backend family, provider, target profile, artifact family, representation, live device, execution context, device-free execution environment, and runtime adapter without inferring meaning from the current Metal crate split.

## Why this slice exists

Only `Target profile` is defined as shared glossary vocabulary. Backend and device responsibilities are coherent but scattered, `BackendKey` is represented without a conceptual ownership contract, and the public device-free `ExecutionEnvironment` can be mistaken for the live device/context it deliberately does not contain. The word `family` currently names backend, target-profile, artifact, and GPU/device subjects.

## Implementation keys

- Derive definitions from accepted ADRs 0043, 0047, 0072, 0078, 0081, and 0085 and from the exact construction sites of `BackendKey`, `RepresentationKey`, `TargetProfileRef`, and `ExecutionEnvironment`.
- State that a target profile is typed compile-time data, a device is a live execution resource, and a runtime execution context scopes device objects, queues, caches, and asynchronous lifetimes.
- Define a backend by responsibility rather than by copying `tiler-metal` packaging: it consumes verified physical work and produces a target representation, while AOT invocation, artifact assembly, loading, and live execution may have different owners.
- Distinguish provider identity from backend-family and representation identity.
- Add crate-role naming guidance such as `tiler-<backend>` only where the role is actually backend-owned; do not prescribe one package topology for every target.
- Preserve the accepted one-device initial profile and explicitly defer multi-device semantics to its existing scope gate.
- Prove every absence/conflict search used by the update can fail, and read every edited contract in full.

## Closes when

The glossary and governed contracts use one subject for each term, every ambiguous use of `family` or `environment` in the affected passages is qualified, no proposed provider-bundle API is presented as accepted, local links and hand-maintained catalogs agree, and `tkt lint` plus `git diff --check` pass.

## Graph maintenance

- Follow the stale compiler-boundary correction rather than duplicating its installation-status edits.
- Feed these terms into the provider-composition research and every later public-boundary ticket.
- Keep `multi-device-and-sharding-scope-gate` deferred; this ticket defines vocabulary and does not activate that product decision.
