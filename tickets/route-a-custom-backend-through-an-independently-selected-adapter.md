---
id: route-a-custom-backend-through-an-independently-selected-adapter
title: Route a custom backend through an independently selected runtime adapter
status: in-progress
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary, declare-a-required-gpu-family-in-the-artifact]
related: [runtime-execution-contract, prototype-metal-runtime-execution, make-runtime-routing-commit-authority-one-shot]
scopes: [implementation/runtime, implementation/artifact, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, runtime]
claimed_from: todo
assignee: worker-route-a-cust
lease_expires_at: 1785553789
---
## User-visible outcome

A consumer's statically linked runtime adapter for one backend/representation family binds a validated artifact to a live execution context, prepares it before routing commit, and dispatches it with correct resource lifetimes — selected by the consumer, never resolved from a registry.

## Implementation keys

- Define `LiveExecutionContext` separately from the existing device-free `ExecutionEnvironment`; do not make a caller-stated tuple masquerade as discovered device truth.
- **Corrected 2026-07-31 by accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), which this key predated:** there is no adapter registry — independent selection is the runtime adapter's mechanism, and giving it one is a named eliminated alternative. The consumer selects its adapter the way `prototypes/serial-sum-run` and the CPU vertical already do; what joins producer to adapter is the artifact's governed backend/representation/profile identities, compared by the loader, with producer provenance never matched against adapter identity.
- Match backend family and representation as a pair, classify the target profile, and check payload identity and live applicability before adapter preparation — the loader owns every comparison; the adapter reports facts and never adjudicates them (ADR 0090 item 4).
- Reuse the route-requirement family and exact-entry `PreparedEntryTargetRequirement` authority established by `declare-a-required-gpu-family-in-the-artifact`; do not invent an adapter-specific capability, query, or applicability vocabulary.
- Keep device discovery, library/pipeline preparation, binding, encoding, submission, terminal-success observation, and asynchronous retention in the adapter half, downstream of device-free decoding and validation.
- Preserve preflight before routing commit and forbid fallback after allocation, partial encoding, submission, or semantic validation failure.
- Add one non-Metal adapter fixture and retain the existing Metal proof as an independent consumer.
- Perturb every identity/compatibility field, preparation outcome, binding, and post-commit failure path.
- Present the exact public runtime trait, context, and call-site boundary to Tom (no registry exists to present).

## Closes when

One external adapter executes a carried payload through the ordinary loader/route path; incompatible and post-commit failures are typed and explainable (missing/duplicate registration failures do not exist under the accepted no-registry model); asynchronous resources survive final device use, targeted checks and final gate pass, and `tiler-runtime` remains device-free.

## Graph maintenance

- Release backend-aware variant selection only after a registered adapter can establish eligibility.
- Keep Candle and Metal runtime objects out of the consumer-neutral trait.
- Split any unsafe FFI site into its own ADR-0079-conforming review.
