---
id: replace-or-justify-the-barrier-count-axis
title: Replace or justify the barrier-count capability axis
status: review
priority: p0
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer, implementation/artifact, contracts/artifacts, contracts/navigation, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [synchronization, feasibility, target-profiles, correctness]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785534750
---
## User-visible outcome

Target feasibility describes the synchronization a schedule actually requires with semantics a backend and target authority can prove. A zero-synchronization schedule is admitted vacuously, while nonzero synchronization is never represented by an invented numeric capacity.

## Facts and measurement boundary

**Fact:** the superseded implementation represented `CapabilityAxis::Barriers` as a `u64` count with an `AtMost` relation. The governed profile offered 0 and the bounded serial-sum schedules required 0. No inspected Apple source states a maximum number of barrier operations, and language support for a barrier spelling does not establish a numeric capacity.

**Inference:** counting barrier operations loses the distinctions that determine correctness: memory scope, execution scope, convergence, visibility, collective participation, and ordering. Conversely, requiring an authority row to prove `0 <= available` makes a schedule with no synchronization depend on a fact it does not consume.

**Measurement boundary:** this ticket must not infer a target-wide synchronization guarantee from a kernel that happens to compile. If a genuine finite numeric consumer and authority exist, measure and state them exactly; otherwise the numeric axis is the wrong model.

## Implementation and experiment keys

The source audit eliminated the numeric-axis alternative: no construction site produces a nonzero schedule requirement, no authority defines a count capacity, and the only KIR barrier is a negative verifier fixture. Remove `CapabilityAxis::Barriers`, both public target-profile builder methods, and `ResourceRequirements::barriers`. A schedule with no synchronization emits no feasibility requirement, target fact, explain row, or artifact field.

Preserve `BarrierSpec` as a typed KIR reservation, but reject every current barrier intrinsically as `UnexpectedSynchronization`: the current schedule owns no identity-bearing synchronization point, phase, placement, participant set, visibility contract, or convergence proof to which the operation could be matched. Reserve the retired capability tag and advance every identity or fixed record whose canonical bytes change.

The first real nonzero synchronization path is split into `admit-the-first-typed-synchronization-point-and-atomic-target-authority`. That ticket must introduce the complete schedule obligation and one atomic provenance-bearing target realization together; independently asserted component facts are not composable evidence.

## Required evidence

The one-line source audit `rg -n 'CapabilityAxis::Barriers|declare_(measured_)?barriers|barriers:' crates prototypes --glob '*.rs'` must return no construction sites after the correction. Tests must show a zero-synchronization proposal succeeds without a synchronization fact or explain row, the target quantity parser rejects the retired name, and an actually synchronized KIR body is intrinsically rejected before target feasibility. Kernel, target-profile, artifact, and manifest identities must be rebaselined on this tree, and old manifest schema 7 must reject rather than be reinterpreted.

## Closes when

The unsupported count and its public construction methods are absent; zero synchronization is vacuous; current nonzero synchronization fails closed with a stable intrinsic diagnostic; explain and identity paths agree; the complete typed nonzero contract is dependency-tracked; focused tests and `make check` pass; and Tom has reviewed the consequential public changes.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep the parent focused on constructing a real profile; this ticket owns deciding whether the barrier row exists at all and must close before that profile can truthfully enumerate every consumed quantitative fact.

When this ticket closes, remove the barrier row from the parent profile's evidence ledger and release it without adding a replacement synchronization fact. The typed first-nonzero ticket is deliberately non-blocking for that zero-synchronization profile.
