---
id: declare-cpu-vector-realization-facts-in-the-target-profile
title: Declare CPU vector realization facts as atomic target facts
status: blocked
priority: p2
dependencies: [accept-adr-0093-cpu-vector-lane-tier, admit-vector-lane-bindings-into-the-schedule-vocabulary, define-plural-operation-specific-vector-realization-requirements, carry-complete-access-alignment-requirements-on-physical-proposals, establish-vector-execution-form-numerical-authority, earn-cpu-feature-level-execution-environments-from-host-observation, canonicalize-atomic-target-realization-declarations, decide-how-vector-requirements-cross-the-artifact-boundary]
related: [design-the-cpu-vector-lane-tier, name-a-host-process-availability-phase, decide-how-vector-requirements-cross-the-artifact-boundary]
scopes: [implementation/compiler, implementation/ir, implementation/artifact, implementation/runtime, contracts/decisions, contracts/artifacts, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, feasibility, cpu, simd, provenance, public-boundary, decision, needs-tom]
---
## User-visible outcome

A target profile can declare which vector realizations it provides, and a lane-bound schedule composes against that declaration to one of ADR 0043's four outcomes — so a predicated tail is `Rejected` with a named reason on a target with no masked load, rather than `Unknown`.

## Why an atomic subject and not a capability axis

**Fact.** `CapabilityAxis` is the quantitative space: every axis has a `u64` bound, a `Quantity` unit, and a comparison `Relation`. The relation a lane width would take is `AtMost`, and `satisfies(AtMost, 4, 8)` is true — so a profile declaring 8 lanes would admit a schedule requiring 4. **That is unsound**: realizing 8-lane arithmetic is not realizing 4-lane arithmetic, and on a scalable ISA at a given implemented length there may be no fixed narrower form at all. The design and its counterexamples are [CPU vector realization facts](../docs/research/target-profiles/cpu-vector-realization-facts.md).

## Historical implementation keys — superseded by the source-first correction and accepted decision below

The bullets in this section are retained as the original packet for attribution. They are not current delivery instructions; the source-first correction and accepted decision below replace the singular carrier, flat Cartesian subject, alignment dimension, runtime premise, and numerical premise.

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

## Decision packet — 2026-08-09

The atomic equality-matched subject and builder declaration are consequential public target-profile API. Recommendation: accept the whole-subject `VectorRealizationSubject` plus one atomic declaration method, with no per-field setters and no compile-host feature inference. The optional execution-path widening remains excluded and would require its own identity decision.

## Closes when

The subject and its declaration path exist, feasibility composes it to all four outcomes, every check above is observed failing against an accepted neighbour, and no row anywhere is sourced from the compiling host. The honourability-key question is reported to Tom rather than answered here.

## Source-first corrections — 2026-08-11

The singular flat packet cannot represent the first realistic vector schedule. A predicated vector region needs arithmetic plus load and usually store realization simultaneously; the research's own SVE example declares arithmetic and masked load as two subjects. One `Option<VectorRealizationSubject>` would prove at most one and silently omit the rest.

The six-field Cartesian product also admits meaningless states such as masked lane arithmetic and fault-suppressing stores, while the broad `lane arithmetic` class overclaims operations whose ISA and numerical behaviour differ. Gather is not yet statable without index type/width. Alignment is not an equality dimension: an operand proved 32-byte aligned satisfies a 16-byte requirement. The research explicitly left alignment as a subject-versus-applicability question, so the old packet stated an unresolved Proposal as settled.

Three other premises were false or incomplete. Synchronization `None` is encoded as an explicit presence byte in KIR and artifact resource records rather than “no artifact field.” Existing variant eligibility compares a caller-stated execution environment and performs no CPUID/HWCAP qualification. Existing numerical subjects are explicitly scalar and cannot silently attest packed vector execution or a scalar epilogue's two paths.

## Accepted decision — 2026-08-11

Tom accepted a bounded-by-existing-structural-budgets canonical collection of algebraic, exact-operation vector requirements, with an arithmetic-only first implementation slice:

- `VectorShape::Fixed(nonzero literal lanes)` and `VectorShape::Scalable` remain distinct exact identities. A scalable length is cost-only and never defaults a fixed width.
- Each requirement names one legal operation-specific subject. Arithmetic, contiguous load, and contiguous store are separate closed variants with only their meaningful fields. Gather waits for index type/width semantics; ordered horizontal accumulation remains below the schedule boundary under ADR 0093.
- A region carries a sorted, duplicate-refusing collection because it may require several realizations simultaneously. Empty is canonical absence, not permission; every nonempty member must resolve.
- Target declarations match each exact operation subject atomically and state `Realized` or `Unrealizable`. Silence and every neighbouring subject are `Unknown`. No per-dimension facts, quantitative width axis, wildcard row, inherited feature, or compile-host inference exists.
- Operand alignment is a separately related applicability obligation: actual/proved alignment must meet the selected realization's stated minimum. Stronger alignment satisfies weaker. It is never whole-subject equality and is never silently assumed.
- Runtime eligibility must earn the exact feature-level execution environment from CPUID/HWCAP or equivalent backend-owned observation before matching it. A caller-stated target triple or artifact profile is not evidence.
- Vector numerical execution needs its own explicit authority. Scalar rows cannot attest packed instructions; scalar epilogues stay unavailable until one source proves both paths or distinct path facts compose explicitly.
- Do not invent a separate vector-row count cap during alpha. Existing complete profile/artifact structural bounds govern until measurement demonstrates a narrower resource need.
- The first implementation slice may carry only exact arithmetic operations whose schedule, numerical authority, and host qualification are complete. That narrow slice uses the final plural carrier and must not add temporary flat/default APIs.

Identity and artifact delivery are explicit work, not implied compatibility. The new requirement population enters schedule/KIR identity and the feasibility rule-set identity. If carried in the fixed artifact resource record it requires the corresponding major schema and artifact-domain migration; preserving non-vector bytes through a conditional side table is acceptable only after injectivity and ownership are proved. The linked tickets own these prerequisites, and this ticket remains blocked until they finish.
