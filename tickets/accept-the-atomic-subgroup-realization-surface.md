---
id: accept-the-atomic-subgroup-realization-surface
title: Accept the atomic subgroup realization surface
status: todo
priority: p1
dependencies: [minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance]
related: [admit-an-atomic-subgroup-realization-subject-to-target-profiles]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft Rust spelling of the atomic subgroup realization he accepted as a model on 2026-08-11.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public types and builder methods to Tom. The 2026-08-11 packet on [`admit-an-atomic-subgroup-realization-subject-to-target-profiles`](admit-an-atomic-subgroup-realization-subject-to-target-profiles.md) accepted the *model*. This node is the spelling landed at `5cd61fbe` (rebased tip `eecc4002`). Only Tom closes it.

## Readiness correction — 2026-08-13 at `4fb0427319b1504e1549e03ba023ac486343a743`

The packet below is **not ready for Tom**. Its Included list is not the exact landed public surface: it omits public identity/tag helpers, the lookup result and method, the duplicate-declaration error, and exact trait implementations. Independent review also found a public decoder with no production consumer, an unreachable public error variant, and no `Some(subgroup)` kernel-identity test. [`minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance`](minimize-and-prove-the-atomic-subgroup-public-surface-before-acceptance.md) owns the source and evidence repair; this ticket depends on it and will be rewritten against its exact reviewed commit. The recommendation below is retained as the superseded draft, not as a live decision request.

## The surface, as landed at `5cd61fbe`

**Included.**

- `tiler_ir::schedule::{SubgroupWidth, SubgroupTransfer::InRangeXorShuffle, SubgroupRealizationError, SubgroupRealizationSubject}` with private fields, fallible `new`, and getters.
- `ResourceRequirements.subgroup: Option<SubgroupRealizationSubject>`. `None` means no requirement and emits no predicate row.
- `TargetProfileBuilder::{declare_subgroup_realization, declare_measured_subgroup_realization}` and `SubgroupSupport::{Realized, Unrealizable}`.
- Whole-subject equality as the only positive match. Silence and neighbours are `Unknown`.

**Excluded.**

- Per-field setters, a boolean support flag, a default row, an inherited target-family row, or a generic wrong-backend guess.
- KIR subgroup emission or deriving `Some` from an admitted topology.
- A row on the governed or standard Metal profile.
- Encoding a present subject in the artifact resource record (current codec still drops `subgroup` / decodes `None`).
- Stepping `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` or `KERNEL_DOMAIN`. Silent profiles write no section.

## Superseded draft recommendation

Accept as drafted. The spelling follows the accepted model: one checked subject, separate normative and measured constructors, silence-as-absence so standard profiles keep their bytes, and `v6` only on the feasibility rule-set key because `assess` now decides a predicate `v5` could not express. **Strongest counterpoint:** adding `subgroup` to the public `ResourceRequirements` struct is a breaking field addition even for callers who only construct `None`.

## Closes when

The dependency rewrites this packet against the exact repaired surface and the complete readiness gate, then Tom accepts, accepts with named exclusions, or revises.
