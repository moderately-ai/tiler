---
id: prototype-a-bounded-scalar-cpu-backend-vertical
title: Prototype a bounded scalar CPU backend vertical
status: in-progress
priority: p1
dependencies: []
related: [target-profile-feasibility-model, runtime-execution-contract, reference-evaluator-slice]
scopes: [research/target-profiles, research/artifacts, research/runtime, contracts/artifacts]
shared_scopes: [research/program-planning, contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, pluggability, cpu, spike]
claimed_from: todo
assignee: loop-prototype-a-
lease_expires_at: 1785518570
---
## User-visible outcome

A retained executable spike carries one bounded scalar CPU implementation from a declared CPU target profile through verified physical work, an independently identified executable representation and artifact payload, device/context preflight, execution, and bitwise comparison with `tiler-reference`.

## Why this slice exists

The CPU/SIMD contract is proposed and implementation is not started. A second materially different backend is needed before a generic provider interface can be trusted not to encode Metal's execution hierarchy. The reference evaluator executes on a CPU but is not a physical CPU backend and must not be relabelled as one.

## Implementation keys

- Implement the smallest real scalar program already admitted by the semantic and reference layers; do not scaffold a production `tiler-cpu` crate.
- State a bounded CPU target profile with target triple, ABI/data layout, address width, scalar execution model, and exact operation/dtype support; vector and threading claims remain absent and therefore `Unknown`.
- Consume verified structured KIR or record precisely why its current vocabulary cannot express the CPU realization.
- Define a governed backend key and executable representation distinct from Metal and from the reference evaluator.
- Package and decode a real payload, validate it against a device-free environment, then bind it to a live host execution context before dispatch.
- Compare exact results with `tiler-reference`, while keeping reference identity and backend identity separate.
- Perturb target facts, representation, payload identity, and output behavior and watch the corresponding checks fail.
- Retain a reproducible spike harness and result fixture.
- Do not edit production crates in this spike. File any evidence-backed production blocker as a separate ticket with its own scope and public-boundary review where required.

## Closes when

One scalar CPU payload executes through the recorded vertical or the spike identifies a precise architectural blocker, every unsupported vector/thread/dtype feature rejects explicitly, and no production support claim or permanent crate admission is made.

## Graph maintenance

- Feed the measured CPU requirements into `specify-the-consumer-neutral-backend-provider-composition-contract`.
- Keep Q-PLAN-011 and `docs/backends/cpu.md` explicitly proposed until a later accepted implementation plan.
- Do not satisfy this ticket by calling `ReferenceEvaluator` the backend implementation.
