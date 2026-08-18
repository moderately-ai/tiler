---
id: decide-the-adr-0013-plan-determinism-stability-subject
title: Decide the ADR 0013 plan-determinism stability subject
status: in-progress
priority: p1
dependencies: []
related: [decide-the-semantic-order-contract-for-relaxed-contractions]
scopes: [implementation/ir, implementation/artifact, implementation/compiler, implementation/runtime, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, identity, public-boundary, needs-tom]
claimed_from: todo
assignee: worker-determinism-surface
lease_expires_at: 1787017405
---
## Outcome

Decide the exact identity-bearing public subject that realizes ADR 0013 plan determinism before any relaxed contraction topology can claim deterministic evaluation. This is decision research, not implementation authorization.

## Exact decision

The accepted semantic scope is fixed: identical input bits and runtime bindings, the same artifact digest and selected plan variant, and the same declared target environment must produce identical output bits; recompilation or another artifact may choose a different legal result. Decide the currently unresolved declared target-environment compatibility identity, its construction and verification owners, selected-topology binding, public types and errors, durable schema/domain/version/pin cascade, and runtime refusal path.

## Readiness gate

Use the strongest reasoning model. Re-audit ADRs 0012 and 0013 at the exact base and read target, schedule, kernel-program, artifact manifest/codec, explain, cache, and runtime construction/consumption/refusal paths. Apply the full Pareto-complete decision gate: status quo typed refusal, the narrowest exact target-environment subject, a complete replacement if current target identity is insufficient, bounded research, and deferral. Eliminate any option that invents/defaults environment compatibility, leaves selected topology unbound, conflates artifact identity with live device identity, or claims a schema-complete outcome with unresolved fields.

## Required evidence

- Exact public fields, constructors, accessors, verification errors, and owner for the stability subject.
- Complete request/schedule/kernel/artifact/explain/cache/runtime identity and schema consequences.
- Subject perturbations for artifact digest, selected variant, target environment, and topology, plus a negative execution control for run-dependent selection.
- Strongest counterargument and reversal evidence for every frontier survivor.

## Boundary

Do not implement the result or authorize relaxed contraction semantics. The already-filed [`implement-the-adr-0013-plan-determinism-stability-subject`](implement-the-adr-0013-plan-determinism-stability-subject.md) remains blocked on this decision and may implement only the exact subject Tom accepts. [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) depends on that carrier, so a relaxed-contraction implementation cannot proceed until both the decision and its implementation are complete.
