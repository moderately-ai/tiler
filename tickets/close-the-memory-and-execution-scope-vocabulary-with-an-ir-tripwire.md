---
id: close-the-memory-and-execution-scope-vocabulary-with-an-ir-tripwire
title: Close the memory and execution scope vocabulary with an IR tripwire
status: todo
priority: p2
dependencies: []
related: [add-subgroup-memory-scope-when-collectives-land, compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [implementation/ir, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, barriers, tripwire]
---
## User-visible outcome

Widening `MemoryScope` or `ExecutionScope` becomes a build error naming the ticket that must then act, instead of compiling cleanly while every barrier at the new scope keeps being rejected silently at run time.

## Why this exists

**Fact — both matches in the Metal barrier emitter end in a wildcard.** `barrier_call` (`crates/tiler-metal/src/emit.rs:1431`) matches `spec.execution_scope` with a `_ =>` arm at `:1435` returning `BarrierRejection::ExecutionScope`, then matches `(spec.execution_scope, spec.memory_scope)` admitting only `(Workgroup, Workgroup)` with a `_ =>` arm at `:1450` returning `BarrierRejection::MemoryVisibility`. `fence_flag` (`:1489`) ends the same way at `:1496`.

**Inference — the vocabulary widening that most needs announcing is exactly the one nothing announces.** Adding `MemoryScope::Subgroup` to `crates/tiler-ir/src/kernel/model.rs:614` compiles cleanly in `tiler-metal`, and every subgroup barrier keeps being rejected by the arm at `emit.rs:1450` — silently, and at run time. This is not an argument for deleting the wildcards: they are what make an unhandled scope a typed `UnsupportedBarrier` rather than a panic, and both enums are `#[non_exhaustive]`, so an out-of-crate match needs one regardless.

**Fact — the closure pattern already exists, once.** `body_shaping_vocabulary_is_closed` at `crates/tiler-ir/src/kernel/tests.rs:852` is a test-only exhaustive match whose only job is to fail to compile when the vocabulary widens; it is referenced as the model at `:910` and consumed at `:915`. It is the only such tripwire in the tree. Mirror it for `MemoryScope` (`model.rs:614`) and `ExecutionScope` (`model.rs:595`).

**Fact — the addendum that prescribed this cannot carry it.** The 2026-08-01 addendum on [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) names the fix in exactly these terms and points at the same pattern, but that ticket is `deferred` behind a trigger that has not fired, so the tripwire it prescribes has no dispatchable owner. That addendum also corrected the 2026-07-28 addendum's line citations at `8252312`; **those corrections have themselves drifted** — `ExecutionScope` and `MemoryScope` are at `model.rs:595` and `:614`, not `:562` and `:581`, and `barrier_call` is at `emit.rs:1431` with the visibility match at `:1448-1458`, not `:1298` and `:1312-1322`. Use the numbers in this ticket, verified at base `0017345`, and do not follow either addendum's.

## Boundaries

- A spelling check, not a semantic one, exactly as the body-shaping precedent states. It cannot tell that a widened vocabulary admits a new barrier — only that the vocabulary widened, which is the moment a human has to look.
- Do not relax any rejection. `tiler-metal` refusing a subgroup barrier rather than widening the claim to workgroup visibility is correct and stays; a barrier claiming broader visibility than the hardware primitive provides is a data race the verifier would have blessed.
- Adding the tripwire admits no scope and emits nothing. `MemoryScope::Subgroup` itself remains [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md)'s, whose trigger is now [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md).

## Closes when

Test-only exhaustive matches over `MemoryScope` and `ExecutionScope` exist in `crates/tiler-ir`, each naming the ticket that must act when it breaks; the tripwire is demonstrated failing by adding a throwaway variant locally and observing the build error, and that perturbation is reverted; and the correct current citations replace the stale ones the deferred ticket's addenda carry.
