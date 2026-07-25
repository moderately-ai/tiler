---
id: remove-the-wall-clock-race-from-the-cache-kill-harness
title: Remove the wall-clock race from the expansion cache kill-phase harness
status: in-progress
priority: p1
dependencies: []
related: [prototype-expansion-content-cache, inject-deterministic-expansion-cache-io-failures, design-bounded-expansion-cache-garbage-collection]
scopes: [implementation/cache]
shared_scopes: []
paths: []
tags: [implementation, cache, tests, flake]
claimed_from: todo
assignee: agent-remove-the-wall-clock-race-from-the-cache-kill-harness
lease_expires_at: 1785045284
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
