---
id: specify-the-consumer-neutral-backend-provider-composition-contract
title: Specify the consumer-neutral backend-provider composition contract
status: todo
priority: p1
dependencies: [define-backend-device-and-execution-context-vocabulary, prototype-a-forkless-custom-metal-physical-provider, prototype-a-bounded-scalar-cpu-backend-vertical]
related: [draft-public-extension-seam-ownership-adr, runtime-execution-contract, target-profile-feasibility-model]
scopes: [research/extensions, research/program-planning, research/artifacts, research/runtime, contracts/foundation]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, design, research]
---
## User-visible outcome

A concrete consumer-neutral design explains how statically linked backend components compose from compilation through execution without one monolithic `Device` trait and without requiring custom implementations to maintain Tiler forks.

## Why this slice exists

The corpus defines semantic and lowering providers but does not define `BackendProvider`, provider bundles, emitter registration, runtime-adapter registration, partial backend reuse, or cross-backend selection. The Metal and CPU spikes must constrain this design before any public abstraction is accepted.

## Implementation keys

- Synthesize only requirements exercised by both concrete spikes or forced by accepted correctness contracts.
- Separate target-profile authority, physical implementation proposals, backend emission/artifact production, and live runtime adaptation. State which pieces may be supplied independently and how a partial provider reuses another backend's pieces.
- Keep build-time producers and runtime adapters independently installable and join them only through governed backend, representation, target-profile, payload schema, compatibility contract, entry mapping, and execution-policy identities.
- Carry and validate producer/provider provenance separately without presuming it equals the independently selected runtime-adapter identity. The responsibility matrix must identify which subjects are compared, which are retained only as provenance, and which are selected independently.
- Define explicit per-session builders and immutable frozen registries; forbid global discovery, registration-order precedence, last-wins replacement, and ambient provider mutation.
- Preserve propose-then-reverify, typed explain outcomes, deterministic identity, hard-feasibility versus cost separation, and the one-way routing commit.
- Specify trust and linkage: trusted Rust code statically linked into one binary; native dynamic loading, stable plugin ABI, untrusted code, hot reload, and cross-process callbacks remain deferred.
- Specify the minimum conformance obligations and every unsupported case.
- Provide small end-to-end examples for standard Metal plus a partial custom provider and for a CPU backend.

## Closes when

The research record contains an exact responsibility/identity/lifecycle matrix, concrete interface sketches grounded in the spikes, eliminated alternatives, a proposed dependency direction, and the atomic decisions a durable ADR must make.

## Graph maintenance

- Release `draft-the-backend-provider-composition-adr` only after both evidence spikes and the vocabulary contract are complete.
- File narrow feasibility tickets for any missing verifier or identity authority; do not hide them inside a proposed universal abstraction.
- Keep public visibility unchanged in this ticket.
