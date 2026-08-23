---
id: test-the-partitioned-copy-refusal-walls
title: Test the partitioned-copy refusal walls
status: in-progress
priority: p2
dependencies: []
related: [admit-an-explicit-non-arithmetic-region-and-delivery-state, admit-the-partitioned-copy-scheduled-region, lower-the-partitioned-copy-region-through-kernel-ir]
scopes: [implementation/ir, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, testing, verification, strict]
claimed_from: todo
assignee: worker-copywalls
lease_expires_at: 1787488807
---
## Outcome

Every typed refusal that currently keeps a verified partitioned-copy region out of kernel IR and out of an artifact is watched failing, so none of them can be deleted or bypassed silently.

## Fact — the population is untested, at `41a018fbc7e33e9a573a63a61264f49e5f41717a`

Grepped workspace-wide at this base, `UnloweredRegionProgram`, `unlowered-region-program`, `BitPreservingCopyResources`, and `bit-preserving-copy-resources` appear in **no test file at all**. The two existing tests that call `push_resources` directly — in `crates/tiler-artifact/src/program/codec/tests/subgroup.rs` and `crates/tiler-artifact/src/program/tests/identity_encoders.rs` — both `.expect(...)` the arithmetic arm and therefore never reach the refusal.

Two of the four raise sites are genuinely reachable, because a verified copy region is constructible: `crates/tiler-ir/src/schedule/builder/intrinsic.rs` dispatches one (anchor `RegionProgram::PartitionedCopy(program) =>`), and both `plan` in `crates/tiler-ir/src/kernel/lower.rs` (anchor `let RegionProgram::Numerical { scalar, numerical } = &schedule.index.program else`) and `verify_signature` in `crates/tiler-ir/src/kernel/verify.rs` (anchor `The copy region is refused before any buffer is compared`) refuse it. The other two — the `addressing` arm in `lower.rs` and `push_requirements` in `crates/tiler-ir/src/kernel/model.rs` — are dead behind those, by their own recorded arguments.

## Required delivery

Drive a verified copy region into `plan` and into `verify_signature` and assert the stable rule identifier `unlowered-region-program` one-for-one, in the idiom the schedule builder's eleven `partitioned-copy-*` rule tests already use. Do the same for `bit-preserving-copy-resources` by handing `push_resources` a `ResourceRequirements` carrying the `BitPreservingCopy` arm, on both the identity path and the wire path where the diagnostic is wrapped as `ArtifactCodecError::ModelObligation`.

Perturb the subject, not the assertion: remove or widen the guard and quote the failure text. Where a site is unreachable, say so and state what would have to change for it to become reachable, rather than adding a test that cannot fail.

## Closes when

Each reachable refusal is reached by a test that fails when its guard is removed, with the quoted failure text recorded; and each unreachable site carries a written reachability argument naming the guard ahead of it.
