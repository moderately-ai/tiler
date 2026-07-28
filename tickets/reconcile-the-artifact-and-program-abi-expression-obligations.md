---
id: reconcile-the-artifact-and-program-abi-expression-obligations
title: Reconcile the artifact and program ABI expression obligations
status: closed
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

## Narrowed 2026-07-27 — `adopt_abi` is sound; the earlier failures were wiring

**Measured, isolated from the build path so a wiring fault cannot be mistaken for a layer disagreement.** `probe_whether_a_program_expression_satisfies_the_artifact_obligations` in `crates/tiler-artifact/src/program/tests.rs` adopts a verified program's ABI arena onto a fresh artifact builder and inspects the result:

```text
PROBE grid handle AbiExprId { owner: ArtifactBuilderId(1), index: 0 }
PROBE workgroup handle AbiExprId { owner: ArtifactBuilderId(1), index: 1 }
PROBE program arena 5 nodes
```

The replayed handles are the **artifact builder's own**, minted at its own indices, and the two distinct launch expressions stay distinct. So the replay boundary is sound and `ForeignHandle` in the abandoned attempt was a fault in how that attempt threaded handles through `&self` methods — it adopted in a `&mut self` context and used the map from immutable ones — and not evidence about either layer's obligations.

**What is now known and what is not.** Known: `adopt_abi` produces valid, distinct, owner-correct handles from a program arena, and at least that part of the derive route is not the obstacle. Not known: whether `check_use`'s availability-phase and interface-root obligations accept a program-owned expression, because `check_use` is private and the probe cannot reach it without changing the build path — which is exactly what conflated the two signals last time.

**Next step, and it is now small.** Re-attempt the derive with the adoption performed once in `push_variant` and the resulting handles passed by value into `check_launch` and `check_bindings` — never re-resolved — then classify whatever remains. The `ForeignHandle` class should be gone; anything left is the evidence this ticket wants. If nothing is left, this closes and `bind-the-artifact-variant-abi-to-the-program-abi` reverts to its measured 13-deletion estimate.

The probe test is retained, because it is the thing that distinguishes the two signals and it cost one test to have.

## Closed 2026-07-27 — the premise was wrong three times over, and there is nothing to reconcile

**The failures were `UnusedExpression`. All of them.** Classifying the diagnostic set rather than the error's outer type gives 125 blocks and every one contains exactly `UnusedExpression` — "an expression node is not reachable from any declared use site" (`crates/tiler-artifact/src/program/error.rs:512`).

**That is not an obligation failure, not a layer disagreement, and not a wiring bug.** It is the direct consequence of adopting the program's whole ABI arena while the fixtures still mint their own expressions through `formulas(&mut draft)`. Once a variant stops referencing those, they are unreachable from any use site, and the artifact correctly refuses an arena carrying nodes nothing uses.

**So the artifact layer accepts program-owned expressions fine.** Nothing here contradicts anything. This ticket exists because I read an error's outer type instead of its diagnostics and inferred a contract conflict from it.

**Three successive readings of one run, each wrong:**

1. "The two layers require differently shaped expressions" — inferred from `ArtifactVerificationError` without opening it.
2. "It was a `ForeignHandle` wiring fault" — inferred from error heads that were incidental; a probe then showed the replay boundary is sound, which was true but did not explain the failures either.
3. The actual cause, found only by extracting the `diagnostics` list: the fixtures mint expressions the derived variant no longer uses.

**Closed as obsolete rather than done**, because it asks a question that does not arise. `bind-the-artifact-variant-abi-to-the-program-abi` loses its dependency and reverts to the mechanical change its original estimate described — with one addition now known: the fixtures must also stop minting the expressions they no longer supply, which is deletion in the same helper.

**Retained:** `adopt_abi`, its two tests, and the replay probe, all on `main`. They are correct and either route needs them.
