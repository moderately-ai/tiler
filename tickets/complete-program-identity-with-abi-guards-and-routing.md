---
id: complete-program-identity-with-abi-guards-and-routing
title: Complete program identity with ABI expressions, guards, and routing
status: in-progress
priority: p1
dependencies: [relocate-abi-expressions-into-tiler-ir]
related: [prototype-kernel-program-ir, prototype-artifact-program-model]
scopes: [implementation/ir, implementation/compiler]
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
