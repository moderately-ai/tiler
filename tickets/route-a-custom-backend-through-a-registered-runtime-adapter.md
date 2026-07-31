---
id: route-a-custom-backend-through-a-registered-runtime-adapter
title: Route a custom backend through a registered runtime adapter
status: todo
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary, declare-a-required-gpu-family-in-the-artifact]
related: [runtime-execution-contract, prototype-metal-runtime-execution, make-runtime-routing-commit-authority-one-shot]
scopes: [implementation/runtime, implementation/artifact, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, runtime]
---
## User-visible outcome

A consumer can register a statically linked runtime adapter for one backend/representation family, bind a validated artifact to a live execution context, prepare it before routing commit, and dispatch it with correct resource lifetimes.

## Implementation keys

- Define `LiveExecutionContext` separately from the existing device-free `ExecutionEnvironment`; do not make a caller-stated tuple masquerade as discovered device truth.
- Register adapters explicitly and freeze the registry before loading; reject duplicate ownership rather than selecting by insertion order.
- Match backend family, representation, target profile, payload identity, and live applicability before adapter preparation.
- Reuse the route-requirement family and exact-entry `PreparedEntryTargetRequirement` authority established by `declare-a-required-gpu-family-in-the-artifact`; do not invent an adapter-specific capability, query, or applicability vocabulary.
- Keep device discovery, library/pipeline preparation, binding, encoding, submission, terminal-success observation, and asynchronous retention in the adapter half, downstream of device-free decoding and validation.
- Preserve preflight before routing commit and forbid fallback after allocation, partial encoding, submission, or semantic validation failure.
- Add one non-Metal adapter fixture and retain the existing Metal proof as an independent consumer.
- Perturb every identity/compatibility field, preparation outcome, binding, and post-commit failure path.
- Present the exact public runtime trait, context, registry, and call-site boundary to Tom.

## Closes when

One external adapter executes a carried payload through the ordinary loader/route path, missing, duplicate, incompatible, and post-commit failures are typed and explainable, asynchronous resources survive final device use, targeted checks and final gate pass, and `tiler-runtime` remains device-free.

## Graph maintenance

- Release backend-aware variant selection only after a registered adapter can establish eligibility.
- Keep Candle and Metal runtime objects out of the consumer-neutral trait.
- Split any unsafe FFI site into its own ADR-0079-conforming review.
