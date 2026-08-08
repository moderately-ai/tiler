---
id: expose-explicit-backend-provider-and-selection-policy-composition
title: Expose explicit backend-provider and selection-policy composition
status: in-progress
priority: p1
dependencies: [drive-an-external-physical-implementation-provider-through-compilation, produce-a-custom-backend-payload-through-the-build-orchestrator, select-executable-variants-across-registered-backend-families, route-a-custom-backend-through-an-independently-selected-adapter]
related: [prototype-public-compiler-api, admit-the-tiler-facade-and-proc-macro-crate-boundary]
scopes: [implementation/compiler, implementation/build, implementation/runtime, implementation/artifact, contracts/foundation, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, api]
claimed_from: todo
assignee: coord
lease_expires_at: 1786180895
---
## User-visible outcome

An ordinary compiler/runtime consumer can explicitly compose accepted statically linked backend components and state allowed or required backend policy without global discovery, forks, or registration-order behavior.

## Implementation keys

- Build the facade from the already exercised component seams rather than introducing a second set of traits.
- Let consumers register complete backends or partial provider components only in combinations the accepted composition contract can validate.
- Freeze compiler/build/runtime registries independently and make their canonical identities available for explain and cache/request provenance.
- Express allowed, required, or fallback-only backend-family policy as typed input outside semantic graph meaning.
- Do not add an unrestricted arbitrary scoring callback; use governed policy and cost identities.
- Reject missing required pieces, duplicate authority, incompatible component identity, and policy that permits no executable route before work.
- Keep inline proc-macro provider visibility bounded by host dependencies and explicit invocation declarations under ADR 0045.
- Provide compile-pass examples for standard Metal, custom Metal specialization, CPU only, and an allowed Metal-or-CPU set.
- Present every public crate/module/trait/type/call-site boundary to Tom.

## Closes when

The consumer-facing API composes the proven seams without duplicating ownership, all example policies produce deterministic request/artifact/routing identities and typed failures, no global registry or backend-specific semantic type appears, and targeted plus full gates pass.

## Graph maintenance

- Make the cross-process identity join and final multi-provider portfolio depend on this exact facade.
- If the facade requires the not-yet-admitted `tiler` crate, sequence behind its admission and review rather than bypassing it.
- Keep shared-library loading and stable plugin ABI deferred.
