---
id: reconcile-input-ordinal-region-local-and-declared-input-semantics
title: Reconcile InputOrdinal region-local and declared-input semantics
status: todo
priority: p1
dependencies: []
related: [decide-the-source-bound-live-row-major-access-surface, admit-symbolic-extents-through-schedule-formation, associate-live-extent-operands-with-symbolic-semantic-interface-axes]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal, implementation/build, implementation/conformance, implementation/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [defect, public-boundary, schedule, identity, shapes]
---
## User-visible outcome

One checked meaning governs `InputOrdinal` and `TensorRole::Input` across schedule construction, kernel lowering, program-stage binding, artifact extent rows, identity, and compiler subject binding. A region-local positional handle is never treated as a program-interface key, and a declared program ordinal is never accepted where a dense region-local position is required.

## Exact-base Facts — 2026-08-14, `a660ed618446ade55234993b835e75e26d44921c`

- **Fact — the defining type says region-local.** `crates/tiler-ir/src/schedule/handles.rs`, anchor `The ordinal is *region-local and positional*`, requires a region reading `n` inputs to use every ordinal in `0..n` exactly once, explicitly says the handle is not an interface key, and assigns named-input binding to stage accesses.
- **Fact — the containing role says the opposite.** `crates/tiler-ir/src/schedule/model.rs`, anchor `Which declared input tensor this access binds`, says `TensorRole::Input.ordinal` is a declared program-input ordinal, may differ from the access position, and is resolved against the program interface.
- **Fact — intrinsic verification admits sparse declared ordinals.** `crates/tiler-ir/src/schedule/builder.rs`, anchors `The ordinals need not be the dense prefix` and `two distinct ascending program ordinals need not be dense`, deliberately accepts ordinals such as `1` and `7` and calls them program ordinals.
- **Fact — the physical compiler supplies declared ordinals.** `crates/tiler-compiler/src/physical.rs`, anchors `The recognized ordinal, not the first declared input` and `the input ordinal is part of scheduled-region identity`, copies normalized declared-input ordinals into `TensorRole::Input`; tests require nonzero and sparse values to survive.
- **Fact — the actual program/artifact mapping is positional through stage accesses.** `crates/tiler-ir/src/kernel/model.rs`, anchor `region-local input and axis`, describes `InputExtentParameter` as region-local. `crates/tiler-artifact/src/program/builder.rs`, anchor `maps that tensor through the stage access`, matches the parameter to the kernel buffer role, zips buffers with stage accesses, then reads `MaterializedOrigin::ProgramInput { key }`. It does not resolve `InputOrdinal` directly against the interface.
- **Inference — the coincidence is not a contract.** Whole-program pointwise currently reads a dense declared prefix, so local and declared ordinals coincide there. Subset, epilogue, fold, and contraction construction already claim populations where they differ. The contradictory meanings therefore cannot be used as the source field of a new public live-extent relation until one authority is selected and every construction/consumer is made coherent.

Reproduce:

```sh
rg -n 'region-local and positional|not an interface key|Which declared input tensor this access binds' crates/tiler-ir/src/schedule/handles.rs crates/tiler-ir/src/schedule/model.rs
rg -n 'The ordinals need not be the dense prefix|program ordinals need not be dense|The recognized ordinal, not the first declared input|declared ordinal is retained' crates/tiler-ir/src/schedule/builder.rs crates/tiler-compiler/src/physical.rs
rg -n 'region-local input and axis|maps that tensor through the stage access|MaterializedOrigin::ProgramInput' crates/tiler-ir/src/kernel/model.rs crates/tiler-artifact/src/program/builder.rs
rg -n 'tensor_role_comment|LiveRowMajor|InputOrdinal|TensorRole::Input' crates/tiler-metal/src crates/tiler-build/src crates/tiler-conformance/src crates/tiler-runtime/src
```

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
