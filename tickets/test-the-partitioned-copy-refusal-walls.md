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

## Fact — every Fact above re-verified at `97178f3697381d42bf3a14eae629299e4a81bfd7`

Re-read at the dispatched base rather than inherited. `grep -rl` over `crates/` returns 4 files for `UnloweredRegionProgram` (`kernel/error.rs`, `kernel/lower.rs`, `kernel/model.rs`, `kernel/verify.rs`), 1 for `unlowered-region-program` (`kernel/error.rs`), 2 for `BitPreservingCopyResources` (`program/error.rs`, `program/model.rs`), and 1 for `bit-preserving-copy-resources` (`program/error.rs`); none is a test file, and none of the four appears anywhere under `docs/` or `spikes/`. Each anchor resolves once in the file its citation names: `RegionProgram::PartitionedCopy(program) =>` in `schedule/builder/intrinsic.rs`, `let RegionProgram::Numerical { scalar, numerical } = &schedule.index.program else` in `kernel/lower.rs`, and `The copy region is refused before any buffer is compared` in `kernel/verify.rs`.

The dead/reachable split holds, and `push_requirements` is dead behind **two** guards rather than the one recorded. Kernel identity is encoded from exactly one site — `encode_identity` in `builder.rs`, reached only after `verify_kernel` returned `Ok` — so a copy region cannot reach it. An arithmetic region cannot smuggle the copy arm in through its declared requirements either: `verify_kernel` proves `data.requirements != derived` and refuses as `resource-requirements`, and an arithmetic region derives `FloatingPoint`.

## Required delivery

Drive a verified copy region into `plan` and into `verify_signature` and assert the stable rule identifier `unlowered-region-program` one-for-one, in the idiom the schedule builder's eleven `partitioned-copy-*` rule tests already use. Do the same for `bit-preserving-copy-resources` by handing `push_resources` a `ResourceRequirements` carrying the `BitPreservingCopy` arm, on both the identity path and the wire path where the diagnostic is wrapped as `ArtifactCodecError::ModelObligation`.

Perturb the subject, not the assertion: remove or widen the guard and quote the failure text. Where a site is unreachable, say so and state what would have to change for it to become reachable, rather than adding a test that cannot fail.

## Closes when

Each reachable refusal is reached by a test that fails when its guard is removed, with the quoted failure text recorded; and each unreachable site carries a written reachability argument naming the guard ahead of it.

## Delivered

Six tests, none of which existed before, in the idiom the schedule builder's `partitioned-copy-*` rule tests use.

`crates/tiler-ir/src/kernel/tests/copy_refusal.rs` builds a verified arity-2 partitioned copy and drives it into both reachable kernel refusals, asserting `unlowered-region-program` one for one: `lower_scheduled_region` through `plan`, and `KernelBuilder::build` through `verify_signature` at a signature width `buffer-contract` accepts, so the diagnostic is the region-program refusal and not a width defect standing in for it. Its module documentation carries the reachability argument for the two dead sites.

`crates/tiler-artifact/src/program/codec/tests/copy_resources.rs` hands `push_resources` a `ResourceRequirements` carrying `BitPreservingCopy` three ways: directly, through the identity path (`ArtifactEnvelope::canonical_identity`, and `encode`, which derives the identity first and so reports `IdentityDerivation`), and through the wire path (`encode_with_identity`), where the diagnostic arrives wrapped as `ArtifactCodecError::ModelObligation`.

### Perturbation evidence

Each guard was removed or widened and the failure text recorded; the assertions were never touched.

- Deleting any of the three `let ... else` refusals is a compile error, not a silent widening: `error[E0005]: refutable pattern in local binding`, naming `&RegionProgram::PartitionedCopy(_)` at `lower.rs` and `verify.rs` and `RegionNumericalRequirements::BitPreservingCopy` at the artifact's `model.rs`.
- `plan`'s refusal renamed to `BodyRefinement`: `assertion left == right failed: Verification(BodyRefinement) / left: "body-refinement" / right: "unlowered-region-program"`. Only the lowering test reddened, so the two kernel refusals are independently reached.
- `verify_signature` widened to admit the copy arm (`PartitionedCopy(_) => return Ok(())`): `assertion left == right failed: [OutputCoverage] / left: "output-coverage" / right: "unlowered-region-program"`. Only the verification test reddened.
- `push_resources` widened to `return Ok(())` on the copy arm: all three artifact tests reddened — `the copy arm has no grammar: ()`, `the copy arm has no identity grammar: CanonicalArtifactProgramIdentity([...])`, and `the copy arm has no wire grammar: [84, 73, 76, 69, 82, 65, 82, 84, ...]`. The last is the failure the wall exists to prevent: a copy-carrying envelope encoding to complete bytes under an arithmetic artifact's identity.

No identity value moves. The change is six tests and two `mod` lines; no encoder, grammar, or pinned identity was touched, and the pinned-identity tests pass unchanged.
