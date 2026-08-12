---
id: prove-planned-binding-alignment-before-routing-commit
title: Prove planned binding alignment before routing commit
status: blocked
priority: p1
dependencies: [derive-artifact-binding-alignment-from-selected-access-requirements, make-prepared-entry-observations-typed-and-key-dispatched]
related: [promote-the-bounded-scalar-cpu-vertical-into-a-production-backend, carry-the-binding-offset-through-the-runtime-route]
scopes: [implementation/artifact, implementation/runtime, implementation/cpu, implementation/metal, implementation/candle, implementation/frontend, implementation/conformance, research/runtime, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [alignment, runtime, preflight, cpu, fail-closed, public-boundary]
---
## User-visible outcome

The loader proves every selected binding's actual or allocator-guaranteed alignment before the one-way routing commit; an unknown or insufficient address may fall back, while an allocator that breaks its accepted guarantee fails terminally after commit.

## Facts at filing base `f199b26376612e4b39c35569b084dda4c67490ce`

- **Verified.** `RoutedBinding` already publishes the decoded binding, transport slot, evaluated accessible offset, and accessible extent. The decoded binding exposes the required alignment.
- **Verified.** `RuntimeAdapter::plan_dispatch` is the last pre-commit method and already owns storage placement/capacity planning, but returns only `()`. The loader therefore has no alignment observation to compare.
- **Verified.** Caller-supplied storage exists before commit and can be observed without allocation. Adapter-owned output/internal storage is allocated only after commit under ADR 0051, so only its allocator contract can be reported beforehand.
- **Verified.** The runtime module's ownership rule is that adapters report and the loader compares. Letting each adapter decide alignment internally would violate the same rule already applied to live-device and prepared-entry requirements.
- **Verified.** Runtime-adapter implementations currently occur in `tiler` facade tests, runtime integration tests, the Candle prototype, and the retained inline-dispatch spike. A trait change reaches frontend and research-runtime scopes even though none of those test/prototype adapters becomes the CPU consumer.

## Required public carrier

- Change `RuntimeAdapter::plan_dispatch` to return a complete `PlannedBindingAlignments` report in addition to its adapter-private retained plan state. Do not add an independent registry or a second planning pass.
- Each exact route entry/slot has one `BindingAlignmentEvidence`: `ObservedAddress(AlignmentGuarantee)`, `AllocatorGuaranteed(AlignmentGuarantee)`, or `Unknown`. This enum is the MECE acquisition state; a missing row is malformed report cardinality, not another spelling of `Unknown`.
- The report is bounded by the decoded route's existing entry/binding limits and must be in exact execution/slot order. The loader rejects missing, extra, duplicate, foreign, or reordered rows before comparing any requirement.
- The loader compares every reported guarantee against the artifact binding's `AlignmentRequirement`. `Unknown` and insufficient guarantees are distinct typed `LoadRejection`s and remain fallback-permitted. No zero, maximum integer, boolean support flag, pointer value, or adapter verdict crosses the boundary.
- `allocate_dispatch` remains after commit. Every real adapter asserts that the final allocated/suballocated address meets the guarantee it reported. A breach is an adapter-specific terminal `Failure`, never converted to a recoverable alignment miss.

## Real consumers

`tiler-cpu-runtime` observes the actual final address for caller-retained host storage and reports the exact guarantee of its allocator/suballocation policy for future storage. The retained NEON route must pass from both exactly four-byte and stronger 16-byte host addresses and refuse an insufficient or unknown address before executing one instruction. `tiler-metal` must derive a guarantee from its real buffer/offset contract or return `Unknown`; no Metal constant is accepted without owning API evidence. Candle prototypes receive only exhaustive mechanical updates and do not count as a consumer or correctness proof.

## Required evidence

- Make incomplete, duplicate, foreign, and reordered reports fail by distinct loader causes.
- Prove equal and stronger guarantees pass, weaker and `Unknown` refuse before `Preflight::commit`, and each refusal reports entry, slot, required, and observed/acquisition state.
- Perturb an external CPU address to exact natural and stronger alignment without changing bytes or artifact; both execute and match independently.
- Perturb an adapter allocator so the post-commit address violates its reported guarantee; assert terminal failure and `fallback_permitted() == false`.
- Prove no program allocation occurs while constructing the report and no alignment verdict is cached across artifact identity, entry, storage plan, or invocation.
- Exercise every in-tree `RuntimeAdapter` implementation. Unknown keys/states refuse; no implementation returns an optimistic constant just to compile.

## Identity and performance

Runtime evidence is invocation-local and enters no artifact, cache, or canonical identity. Work is one bounded linear pass over routed bindings plus O(1) power-of-two comparisons. No kernel/device operation is added, and ordinary compilation pays nothing.

## Non-goals

Selecting a fallback automatically, allocating before commit, exposing raw addresses in neutral APIs, changing artifact schema, inferring target support, or treating Candle as the CPU backend.

## Closes when

The loader owns a complete pre-commit alignment proof for every binding, all real adapters answer honestly, the CPU allocator asserts its promise after commit, and the retained real CPU route exercises both stronger satisfaction and failure timing.
