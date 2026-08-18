---
id: revisit-kernel-body-single-spelling-gate
title: Revisit the single-spelling kernel body refinement gate when the profile widens
status: deferred
priority: p2
dependencies: []
related: [prototype-structured-kir-slice, own-operation-family-support-matrix]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, verification]
---
The kernel verifier's final check is **derive-and-compare**: after the specific
rules run (so diagnostics stay precise), it re-derives the canonical body from
the scheduled region and requires structural equality. A kernel that is
semantically equivalent but *differently spelled* is therefore rejected as
`BodyRefinement`. This is deliberate and fail-closed — it never accepts a body on
an unproven equivalence — and the alternative, a symbolic index normalizer, was
designed and rejected as materially more machinery for the same guarantee.

The consequence, which is fine today and will not stay fine: the bounded profile
admits **exactly one spelling** of a legal kernel. That holds because the profile
has one canonical form per scheduled region. It stops holding as soon as the
operation and schedule surface widens enough that two genuinely different
spellings are both legal for one region — at which point derive-and-compare
starts rejecting *valid* kernels, and an external producer cannot supply its own
legal body at all.

**Trigger for reconsideration:** the first time a widened profile admits more
than one legal body for a single scheduled region, or the first time an external
producer needs its own spelling accepted. Both are foreseeable consequences of
the operation-family breadth work, so this ticket is related to that owner.

When that happens, the replacement must still prove equivalence rather than
assume it — either a normalizer with its own correctness argument, or a checked
equivalence relation with stated soundness. **Do not weaken this gate before the
trigger**, and specifically do not replace structural equality with a
looser structural heuristic that admits more bodies without proving they mean the
same thing; a fail-closed rejection of a valid kernel is recoverable, an accepted
wrong kernel is not.

## Outcome

**The trigger has not fired. The gate is unchanged, deliberately, and the ticket stays deferred — but the trigger is now detectable instead of depending on someone remembering this ticket exists.**

This ticket's charter is a conditional: revisit *when* the profile widens, and "do not weaken this gate before the trigger". The work was therefore to check the condition, not to change the verifier. No behaviour changed.

### Trigger check (inspected source, base `f286289`)

**Neither trigger condition holds.**

*Condition (a) — more than one legal body for one scheduled region.* `crates/tiler-ir/src/kernel/verify.rs::verify_kernel` calls `super::lower::derive_canonical(schedule, schedule_identity, derived)` and rejects on inequality. `derive_canonical` is a deterministic function of the `ScheduledRegion`, so the profile admits one body per region by construction. What decides whether that is still *correct* is whether the vocabularies shaping a body leave a producer any legal degree of freedom. They do not:

- `ExecutionBinding` — one variant, `GlobalLinearInvocation`.
- `TailPolicy` — one variant, `Exact`.
- `LogicalAccess` — `LinearIdentity`, `ReductionContributor`.
- `ReductionTopology` — `None`, `Serial`.
- `ScalarProgram` — `MultiplyThenAdd`, `StrictSerialSum`, `FusedMultiplyAddSerialSum`.

A single execution binding with no tail is the substance of it: there is one way to map invocations to coordinates and no remainder to handle, so a region's body has no alternative legal shape. The numerical contract closes the remaining freedom — `StrictF32NumericalContract::governed` forbids reassociation, so the fused program's combine order is not a choice either.

*Condition (b) — an external producer needing its own spelling.* No such producer exists. `KernelBuilder` is public and `crates/tiler-ir/src/kernel/tests.rs` uses it to hand-build a producer kernel, but that test's purpose is the opposite of the trigger: it proves the hand-built kernel reaches the *same* verified product and identity as the canonical lowering. It is evidence the single spelling is currently sufficient, not that it is constraining.

*Breadth work has not moved this.* `own-operation-family-support-matrix` is `done`, but it is a `contracts/navigation` ticket that added a maturity-tracking owner; it enumerates recognized-versus-supported state and explicitly records that the first profile is four strict-`f32` operations. Documenting the breadth gap did not widen the implemented surface.

### Re-evaluation after bounded pointwise expressions (inspected source, 2026-07-28 working tree based on `6a7278f`)

**The vocabulary check fired, and neither reconsideration condition did.** `broaden-governed-physical-support-for-reassociated-programs` replaced the fixed `ScalarProgram::MultiplyThenAdd` case with `ScalarProgram::PointwiseF32(PointwiseF32Expression)`. The exhaustive `body_shaping_vocabulary_is_closed` match had to change before the tree compiled, which is the announcement mechanism this ticket installed. Reading the new representation and lowering shows why the gate itself remains correct: a verified expression retains one canonical topological node order, exact constant bits, ordered operands, DAG sharing, and an explicit root; `derive_canonical` walks that exact expression through a total `PointwiseF32Node` match and emits one determined KIR body, including the required NaN canonicalization after every arithmetic node. The representation admits more exact expressions, but no one exact scheduled region admits two legal spellings.

**The same result holds for external production.** `KernelBuilder` remains public, but no consumer has appeared that needs a noncanonical body accepted for the same schedule. A producer can author the body the schedule determines and reach the existing verified product; accepting a second spelling is still neither required nor proved.

**The new nested vocabulary is guarded at both boundaries.** `PointwiseF32Node` is intentionally exhaustive because schedule identity and structured-KIR lowering are total maps over it. `ScalarProgram` remains exhaustive because the compiler recognizes its support out of crate, and `body_shaping_vocabulary_is_closed` separately prevents the same-crate product recognizer's wildcard from absorbing a future body-shaping variant. This refresh therefore records an exercised warning system, not a fired replacement trigger.

### What landed: the trigger is now a compile error

The risk this ticket carries is not that the gate is wrong today — it is that the profile widens later and nobody connects the resulting `BodyRefinement` rejections to a deliberate bounded decision recorded in a ticket. A deferral whose trigger depends on recall is a deferral that fires late, after someone has debugged a valid kernel being rejected.

`crates/tiler-ir/src/kernel/tests.rs::body_shaping_vocabulary_is_closed` matches all five vocabularies above exhaustively with no wildcard arm, so adding a variant to any of them is a **compile error in that function**, carrying a comment naming this ticket and telling the author to read it before adding an arm — because the right response may be to widen the gate rather than the match. `the_single_spelling_profile_is_still_narrow_enough_for_derive_and_compare` exercises it against the canonical pointwise region.

This is a spelling check, not a semantic one, and is documented as such: it cannot tell that a widened vocabulary admits two bodies, only that the vocabulary widened — which is exactly the point at which a human has to make the judgement this ticket reserves for them. It weakens nothing; it makes the existing decision loud.

### Status

Left `deferred` rather than `done`. The 2026-07-28 pointwise widening exercised the compile-time warning and was re-evaluated without finding either trigger condition. The stated work — replacing derive-and-compare with a normalizer or a checked equivalence relation carrying its own soundness argument — remains undone and correctly so. Reconsider whenever the exhaustive vocabulary check fires, but replace the gate only when inspection finds more than one legal body for one exact scheduled region or an external producer genuinely needs another spelling accepted.

## Trigger check log

- 2026-08-04 — **not fired**, re-evaluated after four further vocabulary widenings that each fired the compile-time tripwire. `body_shaping_vocabulary_is_closed` (`crates/tiler-ir/src/kernel/tests.rs:853-904`) now matches five `LogicalAccess` variants, six `ReductionTopology` variants, and seven `ScalarProgram` variants — against the 2026-07-28 record's two, two, and three. **Neither reconsideration condition holds, and the reason is the pair that did *not* widen:** `ExecutionBinding` still has only `GlobalLinearInvocation` and `TailPolicy` still only `Exact`, which the ticket's own derivation identifies as the substance of single-spelling — one way to map invocations to coordinates and no remainder to handle leaves a region's body no alternative legal shape, and `derive_canonical` stays a deterministic function of the scheduled region. No external producer needs a noncanonical body accepted. Recheck: `grep -n 'ExecutionBinding::\|TailPolicy::' crates/tiler-ir/src/kernel/tests.rs`.
- 2026-08-09 — **not fired.** Caller-installed physical providers now propose checked scheduled regions and kernel subprograms, but they do not supply an alternative `VerifiedKernel` spelling: each scheduled body still passes through the same deterministic canonical lowering and derive-and-compare verifier. `ExecutionBinding` remains `GlobalLinearInvocation` and `TailPolicy` remains `Exact`; no exact scheduled region has acquired a second proved body spelling.
- 2026-08-18 — **not fired**, re-evaluated because `admit-vector-lane-bindings-into-the-schedule-vocabulary` widened `ExecutionBinding` with `FixedVectorMap { lanes }` and the compile-time tripwire fired as designed. The earlier entries' premise that `ExecutionBinding` and `TailPolicy` never widened is stale — `BlockedWorkgroup` and `TailPolicy::Predicated` are present at this base and the tripwire matches them — but neither reconsideration condition holds for the new variant: `plan` in `crates/tiler-ir/src/kernel/lower.rs` refuses `FixedVectorMap` as `KernelDiagnostic::UnloweredExecutionBinding` before any body is derived, and the refinement gate's builtin match refuses it independently, so a region carrying it has **zero** legal bodies rather than two, and `derive_canonical` stays a deterministic function of every region that has one. No external producer needs a noncanonical spelling. Recheck: `grep -n 'UnloweredExecutionBinding' crates/tiler-ir/src/kernel/lower.rs crates/tiler-ir/src/kernel/verify.rs`.
