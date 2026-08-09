---
id: resolve-the-provisional-normal-scale-discharge-public-surface
title: Resolve the provisionally accepted normal-scale discharge public surface
status: todo
priority: p1
dependencies: [admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode]
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [public-api, numerics, decision]
---
## User-visible outcome

The normal-scale discharge API is either accepted by Tom with its exact included
and excluded surface recorded, or every still-unaccepted exported item is
truthfully labelled as a draft. Provisional coordinator acceptance is not
presented as final public-boundary authority.

## Facts audited at `a5eebb43` on 2026-08-09

**Verified — the implementation ticket records provisional authority only.**
The Outcome of
[`admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`](admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode.md)
ends at the source-safe anchor `Provisional boundary acceptance (2026-08-01,
overnight mode)` and says the coordinator recorded the surface for Tom's morning
review. No later ticket or accepted ADR names Tom accepting these exact items.

**Verified — six public symbols are in the current surface.** Full reads of
`crates/tiler-ir/src/semantic/precondition.rs`,
`crates/tiler-ir/src/semantic/quantization.rs`,
`crates/tiler-ir/src/schedule/numerics.rs`,
`crates/tiler-ir/src/schedule/model.rs`, and
`crates/tiler-ir/src/kernel/model.rs` locate:

- `pub fn positive_normal_scalar_predicate`;
- `pub const ENCODED_NUMERIC_SCALE_DOMAIN`;
- `pub enum SubnormalFreedom`;
- `pub const fn discharges` on that enum;
- `pub const fn subnormal_freedom` on `VerifiedScheduledRegion`; and
- `pub const fn subnormal_freedom` on `VerifiedKernel`.

The implementation ticket calls these five grouped items because it groups the
enum with its method. This ticket uses the six-symbol population so a method
cannot silently escape the decision.

**Verified — none is currently labelled as a draft.** Their complete rustdoc
blocks explain semantics, derivation, measurement, and identity, but contain no
`Draft public surface`, `not yet accepted`, or equivalent marker. The root agent
guide requires a tested public boundary to remain labelled draft until Tom
accepts its exact included and excluded surface.

**Verified — this is a public-authority repair, not a numerical change.** The
normal-scale predicate, derived `f32`-only discharge, Metal honourability result,
semantic contract field, and identity consequences are implemented and tested.
No evidence in this audit disputes them. Labelling the surface does not change
program bytes, request identity, artifact identity, dispatchability, or admitted
values.

## Work

1. Present Tom with the exact six-symbol population above, including the
   deliberate exclusions already recorded by the implementation ticket: the
   freedom is derived rather than caller-settable, is `f32`-only, and is not
   redundantly encoded into kernel identity.
2. If Tom accepts it, record who, date, venue, relay source, and the exact
   included and excluded surface in the owning durable record and update all six
   rustdoc sites consistently.
3. If that acceptance is not available in this work, add an explicit draft
   marker to every public rustdoc block without changing signatures or behavior.
4. Census the six exact symbols and the six corresponding accepted-or-draft
   statements. Make a subject perturbation by removing one statement and quote
   the failing population check before restoring it.

## Non-goals and stop conditions

Do not change predicate meaning, target facts, schedule derivation, numerical
realization, identity encoding, or runtime enforcement. Do not infer acceptance
from the completed implementation, its tests, or coordinator provenance. Stop
for Tom if the decision would add, remove, rename, or otherwise reshape any
public item beyond documenting the exact existing surface.

## Closes when

All six exported symbols carry one coherent, truthful authority status; the
acceptance provenance or draft status is recorded durably; the six-item census
has been deliberately reddened by removing one subject statement and restored;
targeted `tiler-ir` rustdoc and tests pass; and `tkt lint`, `make citations`,
`git diff --check`, the exact-base scope guard, and the required final gate pass.
