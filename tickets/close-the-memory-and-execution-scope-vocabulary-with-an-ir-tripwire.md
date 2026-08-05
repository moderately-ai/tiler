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

> **The `emit.rs` half of this ticket's own numbers has drifted since `0017345`, corrected 2026-08-04 by the stale-claim sweep at base `c4b4bdb9`. This paragraph told a reader to trust these numbers over the addenda's, so its drift costs more than an addendum's would.** Read rather than searched: `barrier_call` is `crates/tiler-metal/src/emit.rs:1601`; its execution-scope `_ =>` arm is `:1605`; the `(execution_scope, memory_scope)` match opens at `:1618` with its `_ =>` arm at `:1620` and the `BarrierRejection::MemoryVisibility` return at `:1622`; the ordering match's `_ =>` arm is `:1633`; `fence_flag` is `:1659` with its `_ =>` arm at `:1666`. **The `tiler-ir` half is unchanged and re-verified**: `ExecutionScope` is `crates/tiler-ir/src/kernel/model.rs:595` and `MemoryScope` is `:614`, exactly as this ticket states. The tripwire model is `body_shaping_vocabulary_is_closed` at `crates/tiler-ir/src/kernel/tests.rs:853` — one line down from the `:852` above — referenced at `:911` and consumed at `:916`. Reproduce with `grep -n 'fn barrier_call\|fn fence_flag' crates/tiler-metal/src/emit.rs`, `grep -n 'pub enum ExecutionScope\|pub enum MemoryScope' crates/tiler-ir/src/kernel/model.rs`, and `grep -n 'body_shaping_vocabulary_is_closed' crates/tiler-ir/src/kernel/tests.rs`. **Every argument above survives unchanged** — both wildcard arms are still there, `fence_flag` still ends the same way, and the closure pattern is still the only tripwire in the tree.

## Boundaries

- A spelling check, not a semantic one, exactly as the body-shaping precedent states. It cannot tell that a widened vocabulary admits a new barrier — only that the vocabulary widened, which is the moment a human has to look.
- Do not relax any rejection. `tiler-metal` refusing a subgroup barrier rather than widening the claim to workgroup visibility is correct and stays; a barrier claiming broader visibility than the hardware primitive provides is a data race the verifier would have blessed.
- Adding the tripwire admits no scope and emits nothing. `MemoryScope::Subgroup` itself remains [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md)'s, whose trigger is now [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md). **The second half of that sentence is refuted, corrected 2026-08-04 by the stale-claim sweep; the first half stands.** That ticket's own 2026-08-01 second addendum derived from MSL 4.1 §6.16.2 and §6.10.1 that a handoff *between* SIMD-groups needs **threadgroup**-scoped visibility, which `required_subject` already derives — so the two-level reduction never fires it, and it is now `done` without this capability, which is what the refutation predicted. That ticket's 2026-08-04 trigger check log rewrote the trigger to **the first schedule declaring a staged allocation through threadgroup memory whose writer and every reader lie in one subgroup** — a subgroup-private scratch tile — and deliberately added no frontmatter edge, because an edge whose premise is disproved makes unreachable work look reachable. Nothing in the graph proposes such a schedule, so the trigger is a corpus-state condition with no owning ticket. Reproduce by reading that ticket's "The proposed narrowing is wrong too" section and its trigger check log. **This ticket is unaffected either way:** it is dispatchable now precisely because the tripwire is worth having before any trigger fires.

## Closes when

Test-only exhaustive matches over `MemoryScope` and `ExecutionScope` exist in `crates/tiler-ir`, each naming the ticket that must act when it breaks; the tripwire is demonstrated failing by adding a throwaway variant locally and observing the build error, and that perturbation is reverted; and the correct current citations replace the stale ones the deferred ticket's addenda carry.
