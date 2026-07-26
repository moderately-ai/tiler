---
id: remove-the-wall-clock-race-from-the-cache-kill-harness
title: Remove the wall-clock race from the expansion cache kill-phase harness
status: done
priority: p1
dependencies: []
related: [prototype-expansion-content-cache, inject-deterministic-expansion-cache-io-failures, design-bounded-expansion-cache-garbage-collection]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [implementation, cache, tests, flake]
---
**Measurement — 2026-07-25, this host, during a full-gate run with four concurrent agent worktrees compiling.** `expansion::harness::processes_racing_a_dying_writer_still_resolve` failed the repository gate:

```
crates/tiler-cache/src/expansion/harness.rs:809:9:
assertion `left == right` failed: repetition 0: the armed child must die
  left: Completed
 right: Terminated
```

The same test passed 3/3 in isolation and the full `-p tiler-cache --lib` suite passed 2/2 (86 tests) immediately afterward on an idle machine. So the failure is load-dependent, not order-dependent and not a regression: the merge that surfaced it changed only `docs/**` and `tickets/**`.

**Fact — the mechanism, from `harness.rs:795-812`.** The armed run is `Run::new(...).killed_at(Phase::AfterLock).with_build_delay(Duration::from_millis(50))`, and the assertion requires `deaths[0] == Death::Terminated`. The only thing keeping the armed child alive long enough to be killed is a 50 ms wall-clock sleep. When the machine is saturated, the killer is scheduled late, the child completes its build, and the run ends `Completed`.

**Inference — this is a defect in the test, not an environment artifact to be waited out.** A gate assertion whose truth depends on winning a scheduling race has two costs, and the second is worse than the first. It fails in CI, which is by definition a loaded machine. And because a re-run passes, it teaches every reader that a red gate on this test means nothing — which is precisely the habit that let a genuinely red commit reach `origin/main` earlier in this session. A check that cries wolf is a check being trained out of the workflow.

**The obligation is to remove the race, not to widen the margin.** Raising 50 ms to 500 ms buys a larger window and keeps the same failure mode, now rarer and therefore harder to diagnose when it fires. `AGENTS.md`'s standing rule applies in its usual form: remove the limit if it is not needed, and fail loud if it is. A timing margin is a limit that cannot fail loud — when it is missed the test reports a *correctness* failure for a *scheduling* event, which is the opposite of an explainable error.

**What the test is actually for, and must keep proving.** That a writer killed after taking the lock leaves the cache recoverable and that the processes racing it still resolve. That property is about *ordering* — the kill must be observed to happen at `AfterLock` — and ordering can be established without wall-clock: have the child signal that it has reached the phase and block until released, so the killer acts on an observed state rather than a hoped-for one. Sibling tests in the same module (`a_writer_killed_at_any_phase_leaves_a_recoverable_cache`, `a_killed_writers_lock_is_released_without_a_recovery_rule`, `a_stuck_child_is_killed_at_its_deadline`) should be read before choosing a mechanism — if they share the `with_build_delay` idiom they share the defect, and `AGENTS.md`'s find-one-bug-check-all-siblings rule makes that sweep part of this ticket rather than a follow-up.

**Do not fix it by deleting the assertion or marking the test `#[ignore]`.** The property is real and the harness exists to prove it. A deadline-based kill (`a_stuck_child_is_killed_at_its_deadline`) is a legitimately time-dependent test because the deadline *is* the behaviour under test; this one's subject is a phase, not a duration.

## Closes when

No assertion in the kill-phase harness depends on a wall-clock margin to be true, every sibling sharing the idiom is fixed or shown not to share the defect, the recoverability and resolution properties are still proven, and the full gate passes — including under concurrent load, which is the condition that produced the failure.

## Outcome

**Landed in `bc4e1ff`. The race named above is real; this ticket's account of its mechanism was backwards, and the correction is the interesting part.**

**Fact — `store.rs:471` puts `fault::reach(Phase::AfterLock)` *below* the lock-free lookup, and `resolve` returns `Hit` at line 449 when that lookup succeeds, before the fault seam.** So an armed child whose lookup hits never reaches `after-lock` at all. There is no separate killer and no "killer scheduled late": the child kills itself, and it never builds. The 50 ms delay was not keeping the armed child alive — it was keeping the **other** children from publishing before the armed child's lookup ran. Under load the armed child's process startup exceeds a survivor's entire publish, so it hits, exits zero, and reports `Completed`. Confirmed directly by a throwaway probe: publish first, then arm a child at `AfterLock`, and it completes 100% of the time.

**Delivered — `fault::rendezvous()`**, called from `resolve` at the one decidable point: the lookup has run and missed, and no lock is held. A gated child writes its own arrival file and blocks; the parent releases them only once **every** arrival file exists. From there the armed child reaches `after-lock` regardless of what any other process does, including down the fall-open path, because `reach` sits after the `lock.is_some()` fork. It cannot race, because the precondition — a missed lookup — is established by observation before any process is permitted to lock. Arrival is one file per child rather than a counter, so the parent names its population and counts it.

**The ticket's sibling instruction was wrong and the sweep was done properly anyway.** Sharing the `with_build_delay` idiom is *not* sharing the defect: three siblings use it and none has an assertion whose truth depends on it. They were gated regardless, because the barrier converts claims that were previously unverifiable hopes — "concurrent", "overlap" — into checked facts. Two delays were deliberately **kept** with stated reasons: one widens a *span*, which a barrier cannot express, and one holds a shared key's lock long enough for a collector to meet a contended candidate. `a_stuck_child_is_killed_at_its_deadline` is legitimately time-dependent — the deadline *is* its subject — and is untouched.

**The new checks can say no, proven rather than asserted.** Four deliberate breaks, each reverted: a no-op rendezvous fails in 0.016 s naming how many of how many children arrived; parking without announcing fails at the deadline with children reaped and no orphans; disarming the child reproduces the original gate failure exactly; and removing the post-lock recheck yields four compiles where a serial execution could only ever produce one — which is positive proof the barrier works, since it means all four children genuinely passed their lookup before any published.

**Two-sided evidence.** 25 idle repeats and 20 under load pass; harness stress passes at 25×16 and 40×24 children under load. Restoring the base files and running the identical stress **failed on run 1**, so the flake reproduces at roughly 50% in that configuration and the fix survives both it and a harder load.

**Nothing was weakened.** No assertion deleted, no test `#[ignore]`d, no retry loop, no conditional assertion. All 86 tests run with unchanged assertions.

**One time bound remains and is correct.** `CHILD_DEADLINE = 30s` in `wait_bounded` bounds a hang and fails loud; no property is true *because* 30 s elapsed, which is the distinction that separates it from the margin removed here.
