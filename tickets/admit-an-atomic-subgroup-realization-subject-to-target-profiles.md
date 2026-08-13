---
id: admit-an-atomic-subgroup-realization-subject-to-target-profiles
title: Admit an atomic subgroup realization subject to target profiles
status: review
priority: p1
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, decide-the-prepared-subgroup-width-equality-gate]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/build, implementation/candle, contracts/optimizer, contracts/artifacts, contracts/decisions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, subgroup, feasibility, identity, public-boundary, fail-closed]
claimed_from: todo
assignee: worker-atomic-subgroup-realization
lease_expires_at: 1786656521
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

## Fact audit — 2026-08-13 at `0e1d976bc75dcb0723ce11086750461fd1cfdf06`

- **Verified.** `TargetProfileBuilder::declare_synchronization_realization` takes one whole `SynchronizationSubject`, refuses exact duplicates and same-key contradictions, and has no per-dimension spelling. Anchor: `no per-dimension spelling`.
- **Verified.** `complete_descriptor` writes evaluation-order, cost-row, elementary, and tree-width-policy families only when nonempty. Anchor: `Why the evaluation-order family did not step`.
- **Verified.** `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` is `tiler.target-profile.declaration.v11`. Anchor: `tiler.target-profile.declaration.v11`.
- **Verified.** `PROFILE_DESCRIPTOR_DOMAIN` is `tiler.target-profile.descriptor.v10` and writes synchronization unconditionally. Anchor: `tiler.target-profile.descriptor.v10`.
- **Verified.** At this base, `GOVERNED_FEASIBILITY_RULE_SET_KEY` was `tiler.feasibility.phased-capability-and-numerical-honourability.v5`. This delivery mints `v6` because `assess` now decides a subgroup predicate `v5` could not express. Anchor: `Widening that vocabulary mints a new key`.
- **Verified.** No standard profile declared a subgroup row. Anchor: `no target profile declares a subgroup realization subject` in ADR 0094. This ticket keeps that silence.
- **Imprecise as written against this dirty tree.** The previous worker already added `ResourceRequirements.subgroup`, `SubgroupRealizationSubject`, and kernel silence-as-absence encoding. This delivery extends that remainder rather than introducing the subject types.

## Identity blast radius

- `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays `v11`. Silent profiles write no `tiler.target-profile.subgroup-realization.v1` section.
- `PROFILE_DESCRIPTOR_DOMAIN` stays `v10`. Silent checked descriptors write no `tiler.target-profile.descriptor.subgroup-realization.v1` section.
- `KERNEL_DOMAIN` does not step. Absent subgroup requirements write nothing.
- Artifact resource encoding still drops `subgroup` and decodes `None`. Honest while every derived region is `None`; a present subject must later be append-only silence-as-absence, not an unconditional field.
- Feasibility rule-set key steps `v5` → `v6`. Artifact, envelope, and cache identities that fold the key move; descriptor bytes of silent profiles do not.

## Remainder after this delivery

- KIR subgroup emission / deriving `Some` from an admitted topology.
- Declaring a row on the governed or standard Metal profile.
- Accepting this crate's exact public spelling under ADR 0075.
- Encoding a present subgroup subject in the artifact resource record.
