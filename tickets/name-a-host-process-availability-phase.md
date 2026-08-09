---
id: name-a-host-process-availability-phase
title: Name a host-process availability phase
status: deferred
priority: p3
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, artifacts, cpu, vocabulary, decision, needs-tom, public-boundary, deferred]
---
## User-visible outcome

A capability fact that becomes readable once a host *process* is bound has a phase of its own, rather than borrowing `LiveDevicePreflight`.

## Why this slice exists

**Fact.** `AvailabilityPhase` is defined once, at `crates/tiler-ir/src/program/abi.rs:120`, with five variants — `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, `LaunchPreflight` — and re-exported through `tiler-artifact` and consumed by `tiler-compiler`. Its order is total and load-bearing: a use site evaluated at one phase may name only roots available no later than it.

**Measurement.** Finding 5 of the [bounded scalar CPU backend vertical](../spikes/target-profiles/scalar-cpu-vertical/README.md): that backend's second-stage facts are properties of the running process — architecture, pointer width, byte order, and the subnormal behaviour of its actual floating-point arithmetic — and had to borrow `LiveDevicePreflight` because no phase names a bound process. The spike records this as a naming seam today that becomes functional the moment a profile wants to distinguish "known once a device exists" from "known once a process exists".

## Implementation keys

- Decide first whether the distinction is real or whether a CPU host *is* a live device for this vocabulary's purposes; a defensible "no phase is needed" outcome closes this ticket.
- If a phase is added, it changes a governed wire tag vocabulary with a total order. Place it by what it can *see*, not by narrative order, and treat the encoding as a versioned subject rather than an insertion.
- Every `AvailabilityPhase` match in `crates/` is exhaustive by convention 3; a new variant must be a build error at each, not absorbed by a wildcard.

## Decision packet — 2026-08-09

This is not implementation-ready: either answer changes or ratifies a governed public phase vocabulary, and adding a variant changes its tag, total order, artifact encoding, and identity consequences.

- **Option A — add `HostProcessPreflight` between `ArtifactEvidence` and `LiveDevicePreflight` (recommended).** Architecture, pointer width, byte order, and host floating-point behaviour exist once the process is bound and before any live device is available. Giving them their own phase prevents CPU facts from borrowing a device concept and keeps later device facts strictly later.
- **Option B — define the CPU host process as a live device for this vocabulary.** This avoids a tag and identity step, but permanently makes `LiveDevicePreflight` mean two different availability boundaries and weakens the phase names as explanations.

Tom's decision must accept one meaning and, for Option A, the exact placement and versioned encoding consequence. No worker should add the variant or ratify the borrowed phase before that answer.

## Closes when

Either a phase exists with its tag, order, and encoding consequences recorded and every match updated, or the question is closed with the reasoning for why the borrow is correct.

## Trigger check log

- 2026-08-09 — **not fired.** ADR 0092 names the trigger as the first CPU ISA row. The current CPU vector realization ticket explicitly excludes scalable vector length from target facts and records that its tier does not fire this phase ticket. No current target profile needs to distinguish a bound host process from a live device, so the complete packet above remains deferred until a qualifying row arrives.
