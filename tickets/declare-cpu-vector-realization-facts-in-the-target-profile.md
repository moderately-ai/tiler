---
id: declare-cpu-vector-realization-facts-in-the-target-profile
title: Declare CPU vector realization facts as atomic target facts
status: todo
priority: p2
dependencies: [accept-adr-0093-cpu-vector-lane-tier]
related: [design-the-cpu-vector-lane-tier, name-a-host-process-availability-phase]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, feasibility, cpu, simd, provenance, public-boundary]
---
## User-visible outcome

A target profile can declare which vector realizations it provides, and a lane-bound schedule composes against that declaration to one of ADR 0043's four outcomes — so a predicated tail is `Rejected` with a named reason on a target with no masked load, rather than `Unknown`.

## Why an atomic subject and not a capability axis

**Fact.** `CapabilityAxis` is the quantitative space: every axis has a `u64` bound, a `Quantity` unit, and a comparison `Relation`. The relation a lane width would take is `AtMost`, and `satisfies(AtMost, 4, 8)` is true — so a profile declaring 8 lanes would admit a schedule requiring 4. **That is unsound**: realizing 8-lane arithmetic is not realizing 4-lane arithmetic, and on a scalable ISA at a given implemented length there may be no fixed narrower form at all. The design and its counterexamples are [CPU vector realization facts](../docs/research/target-profiles/cpu-vector-realization-facts.md).

## Implementation keys

- **One `VectorRealizationSubject` matched by equality**, carrying lane shape, element `ArithmeticType`, operation class, masking, address space, and alignment requirement. No per-dimension accessor and no per-dimension declaration method — the discipline `declare_synchronization_realization` already states ("no `declare_barrier_execution_scope`, no `declare_fenced_spaces`").
- **Two-valued verdict, `Realized` or `Unrealizable`**, for `SynchronizationRealization`'s stated reason: a profile that could only stay silent about what it cannot do would make "unsupported" and "unmeasured" one state.
- **No deferred query path.** `resolve_synchronization` has no `Later` arm because no query vocabulary can ask a device such a question; the same holds here, so the unresolved case is `Unknown` and fails closed.
- **Facts are `CompileProfile`, authority `ExternalProfile` (a cited vendor architecture specification) or `GovernedProfile` (a base-architecture guarantee).** A row must never be sourced from the compiling host: the compile host is not the execution host. Host feature detection stays in the runtime's existing device-free variant-eligibility filter, which the bounded scalar CPU vertical already exercised.
- **Whether the numerical honourability key gains an execution-path component is a question for Tom, not a deliverable of this ticket.** The class-level packed-versus-scalar divergence that would have forced it is refuted on both ISAs examined — AArch64's `FPCR.FZ`/`FIZ` and x86's `MXCSR.FTZ`/`DAZ` each govern both paths — and only per-instruction cases survive (the `FPCR.AH` reciprocal-estimate family, the min/max exemptions, `FZ16`, and x87's absence of `FTZ`/`DAZ`). If Tom takes the widening it steps `PROFILE_DESCRIPTOR_DOMAIN` and mints a new `GOVERNED_FEASIBILITY_RULE_SET_KEY` rather than bumping its revision, by the rules those constants' own documentation states. **Do not implement it without that decision**, and do not treat the scalar-epilogue obligation as blocked on it: one row covering both paths discharges the obligation on every target that shares a control.
- **`ResourceRequirements` gains an `Option`-shaped vector requirement** whose `None` is a canonical absence — no requirement, no query, no explain row, no artifact field — following the synchronization field's stated discipline.
- **The scalable vector length is not a fact.** No legality predicate consumes it; it is a cost input. Declaring it would create a compile-time claim about a value the compile target does not fix, and it is what keeps this tier from firing [`name-a-host-process-availability-phase`](name-a-host-process-availability-phase.md).

## Required failure-path evidence

A profile declaring a subject differing in exactly one dimension from the required one, once per dimension, each resolving to `Unknown` rather than satisfying it — the check that the subject is matched whole. A profile declaring `Unrealizable` for the required subject, resolving to `Rejected` with the reason named. A profile silent on the subject, resolving to `Unknown`. A profile declaring one subject both `Realized` and `Unrealizable` at one phase, refused at construction. A numerical row that does not honour a dimension the region declares, refusing the lane schedule whose arithmetic would run under it.

## Non-goals

Constructing a real CPU profile, taking any measurement, any emission, any backend crate, and any threading or cache axis.

## Closes when

The subject and its declaration path exist, feasibility composes it to all four outcomes, every check above is observed failing against an accepted neighbour, and no row anywhere is sourced from the compiling host. The honourability-key question is reported to Tom rather than answered here.
