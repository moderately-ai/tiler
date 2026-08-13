---
id: admit-an-atomic-subgroup-realization-subject-to-target-profiles
title: Admit an atomic subgroup realization subject to target profiles
status: in-progress
priority: p1
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, decide-the-prepared-subgroup-width-equality-gate]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/optimizer, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, subgroup, feasibility, identity, public-boundary, fail-closed]
claimed_from: todo
assignee: worker-atomic-subgroup-realization
lease_expires_at: 1786653865
---
## User-visible outcome

A target profile can state one exact subgroup realization without letting independently sourced width, dtype, and transfer facts form a realization nobody observed.

## Accepted boundary — 2026-08-11

Tom accepted one checked `SubgroupRealizationSubject` containing literal `SubgroupWidth`, exact `ArithmeticType`, and an operation-specific `SubgroupTransfer`, initially `InRangeXorShuffle`. Whole-subject equality is the only positive match. `Realized` and `Unrealizable` are explicit; silence and neighbours are `Unknown`.

Normative and measured declarations are separate methods with their existing distinct source authorities. There are no per-field setters, boolean support flag, default row, inherited target-family row, or generic wrong-backend guess.

## Required delivery

- Re-read every target-profile fact family, synchronization whole-subject precedent, feasibility proposal, resource requirement, KIR identity, explain consumer, descriptor encoder, and artifact/cache pin at the implementation base.
- Use private fields plus checked constructors/getters. Reject zero/unsupported width forms and any transfer/arithmetic combination the subject cannot define.
- Add the required subject to `ResourceRequirements` and the exact feasibility predicate. `None` means the program requires no subgroup realization and emits no predicate row; it never means a default subgroup.
- Encode every subject dimension, support verdict, availability phase, authority, validity, and source. Sort deterministically, reject exact duplicates/contradictions, and preserve silence as `Unknown`.
- Add separate normative and measured builder methods. Generic construction validates provenance and structure; backend-family correspondence remains in the backend-owned binding layer.
- Advance the governed feasibility rule-set identity and rederive all nested/outer domain steps. Update request, explain, descriptor, artifact, envelope, and cache pins from the exact tree.
- Perturb width, arithmetic type, transfer, support verdict, phase, source, and silence independently. Quote the distinct failure or `Unknown` result for each.
- Keep every standard profile silent until its own evidence ticket and prepared-entry gate complete.

## Performance boundary

The hot host path adds one fixed-struct comparison and one tiny declared-row lookup per subgroup candidate. No kernel executes this code. Prefer sorted exact lookup; measure before adding indexing machinery.

## Closes when

The public profile vocabulary can state and resolve one atomic subgroup realization, no partial conjunction can satisfy it, and no standard profile gains an unsupported row.
