---
id: separate-the-process-alarm-readiness-check-from-its-deadline
title: Separate the process-alarm readiness check from the deadline it tests
status: done
priority: p1
dependencies: []
related: [remove-the-wall-clock-race-from-the-cache-kill-harness]
scopes: [research/extensions]
shared_scopes: []
paths: []
tags: [tests, flake, spikes, gate]
---
**Measurement — 2026-07-25, this host, full gate with seven concurrent agent worktrees compiling.** `scripts/tests/test_research_harnesses.py::test_retained_research_harness_contracts[command3]` failed:

```
extension probe failed: process-alarm child did not start
Command '[... 'spikes/extensions/run.py', '--self-test']' returned non-zero exit status 1
1 failed, 348 passed in 298.66s
```

`.venv/bin/python spikes/extensions/run.py --self-test` then exited 0 on the same tree. Load-dependent, and not a regression — the commit under test changed only `tickets/**`.

**Fact — the mechanism, `spikes/extensions/run.py:740-760`.** The self-test spawns a child that writes its PID to a file and then sleeps 10 s, under a deadline of `time.monotonic() + 5`. It *requires* the deadline to fire — `else: raise ProbeFailure("process-alarm self-test unexpectedly succeeded")` — which is legitimate, because the alarm firing is the behaviour under test. It then asserts the child actually started, by `if not child_pid.is_file()`.

**Inference — one timer is serving two purposes and only one of them is the subject.** The 5 s window is simultaneously the deadline being proven and the budget within which a fresh CPython interpreter must boot and complete a `write_text`. On a saturated machine the second is not guaranteed, so the alarm kills the child before it records itself and the probe reports "did not start" for a child that started and was simply slow.

**This is the second instance of the class in one session.** `remove-the-wall-clock-race-from-the-cache-kill-harness` was the first, and its resolution is the template: establish the precondition by *observation* rather than by hoping it wins a race. Read that ticket's Outcome before designing — note especially that its stated mechanism turned out to be backwards, so verify this one's account against the code rather than trusting the paragraph above.

**The distinction to preserve.** A deadline test is legitimate when the property is true *because the deadline fired* — `a_stuck_child_is_killed_at_its_deadline` in the cache harness is the reference case. It is illegitimate when some *other* assertion depends on work completing inside the same window. Here the alarm assertion is fine and the readiness assertion is not; the fix is to stop making them share a timer, not to remove either.

**Do not widen the window.** Raising 5 s to 30 s preserves the failure mode and makes it rarer, which makes it harder to diagnose when it fires and slower for every green run. `AGENTS.md`: remove a limit that is not needed and fail loud when one is. A margin that is missed reports a *correctness* failure for a *scheduling* event, which is the opposite of an explainable error.

**Do not fix it by deleting the readiness check.** It exists so that "the alarm fired" cannot be satisfied by a child that never ran — without it the test would pass for entirely the wrong reason, which is the same trap the Apple numerical probes defeat with an execution witness. Have the child announce readiness and have the deadline start from that observed point, so both assertions stay and neither races.

**Sweep the siblings while you are here.** `run_command` and its callers throughout `spikes/extensions/run.py` share the `time.monotonic() + N` idiom. Report, for each, whether the deadline is the subject or a budget some other assertion depends on — the cache precedent showed that sharing an idiom is *not* the same as sharing the defect, so classify rather than assume. `scripts/tests/test_research_harnesses.py` runs three other retained harnesses in the same parametrization; check whether any has the same shape.

## Closes when

No assertion in the extension probe's self-test depends on work completing inside the window whose expiry is the thing being proven; the readiness check still fails when a child genuinely does not start, demonstrated by a deliberate break; every sibling deadline is classified as subject or budget; and the full gate passes under concurrent load, which is the condition that produced the failure.

## Outcome

**Fixed. The diagnosis above named the wrong timer — the second time in one session that this class was mis-diagnosed on first reading, which is itself the finding.**

**Fact — the margin is `signal.setitimer(signal.ITIMER_REAL, 0.2)`, not `time.monotonic() + 5`.** The 5 s value is `run_command`'s own deadline and is generous. The window the readiness check actually raced is **200 milliseconds**, inside which a freshly spawned CPython interpreter had to boot, import three modules, and complete a `write_text`. On an idle machine that is comfortable; under seven concurrent Rust builds it is not, so the alarm fired first and the probe reported "child did not start" for a child that started and was merely slow.

**Delivered — the alarm is armed from the child's own announcement.** `arm_alarm_once_started` polls for the announcement file and only then calls `setitimer(..., 0.05)`. The ordering the block depends on — child running, *then* alarm — becomes an observed fact rather than a margin. Polling is used deliberately rather than a synchronization primitive: the announcement is written by a separate process, and a file appearing is the only signal both sides already share.

**The poll is unbounded on purpose, and that is what keeps the failure loud.** `run_command` already holds its own deadline, so a child that never announces surfaces there, with its own distinct message, instead of as an alarm this thread invents. Both assertions in the block survive: that the alarm interrupts a running child, and that the child got far enough to record its pid.

**Measurement — the check can still say no.** Two deliberate breaks, each reverted:

- Child never writes its pid → `overall timeout expired during process-alarm self-test`, exit 1. That message is *not* `extension-suite timeout`, so the `except` arm re-raises rather than accepting a timeout it did not mean.
- Alarm never armed → identical loud failure, exit 1.

**Measurement — the fix holds where the defect fired.** 5/5 consecutive passes idle; 3/3 under eight artificial spinners. The unfixed harness failed the full repository gate twice in a row under the session's real load.

**Sibling classification.** The `time.monotonic() + N` deadlines at the `timeout self-test` (0.05 s) and `output-limit self-test` (5 s) are *subjects*, not budgets: each proves that its own expiry or limit fires, and no other assertion in either block depends on work completing inside the window. `require_time` and `overall_timeout_handler` are called directly with no child process. This block was the only one holding two assertions on one timer.

**Recorded for the next occurrence.** Both instances of this class today were mis-diagnosed on first reading, and in the same direction: the timer named in the failure message was not the timer that caused it. The cache harness's stated mechanism was backwards, and this ticket blamed the wrong constant. The reliable move is to find *every* timer governing the failing block and ask which assertions each one bounds, rather than reading the nearest one.
