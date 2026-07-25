---
id: complete-program-identity-with-abi-guards-and-routing
title: Complete program identity with ABI expressions, guards, and routing
status: in-progress
priority: p1
dependencies: [relocate-abi-expressions-into-tiler-ir]
related: [prototype-kernel-program-ir, prototype-artifact-program-model]
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot, contracts/decisions, contracts/navigation, contracts/foundation, contracts/artifacts, contracts/numerics, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity, contracts]
claimed_from: todo
assignee: agent-identity
lease_expires_at: 1785013830
---
`tiler_ir::program::CanonicalKernelProgramIdentity` is **not** the complete
program identity ADR 0072 describes, and the divergence is between an accepted
contract and the ticket that was executed — not an oversight in the code.

The tension, recorded by the implementing agent rather than hidden:

- **ADR 0072** states that complete program identity covers buffers, ABI, guards,
  and routing.
- **ADR 0068** places `AbiExpr` and its evaluation in `tiler_ir::program`, and
  **ADR 0070** lists `AbiExpr` under `program`.
- **`prototype-kernel-program-ir`** scoped ABI, guards, and routing to the
  artifact-facing projection instead, and that is what was built.

So today the identity folds the semantic graph identity, each stage's kernel
identity (which already carries its scheduled-region identity), the proven
disjoint occurrence partition, values, views, allocations, dependencies, and
named outputs — while host expressions, the applicability guard, and routing
remain compiler-owned and outside it. The name reads like the complete article
and is not, which is the part most likely to mislead a later reader.

Nothing is presently unsound: the identity is honest about what it covers, and no
consumer treats it as ADR 0072's complete identity. The risk is that one starts
to — for example a cache keyed on program identity that is blind to a differing
ABI expression or routing decision, which is precisely the "complete cache and
artifact identity" hazard `AGENTS.md` singles out.

**Resolve by moving `AbiExpr`, the applicability guard, and routing into
`tiler_ir::program` per ADR 0068/0070, folding them into the identity, and
bumping the canonical domain tag to `v2`** — the tag is versioned exactly so an
identity-semantics change is explicit rather than silent. Rebaseline the affected
fixtures deliberately and state in the Outcome that it is an intended identity
re-baseline.

If instead the *ADRs* are judged wrong — that ABI, guards, and routing genuinely
belong to the artifact projection — then amend ADR 0072 and ADR 0068 explicitly
rather than leaving an accepted contract describing something the code does not
build. Either outcome is acceptable; a silent standing divergence is not.

Also note for whoever takes this: ADR 0071's `VerifiedProgramPortfolio` remains
unimplemented, and the bounded profile restricts the layer in ways that will need
widening — one materialization per value (which blocks recomputation), a required
direct data edge rather than reachability, `MemorySpace::Device` only, and byte
counts derived from static `Shape` (symbolic extents need `ShapeEnv`).

## Correction 2026-07-25: "routing" names two different things, and this ticket conflates them

Claimed by `agent-api` on base `6fae4f3`, scoped, **not implemented**, and released. No code was written for this ticket; everything below is established by reading, with the check that reproduces it.

The ticket's stated resolution is to move "`AbiExpr`, the applicability guard, and routing into `tiler_ir::program`". Three subjects are named and they do not have the same answer, because two unrelated concepts in this workspace are spelled *routing*.

**Routing as a per-program commit state machine.** `crates/tiler-compiler/src/program.rs` defines `RoutingState` (`Preflight`, `Committed`, `Executing`, `Published`) and `RoutingTransition { from, to, fallback_permitted }`, and `KernelProgram` carries a `Vec<RoutingTransition>` that `verify_kernel_program_layers` checks against a fixed `routing_policy()`. This is the fallback-before-commit contract `AGENTS.md` names, it is a property of one program, and it is compiler-owned and outside program identity today. **It occurs nowhere else in the workspace**; the exact check is `grep -rn "RoutingTransition\|RoutingState" crates/ prototypes/`, which returns only `crates/tiler-compiler/src/program.rs`. This is the routing ADR 0072 must mean, and it can move down.

**Routing as per-portfolio variant priority.** `crates/tiler-artifact/src/program/model.rs` defines `RoutingPolicy` (the bounded profile fixes `StablePriority`), exposes `routing_policy()` and a per-variant `routing_rank()` — "zero-based routing rank; lower is tried first" — and folds the policy into `tiler.artifact-program.v2` identity. This one **cannot** move into a single program's identity, and the reason is not a preference: a rank orders variants against each other, so one `VerifiedKernelProgram` in isolation has no rank to carry. A shape that gave it one would be inventing a value, which is the failure this ticket exists to prevent one layer up.

**Consequence.** The ticket's resolution is right for the ABI, right for the applicability guard, right for the commit state machine, and impossible as written for variant priority. Whoever takes it should split the sentence rather than reconcile it, and should not read `tiler-artifact`'s existing guard-and-routing ownership as evidence that ADR 0072 was already satisfied — the two are different subjects with the same name.

**Fact bearing on the "amend the ADRs instead" branch.** `crates/tiler-artifact/src/program/model.rs` already carries an `applicability_guard()` per variant, an `AbiExprRef` vocabulary, and a `guard_and_routing` `SchemaVersion` versioned independently of the rest of the manifest, all inside `tiler.artifact-program.v2`. So the artifact projection is not an accident that grew where the ADR was not looking; it is a deliberate, separately versioned layer. That strengthens the amendment branch for the *variant-priority* half and does nothing for the other three, which is another reason the three-way split has to come first.

**Blast radius, partially established.** `grep -rn "12866\|12833\|identity_bytes\|as_bytes().len()" crates/tiler-artifact/src/program/tests.rs` returns nothing, so `tiler-artifact` pins no exact artifact-program identity byte length or hex; its identity assertions are relational (two assemblies agree, or two differ), and those survive a re-baseline. That suggests the ripple into `implementation/artifact` is smaller than `bind-stage-coverage-to-index-refinement-identity`'s. **It was not compiled and is not a measurement** — bumping `tiler.kernel-program.v1` to `v2` and folding four new subjects into it was not attempted, so no failing-test count is recorded here and the next worker should take one before assuming the scope set is sufficient.

## Outcome

Implemented, taking branch (a): the ABI, the applicability guard, and the per-program routing-commit lifecycle moved into `tiler_ir::program` and are folded into `CanonicalKernelProgramIdentity`, whose domain tag is now `tiler.kernel-program.v2`. Variant priority stayed in `tiler-artifact`, per the correction above.

### What landed in `tiler_ir::program`

- **The ABI expression arena.** `KernelProgramBuilder` gained `push_abi_root`, `push_abi_unary`, `push_abi_binary`, `push_abi_select` over a new owner-bound `AbiExprId`, interning by `tiler_ir::program::abi::expr_key` so the arena is a function of what the program says rather than of how often a producer rebuilt one formula.
- **The applicability guard.** `KernelProgramBuilder::applicability_guard` admits one Boolean expression whose roots are readable no later than `LiveDevicePreflight`; `VerifiedKernelProgram::applicability_guard` returns its arena position.
- **The entry ABI.** `StageAccess` gained `accessible_bytes`, and `push_stage` gained a `StageLaunch { grid_threads, threads_per_workgroup }`. The two travel with the access and the stage rather than in a parallel list, so a consumer binding a buffer cannot pick up the range of a different one.
- **The routing-commit lifecycle.** `RoutingCommitState` (`Preflight`/`Committed`/`Executing`/`Published`) and `RoutingCommitTransition { from, to, fallback_permitted }`, declared through `push_routing_commit_transition`.

### What is verified, and where

Insertion-time: expression operand and use-site types, root-phase escape, interface-only roots at every use site that must be computable before a device query, an accessible range that its own view's window contradicts, a workgroup width its own bound kernel contradicts, a second guard, a routing step out of lifecycle order, and a routing step permitting fallback at or after commit. Whole-program: a declared guard (`MissingApplicabilityGuard`), an arena node no use site reaches (`UnreferencedAbiExpression`), and a lifecycle carried to publication (`IncompleteRoutingCommitContract`).

The accessible-range and workgroup-width checks evaluate the declared expression against the bound semantic program's *own* declared input extents. That is a compile-time consistency check and not a runtime evaluation; it is why a producer cannot declare a range its own program contradicts, and it is the same argument `tiler_artifact::program::builder::evaluate_static` already made one layer up.

### Decision — the arena is folded transitively, not twice

Program identity encodes the guard, each stage's launch pair, and each access's accessible range **by canonical content key**, and encodes the arena nowhere else. A content key names the node's whole subtree, and `UnreferencedAbiExpression` rejects a node no use site reaches, so the two together prove no retained expression escapes identity. Encoding the arena separately as well would have been redundant bytes in a canonical encoder.

### Decision — `STAGE_KEY_DOMAIN` stays at `v1`; only `PROGRAM_DOMAIN` is bumped

A stage key is the *cross-reference* key that dependency edges, value definitions, and allocation bindings name a stage by, and what it means — the implementation bound plus the occurrences covered — did not change. The launch geometry is folded beside the stage key inside the program encoding instead. This also keeps it byte-aligned with `tiler-artifact`'s independently tagged `tiler.artifact-program.stage.v1`, which folds the same two ingredients; changing one and not the other is exactly the hazard `bind-stage-coverage-to-index-refinement-identity` records.

### Decision — naming, so "routing" does not acquire a second definition

The IR types are `RoutingCommitState`/`RoutingCommitTransition`, not `RoutingState`/`RoutingTransition`. `tiler_artifact::program::RoutingPolicy` and `routing_rank()` are the *portfolio* sense — a rank orders variants against each other — and a program has no rank to carry. Both spellings are now visible in one workspace, so the IR one names the concept it is (commit lifecycle) rather than the word they share. `ArtifactConstructionPlan::routing_guard` was renamed `applicability_guard` for the same reason. A new `ProgramAbiUse` names the four program use sites and its doc states explicitly that it is not `tiler_artifact::program::AbiExprUse`, which additionally covers launch preconditions and deferred predicates.

### What the compiler stopped owning

`crates/tiler-compiler/src/program.rs` lost `HostExpr`-shaped `EntryContract`, `EntryBinding`, `EntryBindingId`, `AbiAccess`, `ComponentRole`, `MaterializedValueId`, `RoutingState`, `RoutingTransition`, `routing_policy()`, `entry_contracts()`, `binding()`, `entry()`, `component_role()`, and `canonical_host_expressions()`. `KernelProgram` is now `{ target_profile_key, core }`. `verify_kernel_program_layers` keeps only what a *compilation* can decide: the request and target binding, the request budgets, the compile-time truth of the guard, and the agreement between each stage's declared launch and the scheduled region it was planned from. Roughly 300 lines of second-copy verification were deleted rather than re-pointed, because two representations of one ABI that nothing keeps in agreement is the drift ADR 0068 exists to prevent.

`session::AbiConstruction`/`AbiEntry` kept their exact public shape and now read the program's own ABI, so `prototypes/serial-sum-compile` needed no signature change.

### Tests replaced rather than deleted

Three compiler tests forged a compiler-side ABI copy — a wrong accessible-byte node, a binding naming the wrong value, a routing step permitting fallback after commit. Those fields no longer exist, so the malformations are unrepresentable rather than merely detected. Their successors are `tiler_ir::program::tests`' `an_accessible_range_the_declared_view_contradicts_is_rejected`, `a_workgroup_width_the_bound_kernel_contradicts_is_rejected`, and `a_routing_commit_step_that_breaks_the_lifecycle_is_rejected_at_insertion`; `crates/tiler-compiler/src/program/tests.rs`'s module header names the substitution so a reader does not read a gap. `host_expression_overflow_is_a_hard_failure` survives, exercising the shared evaluator on a hand-built arena instead of a forged program field.

Eleven new tests in `tiler_ir::program::tests`, of which three are the identity claims that matter: identity moves when only the guard changes, when only the *expression form* of an accessible range changes (`UnsignedLiteral(24)` versus `CheckedMultiply(4, 6)` — same value, different formula), and when only pre-commit fallback permission changes.

### A retraction in `prototypes/serial-sum-compile`

`the_pruned_and_wholesale_arena_replays_agree_because_the_builder_dedupes` asserted `!unreachable.is_empty()` — that the compiler's one canonical nine-node graph, shared by both alternatives, held a node the fused variant never names. That premise is now false: each program owns its arena and `UnreferencedAbiExpression` rejects a node no use site reaches, so the unreachable set is provably empty. The test is renamed `..._because_a_program_names_its_whole_arena` and asserts the stronger fact; `assemble` still prunes, and the case now pins that the prune is a no-op rather than assuming it always will be.

### Measurement

`cargo nextest run --workspace`: 799 tests, all passing (258 → 269 in `tiler-ir`, 207 in `tiler-compiler`). `uv run --locked python scripts/check_repository.py` passes. No golden `.metal` fixture moved, and no artifact-program identity byte length or hex needed rebaselining — the earlier grep-based prediction about that held, and is now a measurement rather than an inference.

### Split out, not hidden

`bind-the-artifact-variant-abi-to-the-program-abi` (p1, new) owns the one thing this ticket deliberately did not do: a `VariantSpec` still declares its own guard, launch, and accessible ranges on the artifact's own arena, and nothing proves those *expressions* are the program's. Both are checked against the same program facts, so under static shapes they cannot diverge in value; under dynamic shapes they can, and that ticket states the two candidate resolutions with the derivation that prefers deriving over checking.

`VerifiedProgramPortfolio` remains unimplemented, as ADR 0071 already records.

### Contracts updated

ADR 0072 (dependency-direction line and the complete-plan consequence), ADR 0070 (a consequence naming the routing-commit lifecycle as the one subject its `program` enumeration did not), `docs/status.md`, `docs/glossary.md` (Variant, Program portfolio, Routing commit), `docs/architecture.md` (complete program identity, the `KernelProgram`/portfolio paragraph, the `tiler-ir` responsibility row), `docs/artifact-abi.md` (ownership boundary and the expression-ownership section), `docs/correctness-and-testing.md`, and `docs/research/program-planning/abi-expression-ownership.md` (`implementation_status: not-started` → `implemented`, with what each of the two steps did).

**Observed and deliberately not fixed:** ADR 0074's "every canonical *byte* identity in the workspace" enumeration omits `CanonicalKernelIdentity` and `CanonicalKernelProgramIdentity`. That omission predates this ticket and the sentence is explicitly dated to commit `b642007`, so adding names minted later would misdate its evidence rather than correct it.
