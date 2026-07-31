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

## What the Metal spike supplies

From `prototype-a-forkless-custom-metal-physical-provider`, evidence at commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a`, harness at [`spikes/extensions/forkless-physical-provider/`](../spikes/extensions/forkless-physical-provider/README.md). Six constraints this contract can now treat as measured rather than assumed.

**The seam to design is registration and re-verification, not a way to express an implementation.** A proposal body is a `tiler_ir::schedule::ScheduledRegion`, already fully public and constructible from an out-of-workspace crate. Whatever the contract names, it does not need a new implementation vocabulary.

**Partial reuse of Metal is already available and needs nothing from this contract.** `tiler-metal` does not depend on `tiler-compiler`; it consumes verified kernels and knows nothing about who proposed them. A provider crate reuses `lower_scheduled_region` and `emit_translation_unit` unchanged. So "how does a partial provider reuse another backend's pieces" has a concrete answer for the emission piece — it depends on `tiler-ir` and the emitter crate directly, and the composition contract does not mediate that edge.

**Visibility and installation are two separate obligations, and only one is on anyone's list.** The spike's compile-fail evidence shows that publishing `frontier::PhysicalImplementationProvider` would still leave a provider uninstallable: the provider array is a hardcoded literal at `pipeline/planning.rs:171` and the internal request carries no provider field. ADR 0078 item 4 recorded exactly this asymmetry for lowering providers and closed it with `CompileRequest::with_capabilities`; the contract must state the physical analogue explicitly rather than assume a `pub` keyword suffices.

**Observability is a third obligation.** `Compilation::offered_providers` reports lowering providers only, and no public type carries physical-provider provenance. The responsibility matrix therefore owes a disclosure rule: which provider identities a compilation reports, and whether an installed-but-never-selected provider is visible. Today neither is answerable.

**The specialization axis is the schedule, and it is identity-bearing.** `verify_region_subject_binding` compares region id, iteration shape, scalar program, semantic members, and access map, and says nothing about `KernelSchedule`; `threads_per_workgroup` is free under the intrinsic verifier and folded into `CanonicalScheduledRegionIdentity`. Two alternatives of one region therefore carry distinct identities and emit distinct entry-point symbols from identical bodies. That is the concrete shape of "several providers' implementations retained side by side", and it means the additivity claim needs no new identity authority for this case.

**Propose-then-reverify is already partly public, and the split is uneven.** Of the gates `verify_schedule_with_feasibility` runs, only the intrinsic verifier (`ScheduledRegionBuilder::build`) is reachable from outside; the request-authority check, the numerical-realization comparison, the subject binding, and the feasibility assessment are private. The contract must say which of those an out-of-crate provider may pre-run and which stay host-only, because a provider that can pre-run none of them cannot report a typed local failure of its own.

## Graph maintenance

- Release `draft-the-backend-provider-composition-adr` only after both evidence spikes and the vocabulary contract are complete.
- File narrow feasibility tickets for any missing verifier or identity authority; do not hide them inside a proposed universal abstraction.
- Keep public visibility unchanged in this ticket.
