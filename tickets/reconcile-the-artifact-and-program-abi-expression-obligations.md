---
id: reconcile-the-artifact-and-program-abi-expression-obligations
title: Reconcile the artifact and program ABI expression obligations
status: todo
priority: p1
dependencies: []
related: [bind-the-artifact-variant-abi-to-the-program-abi]
scopes: [implementation/artifact, implementation/ir]
shared_scopes: []
paths: []
tags: [artifact, abi, correctness]
---
Split from `bind-the-artifact-variant-abi-to-the-program-abi`, which cannot proceed until this is answered. Read that ticket's two 2026-07-27 measurements first; this exists because the second one refuted the assumption the first was built on.

## Fact — measured, not inferred

Deriving a variant's applicability guard, launch geometry, and per-binding accessible ranges from the bound `VerifiedKernelProgram` — replacing the caller's restatement with the program's own expressions, replayed through `ArtifactProgramBuilder::adopt_abi` — **compiles with 13 deletions and then fails 266 tests at verification**: 126 distinct `ArtifactVerificationError`s, plus `ExpressionType` and `NonInterfaceRoot`.

The build succeeds and the artifact is refused. The exact reproduction is to remove those three fields, resolve each use site through `adopt_abi`'s position map, and run `cargo nextest run -p tiler-artifact`.

## Inference — the two ABIs are not one formula differently spelled

If they were, substituting the program's expression would verify, because the artifact's obligations would already hold of it. They do not hold. The program's launch and accessible-range expressions fail the obligations `ArtifactProgramBuilder::check_use` imposes — its availability-phase and interface-root requirements — and the static-evaluation contract behind `ExpressionType`.

So the divergence `bind-the-artifact-variant-abi-to-the-program-abi` describes is **deeper than restatement**. The two layers require differently *shaped* expressions for the same quantity, and each is internally consistent: `tiler-ir`'s `check_stage_accesses` and `check_stage_launch` admit what the program declares, and the artifact's checks admit what a variant declares, and neither was written against the other.

## The question

Which side is wrong, and in what respect?

- **The artifact's requirements may be too strict** for an expression that has already been verified in `tiler-ir` — in which case a program-derived expression should be admitted under a weaker obligation than a caller-supplied one, and the distinction has to be represented rather than assumed.
- **The program's expressions may be under-constrained** relative to what a runtime must evaluate — in which case `tiler-ir` gains an obligation and the artifact's requirements stand.
- **Both may be right for their own layer**, in which case binding needs an explicit translation with its own stated contract, and the ticket that binds them owns writing it.

Do not settle this by relaxing whichever check is in the way. Each of the three answers implies a different thing about what a verified program *promises* about its ABI, and that promise is what a runtime evaluates.

## Closes when

The failing obligations are enumerated by class with a named example of each; it is decided and recorded which layer's contract changes and why; the decision is expressed in the two builders rather than in prose alone; and `make full` passes. `bind-the-artifact-variant-abi-to-the-program-abi` then becomes the mechanical change it was originally estimated to be.
