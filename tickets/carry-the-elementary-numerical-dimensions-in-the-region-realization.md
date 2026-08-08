---
id: carry-the-elementary-numerical-dimensions-in-the-region-realization
title: Carry the elementary numerical dimensions in the region realization
status: todo
priority: p2
dependencies: []
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, feasibility]
---

# Carry the elementary numerical dimensions in the region realization

## User-visible outcome

A caller that states a numerical contract permitting reciprocal replacement or approximate elementary functions gets an answer about a *target*, rather than a build-level refusal or a silently unasked question, for a program containing `tiler::silu-f32@1`.

## Why this is filed rather than done

**Fact — the admission of the activation made two dimensions consumable, and the capability table withholds both.** `tiler::silu-f32@1` contains a division and an elementary function, so its observable result differs between the two resolutions of `NumericalDimension::ReciprocalTransform` and between the two of `NumericalDimension::ApproximateIntrinsics`. Under `crates/tiler-compiler/src/policy.rs`'s own stated rule — "'consume' means the operation's observable result can differ between two resolutions of that dimension", read conservatively — both belong in the activation's capability row.

**Fact — listing them would refuse a public preset for every program.** `dimension_requirements` filters by `is_consumable`, so a row entry enters the dimension into the requirement set every contract places on every target; and `unrepresentable_dimension` refuses any consumable dimension that `tiler_ir::schedule::NumericalRealization` cannot carry, which neither is. `session::NumericalContract::RelaxedF32` authorizes both, so it would become unrepresentable — for programs with no activation in them as well.

**Fact — the omission is a checked claim rather than a gap.** `policy::SILU_UNCARRIED_DIMENSIONS` names both, and `the_uncarried_elementary_dimensions_are_outside_the_realization` fails the moment the region realization grows to carry either, which is the condition under which withholding the row stops being honest.

**Fact — what holds the obligation meanwhile is backend-local.** `crates/tiler-metal/src/emit.rs` writes `precise::exp` and the `/` operator rather than a fast intrinsic or a reciprocal multiply, and records `MetalNumericalRequirement::PreciseFp32Functions`. That is a guarantee over the operations actually emitted; it is not a profile-level assessment, and it speaks for one backend.

## Required delivery

- Widen `tiler_ir::schedule::NumericalRealization` to carry the reciprocal-transform permission and the approximate-intrinsic envelope, each encoded by an exhaustive match over a non-`#[non_exhaustive]` vocabulary, in `schedule::model::push_numerical` and `kernel::model::push_numerical` alike. This is ADR 0076 item 1 and item 6's shape and the two must land together: a widened realization with an incomplete encoder gives two semantically different regions one identity.
- Carry both forward through `ResourceRequirements`, `region_proposal`, and the artifact envelope's `NumericalFacts`.
- Add both to `REALIZED_DIMENSIONS` and to `tiler::silu-f32@1`'s capability row, and delete `SILU_UNCARRIED_DIMENSIONS` and its test.
- Declare both on the governed target profile, and decide what the measured Apple row honours for each.
- Step whatever identity domains the encoders' record layouts require, and rebaseline every pinned identity on the same tree.

## Non-goals

Widening `MaterializationRounding`, which no admitted operation consumes. Reaching the accuracy assessment from the compile path, which needs the whole-program recognizer that `reach-a-verified-kernel-through-the-structural-families` owns.

## Reconsideration trigger

Active as soon as a caller needs `RelaxedF32` for a program containing an activation, and unconditionally once a second elementary family is admitted — `admit-the-rms-normalization-family` and `admit-the-softmax-family` both reach the same two dimensions.
