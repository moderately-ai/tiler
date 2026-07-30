---
id: replace-or-justify-the-barrier-count-axis
title: Replace or justify the barrier-count capability axis
status: in-progress
priority: p0
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [synchronization, feasibility, target-profiles, correctness]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785534750
---
## User-visible outcome

Target feasibility describes the synchronization a schedule actually requires with semantics a backend and target authority can prove. A zero-barrier schedule is admitted vacuously, while nonzero synchronization is never represented by an invented numeric capacity.

## Facts and measurement boundary

**Fact:** `CapabilityAxis::Barriers` is currently a `u64` count with an `AtMost` relation. The governed profile offers 0 and the bounded serial-sum schedules require 0. No inspected Apple source states a maximum number of barrier operations, and language support for a barrier spelling does not establish a numeric capacity.

**Inference:** counting barrier operations loses the distinctions that determine correctness: memory scope, execution scope, convergence, visibility, collective participation, and ordering. Conversely, requiring an authority row to prove `0 <= available` makes a schedule with no synchronization depend on a fact it does not consume.

**Measurement boundary:** this ticket must not infer a target-wide synchronization guarantee from a kernel that happens to compile. If a genuine finite numeric consumer and authority exist, measure and state them exactly; otherwise the numeric axis is the wrong model.

## Implementation and experiment keys

Audit every construction and consumption site for the barrier count. Either document a real bounded resource whose required and available quantities share exact semantics, or replace the axis with typed barrier scopes, kinds, convergence/participation obligations, and visibility capabilities. Make an absent synchronization requirement vacuously satisfied without manufacturing a target fact. Preserve hard-feasibility diagnostics and canonical identity. Tom must review a changed public or durable capability boundary.

## Required evidence

A one-line source audit must enumerate every current barrier-count construction. Tests must show a zero-barrier proposal succeeds without a barrier capability row, an actually synchronized proposal is `Unknown` or rejected until every typed obligation has authority, and mismatched scope/convergence/visibility cannot satisfy one another. Identity mutation tests must cover every retained synchronization dimension.

## Closes when

The count has either a reproducible real authority and consumer or is fully replaced by the typed synchronization contract, zero is vacuous rather than an invented capability fact, all explain and identity paths agree, focused tests and `make check` pass, and Tom has reviewed consequential public changes.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep the parent focused on constructing a real profile; this ticket owns deciding whether the barrier row exists at all and must close before that profile can truthfully enumerate every consumed quantitative fact.
