---
id: carry-the-elementary-numerical-dimensions-in-the-region-realization
title: Carry the elementary numerical dimensions in the region realization
status: todo
priority: p2
dependencies: [carry-every-realized-dimension-through-region-feasibility]
related: [admit-the-silu-activation-family, admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/reference, implementation/metal, contracts/decisions, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, identity, feasibility, decision, needs-tom, public-boundary]
---

# Carry the elementary numerical dimensions in the region realization

## User-visible outcome

A caller that states a numerical contract permitting reciprocal replacement or approximate elementary functions gets an answer about a *target*, rather than a build-level refusal or a silently unasked question, for a program containing `tiler::silu-f32@1`.

## Why this is filed rather than done

**Fact — corrected 2026-08-09.** The obligation is no longer activation-only. `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, and `tiler::softmax-f32@1` are all admitted elementary families; each can consume `NumericalDimension::ReciprocalTransform` and `NumericalDimension::ApproximateIntrinsics`. `ELEMENTARY_UNCARRIED_DIMENSIONS` names the common omission explicitly.

**Fact — listing them would refuse a public preset for every program.** `dimension_requirements` filters by `is_consumable`, so a row entry enters the dimension into the requirement set every contract places on every target; and `unrepresentable_dimension` refuses any consumable dimension that `tiler_ir::schedule::NumericalRealization` cannot carry, which neither is. `session::NumericalContract::RELAXED_F32` authorizes both, so it would become unrepresentable — for programs with no elementary family occurrence as well.

**Fact — the omission is a checked claim rather than a gap.** `policy::ELEMENTARY_UNCARRIED_DIMENSIONS` names both, and `the_uncarried_elementary_dimensions_are_outside_the_realization` fails the moment the region realization grows to carry either, which is the condition under which withholding the rows stops being honest.

**Fact — what holds the obligation meanwhile is backend-local.** `crates/tiler-metal/src/emit.rs` writes `precise::exp`, `precise::rsqrt`, and the `/` operator rather than a fast intrinsic or a reciprocal multiply, and records `MetalNumericalRequirement::PreciseFp32Functions`. That is a guarantee over the operations actually emitted; it is not a profile-level assessment, and it speaks for one backend.

## Required delivery

- Widen `tiler_ir::schedule::NumericalRealization` to carry the reciprocal-transform permission and the approximate-intrinsic envelope, each encoded by an exhaustive match over a non-`#[non_exhaustive]` vocabulary, in `schedule::model::push_numerical` and `kernel::model::push_numerical` alike. This is ADR 0076 item 1 and item 6's shape and the two must land together: a widened realization with an incomplete encoder gives two semantically different regions one identity.
- Carry both forward through `ResourceRequirements`, `region_proposal`, and the artifact envelope's `NumericalFacts`.
- Site both dimensions explicitly at the existing `PolicyLocus::Computation` for each consuming composite occurrence. The subordinate division, exponential, or reciprocal square root is arithmetic inside that semantic occurrence; `OperationNumericalCapability::founded_locus` must state that derivation rather than retaining its current `None` or substituting a fallback in the delivered-record producer.
- Add both to `REALIZED_DIMENSIONS` and to all three elementary families' capability rows, and delete `ELEMENTARY_UNCARRIED_DIMENSIONS` and its superseded omission test.
- Declare both on the governed target profile, and decide what the measured Apple row honours for each.
- Extend `tiler-reference`'s exhaustive realization bridge with typed refusals for a reciprocal permission or non-`Forbidden` approximation it cannot evaluate, and extend `tiler-metal`'s realization-to-emission checks so neither new field can compile while being ignored.
- Step whatever identity domains the encoders' record layouts require, and rebaseline every pinned identity on the same tree.

## Non-goals

Widening `MaterializationRounding`, which no admitted operation consumes. Changing the already-live elementary-accuracy assessment path or admitting another semantic family: `request::require_elementary_accuracy` already calls `assess_program_elementary_accuracy`; this ticket carries the generic dimensions rather than creating that assessment.

## Reconsideration trigger

**Fired.** Three elementary families are admitted and the public `RELAXED_F32` preset authorizes both withheld dimensions. This is current implementation work, not a deferred trigger.

## Decision packet — 2026-08-09

The trigger establishes need, but the work changes public `NumericalRealization`, target requirements, artifact facts, and several identity domains. Recommendation: add explicit reciprocal-transform permission and approximate-intrinsic envelope fields, using the already-governed typed behaviours rather than backend booleans, and carry them atomically through schedule, kernel, compiler, and artifact records. Tom must accept that exact cross-layer public shape and its identity step before implementation.

## Accepted decision — 2026-08-11

Tom accepted the revised narrow first pass in the live decision review, relayed first-hand by the coordinator. `NumericalRealization` gains two direct required fields — `reciprocal_transform: NumericalPermission` and `approximate_intrinsics: ApproximationEnvelope` — rather than an optional/defaulted value, inferred profile-key projection, nested elementary bundle, or dynamically typed dimension map. Every constructor must state both. A missing target declaration remains `Unknown` and refuses; reference and backend consumers must either realize the exact typed value or return a typed refusal. There is no compatibility default and no backend retry.

The two obligations use the existing `PolicyLocus::Computation`. Numerical semantics already says the operation's own arithmetic is the locus for dimensions not assigned to input, result, accumulator, component, or materialization, and the subordinate arithmetic remains inside the composite occurrence. This acceptance therefore adds no new locus variant or locus wire tag. It does require repairing `founded_locus`'s current deliberate `None`; the delivered-realization producer must never silently substitute `Computation` for an unfounded row.

The source audit also found a pre-existing prerequisite: `ResourceRequirements` carries eight numerical dimensions while `physical::region_proposal` asks target feasibility about only input/result subnormals, contraction, and reassociation. [`carry-every-realized-dimension-through-region-feasibility`](carry-every-realized-dimension-through-region-feasibility.md) owns restoring the complete existing eight-dimension projection before this ticket grows it to ten. The carrier also explicitly includes the reference and Metal total consumers that the original delivery packet omitted.

At decision base `b48e7719ccd6e9b4919d40bde998beb122bdb9b0`, the expected coordinated layout steps are scheduled region `tiler.schedule.v5` to `v6`, structured kernel `tiler.kernel.v7` to `v8`, artifact program `tiler.artifact-program.v16` to `v17`, and manifest schema `16.0` to `17.0`. The coherent numerical-contract key domains do not step merely for this work because they already encode both dimensions, and the delivered-realization record already carries the complete eleven-dimension vocabulary. Implementation must rederive those expectations and every pin on its own exact base rather than copying these numbers.

**Strongest counterpoint accepted consciously.** The direct fields break every exhaustive constructor and move multiple identity domains. That blast radius is the safety mechanism: old constructors, encoders, reference bridges, and backend projections must stop compiling until they state the new meaning, whereas an optional field or inferred strict value would preserve source compatibility by silently narrowing the caller's contract.

## Source-audit correction — 2026-08-11

The original closure inventory was incomplete in two correctness-bearing directions. `OperationNumericalCapability::founded_locus` explicitly returns `None` for both dimensions, so adding capability rows alone reaches `numerical-realization-locus-unfounded`; and `physical::region_proposal` currently forwards four of the eight fields `ResourceRequirements` already carries. `ReferenceNumericalConformance::from_realization` and Metal's realization checks are additional total consumers that field access could otherwise leave silently incomplete. The accepted packet above supersedes the narrower inventory without changing the ticket's purpose.

## Scope repair — 2026-08-09

`implementation/build` is declared because the required governed Metal profile declaration is owned by `crates/tiler-build`; the prior IR/compiler/artifact scopes could not complete that stated delivery.
