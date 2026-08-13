---
id: accept-the-atomic-subgroup-realization-surface
title: Accept the atomic subgroup realization surface
status: awaiting-decision
priority: p1
dependencies: []
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

## Recommendation

Accept as drafted. The spelling follows the accepted model: one checked subject, separate normative and measured constructors, silence-as-absence so standard profiles keep their bytes, and `v6` only on the feasibility rule-set key because `assess` now decides a predicate `v5` could not express. **Strongest counterpoint:** adding `subgroup` to the public `ResourceRequirements` struct is a breaking field addition even for callers who only construct `None`.

## Closes when

Tom accepts, accepts with named exclusions, or revises.
