---
id: carry-the-elementary-numerical-dimensions-in-the-region-realization
title: Carry the elementary numerical dimensions in the region realization
status: awaiting-decision
priority: p2
dependencies: []
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, contracts/decisions, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, feasibility, decision, needs-tom, public-boundary]
---

# Carry the elementary numerical dimensions in the region realization

## User-visible outcome

A caller that states a numerical contract permitting reciprocal replacement or approximate elementary functions gets an answer about a *target*, rather than a build-level refusal or a silently unasked question, for a program containing `tiler::silu-f32@1`.

## Why this is filed rather than done

**Fact — corrected 2026-08-09.** The obligation is no longer activation-only. `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, and `tiler::softmax-f32@1` are all admitted elementary families; each can consume `NumericalDimension::ReciprocalTransform` and `NumericalDimension::ApproximateIntrinsics`. `ELEMENTARY_UNCARRIED_DIMENSIONS` names the common omission explicitly.

**Fact — listing them would refuse a public preset for every program.** `dimension_requirements` filters by `is_consumable`, so a row entry enters the dimension into the requirement set every contract places on every target; and `unrepresentable_dimension` refuses any consumable dimension that `tiler_ir::schedule::NumericalRealization` cannot carry, which neither is. `session::NumericalContract::RelaxedF32` authorizes both, so it would become unrepresentable — for programs with no activation in them as well.

**Fact — the omission is a checked claim rather than a gap.** `policy::ELEMENTARY_UNCARRIED_DIMENSIONS` names both, and `the_uncarried_elementary_dimensions_are_outside_the_realization` fails the moment the region realization grows to carry either, which is the condition under which withholding the rows stops being honest.

**Fact — what holds the obligation meanwhile is backend-local.** `crates/tiler-metal/src/emit.rs` writes `precise::exp` and the `/` operator rather than a fast intrinsic or a reciprocal multiply, and records `MetalNumericalRequirement::PreciseFp32Functions`. That is a guarantee over the operations actually emitted; it is not a profile-level assessment, and it speaks for one backend.

## Required delivery

- Widen `tiler_ir::schedule::NumericalRealization` to carry the reciprocal-transform permission and the approximate-intrinsic envelope, each encoded by an exhaustive match over a non-`#[non_exhaustive]` vocabulary, in `schedule::model::push_numerical` and `kernel::model::push_numerical` alike. This is ADR 0076 item 1 and item 6's shape and the two must land together: a widened realization with an incomplete encoder gives two semantically different regions one identity.
- Carry both forward through `ResourceRequirements`, `region_proposal`, and the artifact envelope's `NumericalFacts`.
- Add both to `REALIZED_DIMENSIONS` and to all three elementary families' capability rows, and delete `ELEMENTARY_UNCARRIED_DIMENSIONS` and its superseded omission test.
- Declare both on the governed target profile, and decide what the measured Apple row honours for each.
- Step whatever identity domains the encoders' record layouts require, and rebaseline every pinned identity on the same tree.

## Non-goals

Widening `MaterializationRounding`, which no admitted operation consumes. Changing the already-live elementary-accuracy assessment path or admitting another semantic family: `request::require_elementary_accuracy` already calls `assess_program_elementary_accuracy`; this ticket carries the generic dimensions rather than creating that assessment.

## Reconsideration trigger

**Fired.** Three elementary families are admitted and the public `RelaxedF32` preset authorizes both withheld dimensions. This is current implementation work, not a deferred trigger.

## Decision packet — 2026-08-09

The trigger establishes need, but the work changes public `NumericalRealization`, target requirements, artifact facts, and several identity domains. Recommendation: add explicit reciprocal-transform permission and approximate-intrinsic envelope fields, using the already-governed typed behaviours rather than backend booleans, and carry them atomically through schedule, kernel, compiler, and artifact records. Tom must accept that exact cross-layer public shape and its identity step before implementation.

## Scope repair — 2026-08-09

`implementation/build` is declared because the required governed Metal profile declaration is owned by `crates/tiler-build`; the prior IR/compiler/artifact scopes could not complete that stated delivery.
