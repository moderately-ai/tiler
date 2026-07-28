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

## Correction 2026-07-27 — the inference above is weaker than it was stated

**The "two layers require differently shaped expressions" conclusion is not established by the run it was drawn from, and it should not be carried forward as fact.**

Re-reading the captured failure output classifies the error heads as `ExpressionType`, `RootPhaseEscape`, `NonInterfaceRoot`, and **`ForeignHandle`**. That last one is the problem: `ForeignHandle` means a handle was resolved against a builder that does not own it. That is a *wiring* fault in the attempted change — the `AbiExprId`s returned by `adopt_abi` being used somewhere they had not been re-resolved — not evidence that the artifact's obligations reject a program-owned expression.

If `ForeignHandle` accounts for the bulk of the 266, the whole inference collapses: the failures would say the attempt was wired wrongly, not that the layers disagree. The 126 `ArtifactVerificationError`s cannot be classified from the captured output at all, because that error's `Debug` dumps the entire builder and the cause is past the truncation.

**So the first deliverable of this ticket is unchanged but its premise is now open:** re-run the derive change, capture the *tail* of each failure rather than its head, and separate

- failures caused by the attempt's own handle plumbing, which are bugs to fix and prove nothing, from
- failures where a correctly-plumbed program expression is genuinely refused by `check_use`'s phase or interface-root obligation, which are the evidence this ticket needs.

Only the second class supports the question below. **It is possible there is no second class**, in which case this ticket closes as "the layers agree and the binding is mechanical after all", and `bind-the-artifact-variant-abi-to-the-program-abi` reverts to its original estimate.

**Why this correction is here rather than quietly fixed.** The inference was recorded confidently on two tickets and used to justify a dependency edge and a split. A reader who took it at face value would design a contract reconciliation for a problem that may be a bug in one afternoon's branch.
