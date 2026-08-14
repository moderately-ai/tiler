---
id: reconcile-input-ordinal-region-local-and-declared-input-semantics
title: Reconcile InputOrdinal region-local and declared-input semantics
status: blocked
priority: p1
dependencies: [decide-the-schedule-local-input-ordinal-model]
related: [decide-the-source-bound-live-row-major-access-surface, admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, implementation/conformance, implementation/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [defect, public-boundary, schedule, identity, shapes]
---
## User-visible outcome

One checked meaning governs `InputOrdinal` and `TensorRole::Input` across schedule construction, kernel lowering, program-stage binding, artifact extent rows, identity, and compiler subject binding. A region-local positional handle is never treated as a program-interface key, and a declared program ordinal is never accepted where a dense region-local position is required.

## Exact-current-base Fact audit — 2026-08-14, `bbbf936ad3d8170ec601cd26eda5235cc2ac1d6b`

- **Fact — the defining type says region-local.** `crates/tiler-ir/src/schedule/handles.rs`, anchor `The ordinal is *region-local and positional*`, requires a region reading `n` inputs to use every ordinal in `0..n` exactly once, explicitly says the handle is not an interface key, and assigns named-input binding to stage accesses.
- **Fact — the containing role says the opposite.** `crates/tiler-ir/src/schedule/model.rs`, anchor `Which declared input tensor this access binds`, says `TensorRole::Input.ordinal` is a declared program-input ordinal, may differ from the access position, and is resolved against the program interface.
- **Fact — intrinsic verification admits sparse declared ordinals.** `crates/tiler-ir/src/schedule/builder.rs`, anchors `The ordinals need not be the dense prefix` and `two distinct ascending program ordinals need not be dense`, deliberately accepts ordinals such as `1` and `7` and calls them program ordinals.
- **Fact — the physical compiler supplies declared ordinals.** `crates/tiler-compiler/src/physical.rs`, anchor `The recognized ordinal, not the first declared input`, copies normalized declared-input ordinals into `TensorRole::Input`; helper tests project `0, 1, 7`, the end-to-end positive admits later input `1`, and forged `0`/`7` roles refuse request binding. Separately, `crates/tiler-ir/src/schedule/builder.rs`, anchors `two distinct ascending program ordinals need not be dense` and `the input ordinal is part of scheduled-region identity`, admits sparse roles intrinsically and pins their schedule-identity effect.
- **False as originally written — program assembly resolves the role directly, while artifact mapping is positional.** `crates/tiler-compiler/src/program.rs`, anchor `AssemblyBinding::Input(ordinal)`, converts `TensorRole::Input.ordinal` directly to a semantic program input index. Separately, `crates/tiler-artifact/src/program/builder.rs`, anchor `maps that tensor through the stage access`, matches the kernel buffer role, zips buffers with stage accesses, then reads `MaterializedOrigin::ProgramInput { key }`. The original Fact omitted the live compiler consumer and therefore understated the defect.
- **Fact — the same public type carries two constructibly distinct coordinates.** `crates/tiler-ir/src/schedule/pointwise.rs` and `pointwise_bf16.rs`, anchors `input_ordinals_are_dense` and `SparseInputOrdinals`, require expression leaf ordinals to be the dense local prefix. `request.rs`, anchor `numbers its leaves by the position of the read`, mints them from read-list position. `builder.rs`, anchor `A read's position and its boundary role are separate facts`, simultaneously allows the access role to name a sparse declared input.
- **Fact — the compiler retains both facts, but assembly does not project their checked association.** Normalized pointwise, epilogue, fold, and contraction subjects carry ordered reads plus declared ordinals. `verify_schedule_with_feasibility` proves the region against that exact request subject before `physical.rs::VerifiedScheduledRegion` stores both values. `CoverAssembly` lacks a typed projection/accessor from the retained subject and therefore reuses the schedule role as the program binding.
- **Inference — the required repair is consequential.** Dense local roles retire currently admitted sparse schedule bytes; declared roles retain program-interface naming in shared schedule identity. The public/identity choice is now isolated in [`decide-the-schedule-local-input-ordinal-model`](decide-the-schedule-local-input-ordinal-model.md), and this implementation must remain blocked until Tom accepts an exact meaning.
- **Inference — the coincidence is not a contract.** Whole-program pointwise currently reads a dense declared prefix, so local and declared ordinals coincide there. Subset, epilogue, fold, and contraction construction already claim populations where they differ. The contradictory meanings therefore cannot be used as the source field of a new public live-extent relation until one authority is selected and every construction/consumer is made coherent.

Reproduce:

```sh
rg -n 'region-local and positional|not an interface key|Which declared input tensor this access binds' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/model.rs
rg -n 'The ordinals need not be the dense prefix|program ordinals need not be dense|The recognized ordinal, not the first declared input|declared ordinal is retained' crates/tiler-ir/src/schedule/builder.rs crates/tiler-compiler/src/physical.rs
rg -n 'region-local input and axis|maps that tensor through the stage access|MaterializedOrigin::ProgramInput' crates/tiler-ir/src/kernel/model.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'AssemblyBinding::Input|input_ordinals_are_dense|SparseInputOrdinals|numbers its leaves by the position of the read' crates/tiler-compiler/src/program.rs crates/tiler-compiler/src/request.rs crates/tiler-ir/src/schedule/pointwise.rs crates/tiler-ir/src/schedule/pointwise_bf16.rs
rg -n 'tensor_role_comment|LiveRowMajor|InputOrdinal|TensorRole::Input' crates/tiler-metal/src crates/tiler-build/src crates/tiler-conformance/src crates/tiler-runtime/src
```

## Decision stop

No production edit is authorized while [`decide-the-schedule-local-input-ordinal-model`](decide-the-schedule-local-input-ordinal-model.md) is open. The current source assigns one public type two incompatible meanings and the compiler consumes the ambiguity at program binding. This ticket resumes only after Tom accepts an exact public meaning and identity consequence.

## Required work

- Re-audit every `InputOrdinal` constructor and every `TensorRole::Input` verifier, encoder, subject binder, kernel/program/artifact consumer, and test. This includes Metal `tensor_role_comment`, docs, and fixtures; build assembly and custom fixtures; conformance construction and role matching; and runtime live-extent fixtures. Name the populations that require sparse declared-input association and those that require dense region-local buffer positions.
- Produce a Pareto-complete public/identity decision if both facts need separate types. Do not silently redefine the existing newtype or use positional coincidence as proof.
- Align the defining docs, containing-role docs, intrinsic validation, physical construction, program-stage binding, artifact mapping, and canonical encoders with the accepted meaning. If two identities survive, give them distinct types and explicit conversion at the program binding boundary.
- Preserve named `InputKey` as the program-interface authority. A schedule handle must reach it only through a checked stage access/materialized origin unless a separately accepted public surface says otherwise.

## Required evidence

- A region reading declared inputs `1` and `7` proves whether its schedule handles are local `[0, 1]` or declared `[1, 7]`, and the program binds both to the intended `InputKey`s without positional guessing.
- Reorder stage accesses while retaining schedule roles; the mismatch fails rather than rebinding an extent or buffer silently.
- Perturb the declared program ordinal independently of the region-local access position and quote the owning diagnostic.
- Prove schedule, kernel, program, artifact, and request identities separate every surviving meaning; show unchanged bytes for populations whose meaning did not change, or perform the governed identity migration.

## Closes when

The public docs and all construction/consumption paths agree on one checked ordinal model, the sparse-subset and dense-prefix controls pass, and [`decide-the-source-bound-live-row-major-access-surface`](decide-the-source-bound-live-row-major-access-surface.md) can select an exact source field type without assigning interface authority to a handle that denies it.
