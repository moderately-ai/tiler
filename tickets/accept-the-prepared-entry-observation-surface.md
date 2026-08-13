---
id: accept-the-prepared-entry-observation-surface
title: Accept the prepared-entry observation surface
status: awaiting-decision
priority: p1
dependencies: []
related: [make-prepared-entry-observations-typed-and-key-dispatched]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft prepared-entry observation surface so a second legal property key cannot be answered by an unrelated pipeline quantity.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes a new public type and a trait-return change to Tom. [`make-prepared-entry-observations-typed-and-key-dispatched`](make-prepared-entry-observations-typed-and-key-dispatched.md) landed a tested labelled draft at `b19bc60dd7a6f6bd35d880472885b62b2bf374d1`. This node is not implementation work. Only Tom closes it.

## The surface, as landed at `b19bc60d`

**Included.**

- `PreparedEntryObservation::{Quantity(u64), Unrecognized}` — exhaustive, not `#[non_exhaustive]`, labelled draft under ADR 0075.
- `PreparedEntryPropertySubject` — owned key, provider namespace/name/revision, required quantity, and `TargetPropertyRequirementRelation`.
- `RuntimeAdapter::observe_prepared_entry` now returns `PreparedEntryObservation` rather than `u64`. There is no compatibility method that maps an unknown property to a number.
- `LoadRejection::UnownedPreparedEntryProperty` for unknown provider namespace, name, revision, or property key.
- `LoadRejection::UnsatisfiedDeferredPredicate` now carries `subject` and `observed`, so a measured miss names the property and the quantity.

Adapters exact-match provider namespace, name, revision, and property key before reading a pipeline quantity. The loader still applies the relation.

**Excluded.**

- `Feature` on a prepared-entry observation (these properties are quantitative).
- Numeric sentinels, `Option`, or a catch-all success.
- Adapter-owned satisfaction verdicts.
- Self-acceptance.

## Recommendation

Accept as drafted. The old `u64` return is how a second legal key is admitted when an unrelated quantity equals the required value; `Unrecognized` vs `Quantity` is the same split already accepted for live-device rows. **Strongest counterpoint:** widening `UnsatisfiedDeferredPredicate` with `subject` and `observed` is a breaking field addition on an existing public variant, even for callers who only matched `variant`/`predicate`/`entry`.

## Closes when

Tom accepts, accepts with named exclusions, or revises. Do not treat the implementation merge as an accepted surface on this packet alone.
