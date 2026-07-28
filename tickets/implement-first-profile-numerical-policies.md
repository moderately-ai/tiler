---
id: implement-first-profile-numerical-policies
title: Implement first-profile numerical policy presets
status: in-progress
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [repair-numerical-witness-integrity]
scopes: [implementation/ir, implementation/reference, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, mature-product]
---
Implement typed strict/relaxed numerical dimensions and per-operation/per-dtype conformance for reassociation, reciprocal transforms, approximations, exceptional values, signed zero, contraction, materialization rounding, and reduction order. Preserve compound/quantized seams and fail closed where evidence is Unknown.

## Work in flight — recorded 2026-07-28

Recorded as **Fact** about a working tree, not as delivered work. None of it is on `main`.

- **Branch and worktree.** `tkt/implement-first-profile-numerical-policies`, checked out at `/Users/tsanterre/workspace/github.com/moderately-ai/tiler/.claude/worktrees/agent-ad2893b1fba4d7f5b` (registered — `git worktree list` names it).
- **Base.** The branch HEAD is `06af0c6` ("Claim four conflict-free tickets into the scopes just freed"), which is **319 commits behind `main` at `01264be`** (`git rev-list --count tkt/implement-first-profile-numerical-policies..main` → `319`). Nothing on the branch is a commit of this work; `06af0c6` is a ticket-claim commit.
- **Everything is uncommitted.** The changes are *staged in the index* and no commit holds them, so "uncommitted" is the accurate word and "untracked" is not. `git status --porcelain` in that worktree reports new files `crates/tiler-compiler/src/policy.rs` (722 lines) and `crates/tiler-reference/src/conformance.rs` (367 lines), plus modifications to `crates/tiler-compiler/src/{explain,feasibility,frontier,honourability,lib,physical,pipeline,request,selection,session}.rs`, `crates/tiler-ir/src/schedule/{mod,numerics}.rs`, and `crates/tiler-reference/src/{lib,oracle}.rs`.
- **Three new follow-up tickets, not two.** The same working tree adds `tickets/record-the-arithmetic-type-in-the-numerical-honourability-contract.md`, `tickets/re-record-the-stale-const-eval-trybuild-golden.md`, and `tickets/widen-the-region-realization-to-consumable-dimensions.md`, and modifies `tickets/expose-the-numerical-contract-preference-list.md`. All four are uncommitted alongside the code.
- **Status note.** The remaining decision is whether to land the rebase or return this ticket to `todo` and abandon the worktree; a 319-commit gap makes the rebase the expensive half of the work rather than a formality. This pass did not change `status`, which stays `in-progress` — a frontmatter change is outside its scope and is recorded here instead.
- **Nothing landed, provably.** `main` still has four `NumericalDimension` variants (`crates/tiler-compiler/src/honourability.rs:58-67`, `CANONICAL_DIMENSIONS: [NumericalDimension; 4]` at `:71`) against the worktree's eleven (`CANONICAL_DIMENSIONS: [NumericalDimension; 11]` at `honourability.rs:150` there). That is the one-line check that separates "in flight" from "delivered".

### The worktree's Outcome section, copied verbatim

**Claimed, not verified against main** — written against a base 319 commits old and not re-gated. Every statement below is the in-flight author's; none has been checked against current `main`, and the line and symbol references are to the worktree, not to this checkout.

> #### The dimensions
>
> `crate::honourability::NumericalDimension` went from four to eleven, and every one is a term `docs/numerical-semantics.md` already defines: the two subnormal dimensions, contraction, reassociation, **permutation** (the order contract's second, independent dimension — the "reduction order" this ticket names), **signed zero**, **reciprocal transform**, **approximate intrinsics**, **NaN assumptions**, **infinity assumptions**, and **materialization rounding**. Nothing was invented.
>
> **Distributivity is deliberately absent.** That contract records it as a third numerical dimension and then states that no distributivity permission is admitted and that whether to admit one is reserved to the decision admitting a tensor-contraction family. Adding it would have converted a reserved question into an implemented permission.
>
> Each dimension carries the behaviour space its own normative text requires rather than a uniform permission, and the approximate-intrinsic dimension is the case that forces it: the contract says it "resolves to a maximum accuracy envelope … **not a boolean**", so `ApproximationEnvelope` is a governed named-envelope vocabulary and `Permitted` would have stated no bound at all. Likewise the two exceptional-value dimensions resolve to `ExceptionalValueAssumption::AssumeAbsent { provenance }` over the three provenance classes the contract requires of every value-domain fact, because an assumption with no provenance is indistinguishable from a proven one and the difference is exactly what decides whether a rewrite may consume it.
>
> #### The presets, and what each is a claim about
>
> `crate::policy::NumericalPolicyPreset` registers three, each resolving to a complete contract with its own versioned key, reached publicly through `session::NumericalContract`:
>
> - **`Strict` → `tiler.strict-f32.v1`.** The claim: results are the strict IEEE-754 reading under round-to-nearest ties-to-even, with gradual underflow, every arithmetic NaN canonicalized, and no reordering, fusion, or substitution.
> - **`FlushSubnormalsToZero` → `tiler.flush-f32.v1`.** The claim: flushing subnormals to the zero of their own sign is part of what the program means. It overrides exactly two fields of the strict contract, visibly.
> - **`Relaxed` → `tiler.relaxed-f32.v1`.** The claim: results may differ from the strict reading by contraction, by regrouping a reduction's contributor sequence, by reciprocal replacement of division, and by an approximate elementary function within `tiler::backend-elementary@1`.
>
> A preset is a claim about **what the caller requests**, never about what a target can do. Naming a laxer preset does not make a strict program compile somewhere it could not; it states a different program, with a different identity and a different artifact, which feasibility then assesses on its own terms.
>
> #### Per-dtype conformance, and the shape change it forced
>
> Every honourability key is now `(dimension, arithmetic type)` — `DeclaredBehaviour`, `NumericalRequirement`, `RelaxationRequirement`, and all four resolution outcomes carry a `tiler_ir::schedule::ArithmeticType`, and `CheckedTargetProfile::resolve_dimension` matches on it rather than filtering after the fact. The forcing measurement is on ADR 0076 boundary item 3: one Apple profile flushes subnormals in `f32` and preserves them in `f16`, so `InputSubnormals` is `SupportedExactly` for one and `Unsupported` for the other on one profile, and a dimension-only key would have to state one of them wrongly. `honoured_alternative` matches the arithmetic type for the same reason: reporting a behaviour honoured in a neighbouring dtype would tell a caller a contract is available that this dimension does not offer for the type it asked about.
>
> `ArithmeticType` is not a second dtype identity system. `canonical_type_key` returns the same namespaced versioned spelling a `tiler_ir::semantic::TypeKey` renders, and two tests pin it — one against a constructed key, one against the resolved type the standard registry actually admits — because the `const`-usable string and the owned key are a representation difference that must never become an identity difference.
>
> Each registered contract resolves exactly one arithmetic type and says which. A contract stating a type the profile is silent about resolves to `Unknown` and fails closed; `a_contract_for_an_undeclared_arithmetic_type_is_unknown` drives it.
>
> #### Per-operation conformance
>
> `crate::policy::operation_capabilities` states, for each admitted semantic operation, which dimensions it can consume, and `OperationNumericalCapability::effective` resolves the program ceiling against it — the intersection `docs/numerical-semantics.md` defines. It returns `None` rather than a strict behaviour for a dimension the operation cannot consume, because "no resolution" and "resolved strictly" are different claims and collapsing them would let a later rewrite read a manufactured strictness as an obligation. The table is deliberately conservative: an entry present but unexercised costs a target one declaration, while a missing entry drops a requirement and lets a target be admitted without being asked.
>
> That table is what makes the requirement set derived rather than asserted. A profile is asked about exactly the dimensions some admitted operation can consume — eight of the eleven — so a target is not rejected over a freedom nothing in the program exercises.
>
> #### How an unhonourable request is rejected
>
> Unchanged in shape and widened in content. Resolution happens once, per target, before any planning, in `resolve_numerical_contract`; a stated contract no target honours yields `RequestError::NoResolvableNumericalContract` carrying one canonical-first `ContractRejection` per stated entry in the caller's order, and each names the contract key, the dimension, **the arithmetic type**, the required behaviour, the means the profile declares, the behaviour it does honour if any, and the declaring profile's identity. `Unhonourable`, `Undeclared`, and `Deferred` stay three separate claims. No cost participates and nothing proposes a substitute.
>
> A second, distinct refusal is new. `RequestError::UnrepresentableNumericalDimension` fires **before admission and before any target is consulted**, for a dimension an admitted operation can consume that `NumericalRealization` cannot carry. It is a statement about this build, not about a profile: reporting it as unhonourable would attribute a build limitation to a declaration that never spoke about the dimension. Four such dimensions exist today — permutation, signed zero, and both exceptional-value assumptions — and each is driven individually. `widen-the-region-realization-to-consumable-dimensions` owns removing the limitation; it needs `tiler-metal` and `tiler-artifact`, both outside this ticket's scopes.
>
> #### The reference oracle can now be told a contract
>
> `tiler-reference` reached no numerical contract at all and computed in whatever host `f32` arithmetic does, which preserves subnormals — the dangerous direction against a device that flushes, since the oracle is the side that would be called wrong. `ReferenceNumericalConformance` binds the two subnormal dimensions into both evaluators and both request types, and `from_realization` **refuses** a realization permitting contraction or reassociation rather than accepting it and evaluating the strict reading, because a permissive contract's result is a set and a single-valued oracle would assert a bitwise equality the contract does not promise. `a_stated_flushing_contract_changes_what_the_reference_computes` drives the two dimensions independently over the three multiplies that isolate them, mirroring the measured Apple isolation.
>
> #### Compound and quantized seams
>
> Preserved by absence rather than by a placeholder. `ArithmeticType` names scalar float formats; a compound or quantized value is a scheme-typed `ResolvedValueType::encoded_numeric` whose conversion behaviour is its own typed contract, and `operation_capabilities` enumerates only the scalar `f32` operations this build admits, so an operation outside that table has no capability entry and therefore no effective permission to compute.
>
> #### Repairs found on the way
>
> `crate::physical::fused_region` hard-coded `contraction: false` and `permits_reassociation: false` while its unfused siblings derived both from the contract. Invisible while every registered contract forbade both; under the relaxed preset it would have failed the schedule verifier's realization cross-check and dropped the fused candidate — fail-closed, but for a reason no diagnostic would have named. Both now derive from the contract.
>
> #### Deliberately not done
>
> - `NumericalRealization` still carries four dimensions and `new` keeps its signature, which is what kept `tiler-metal` and `tiler-metal-aot` out of this change. `widen-the-region-realization-to-consumable-dimensions` owns it.
> - `docs/numerical-semantics.md`'s honourability section still describes the declaration as per-dimension with no arithmetic type. That contract is outside this ticket's scopes; `record-the-arithmetic-type-in-the-numerical-honourability-contract` owns it.
> - The public preset spelling stays `session::NumericalContract`. The exact surface added is appended to [`expose-the-numerical-contract-preference-list`](expose-the-numerical-contract-preference-list.md), which owns the public numerical boundary.

## Closes when

- Eleven numerical dimensions resolve per `(dimension, arithmetic type)`, so one profile can flush in one arithmetic type and preserve in another without either being stated wrongly.
- Three presets are registered and each carries its own versioned contract key.
- An unhonourable request rejects with `RequestError::NoResolvableNumericalContract`, naming the dimension, the arithmetic type, the required behaviour, the means the profile declares, and the declaring profile's versioned identity.
- The reference oracle refuses a realization permitting contraction or reassociation rather than evaluating the strict reading against it.
- The compound and quantized seams are preserved by the absence of a placeholder, not by one.
- The work is on `main` as commits, rebased onto a current base, and `make full` passes there.
