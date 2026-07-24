---
id: make-spike-process-group-cleanup-best-effort
title: Make spike process-group cleanup best-effort so it cannot fail the gate
status: done
priority: p1
dependencies: []
related: [record-source-verification-rule-in-agent-guidance]
scopes: [research/embedding, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
---
The repository gate fails intermittently on `os.killpg`, and because it is the
**gate** that fails, every ticket on the host inherits the flakiness.

Five unguarded call sites: `spikes/extensions/run.py:55`, and
`spikes/embedding/measure.py:359,373,386,395`. Each raises out of the harness if
the process group cannot be signalled.

Two distinct triggers, both observed:

- **Already-exited child.** In `test_run_logged_enforces_output_limit` the child
  (`print('x' * 1000)`) usually exits before the capture-limit branch fires, so
  the group holds only an unreaped child and signalling it can return `EPERM`.
  Measured: of four gate invocations at one unmodified tree, two exited 0 and two
  exited 1 on exactly this test. It did **not** reproduce under narrow runs
  (12/12 for the test alone, 11/11 for the directory), which is why it reads as
  flake rather than defect.
- **Sandboxed execution.** A second agent independently hit the same syscall
  being refused outright in `spikes/extensions/run.py`, and had to run the gate
  outside its sandbox to get past it.

Both were provably unrelated to the changes under test — in both cases
`git diff --name-only <base> HEAD -- spikes/` was empty.

**Fix:** treat process-group termination as best-effort. Catch
`ProcessLookupError` and `PermissionError` around each `killpg` and continue;
cleanup that cannot signal a group is not a test failure, and a harness must not
fail the run it is tidying up after.

**Do this carefully — the calls are not all mere cleanup.** Some enforce a limit
(an output cap, a timeout), and there the kill is load-bearing: silently
swallowing its failure would let a harness report success while a runaway child
was never actually stopped. For each site decide which it is. Where the kill
enforces a limit, keep the assertion that the limit *was* enforced by an
independent observation (captured bytes, elapsed time, exit status) rather than
inferring it from the kill having been attempted.

While here, note a **host-local confound that is not this defect** and must not
be folded into it: `spikes/macro-environment/test_probe.py::test_overall_alarm_reaps_child_after_capture_pipes_close`
fails when run from an interactive shell because a bare `python3` resolves to a
pyenv shim (~0.76 s startup) and loses a 0.2 s `setitimer` race. The gate is
unaffected because `sanitized_environment()` puts `.venv/bin` first. That is an
invocation artifact, not a repository defect, and fixing it is not in scope here.

This matters beyond tidiness: an intermittently red gate erodes the signal the
whole review discipline rests on. A gate that fails at random teaches people to
re-run until green, which is exactly how a real failure gets waved through.

## Outcome

Both harnesses now terminate process groups on a best-effort basis through one helper each: `kill_process_group` in `spikes/embedding/measure.py` (new, replacing the four inlined blocks) and the widened helper in `spikes/extensions/run.py`. Each tolerates `ProcessLookupError` and `PermissionError`, falls back to signalling the child the harness directly owns, and bounds the reap at `CLEANUP_REAP_SECONDS`. The bound is part of the fix, not a nicety: suppressing the signal failure while keeping an unbounded `process.wait()` would trade a crash for a hang, and would let a cleanup that never delivered a signal wait out a child's natural exit and then observe it "gone" — the silent false pass this ticket warns against.

Site classification:

- `spikes/extensions/run.py` helper, called from `run_command` at the pre-read deadline check, the output-cap branch, the post-loop deadline check, the `wait` timeout handler, and the `finally`. Load-bearing at the cap and the three deadline branches; pure cleanup in `finally`.
- `spikes/embedding/measure.py` in-loop deadline branch — load-bearing (timeout).
- `spikes/embedding/measure.py` capture-cap branch — load-bearing (output cap).
- `spikes/embedding/measure.py` `TimeoutExpired` handler — load-bearing (timeout).
- `spikes/embedding/measure.py` `finally` — pure cleanup, reached only when the child outlived the body.

Independent observations retained for the load-bearing sites:

- Output cap: `test_run_logged_enforces_output_limit` now asserts the bytes actually retained on disk stay within the cap, instead of only that the harness raised.
- Timeout: `test_run_logged_enforces_deadline` now asserts elapsed wall time is below `CLEANUP_REAP_SECONDS`. The bounded reap makes that observation load-bearing — a kill that failed to land costs the full grace period, and waiting out the 30 s child costs far more.
- Effective termination: `run.py`'s existing process-alarm self-test still requires `os.kill(child, 0)` to raise `ProcessLookupError`, and a new self-test plus `test_kill_process_group_stops_child_when_group_signal_is_refused` require a returncode of `-SIGKILL` when the group signal is refused, so the fallback is verified to stop a live child rather than merely to have been attempted.

**Measurement** (macOS arm64, Darwin 27.0.0, CPython 3.11.12, worktree at base `6555119`, host shared with other agents):

- `killpg` against a group whose only member is an exited-but-unreaped child raised `PermissionError` errno 1 on 200 of 200 attempts, and on 200 of 200 attempts when the exit was observed through end of file on the child's stdout. The `EPERM` trigger is deterministic; only the race that reaches that state is intermittent.
- Against the pre-fix cleanup, the two new `kill_process_group` tests fail 2 of 2 with `PermissionError: [Errno 1] Operation not permitted` raised from `os.killpg`, and `spikes/extensions/run.py --self-test` exits 1 for the same reason.
- Pre-fix, the gate's pytest stage under a gate-equivalent environment failed 2 of 25 runs, both on `test_run_logged_enforces_output_limit` with that same `PermissionError`. The reported flake reproduces here.
- Post-fix, the same loop passed 70 of 70 across two batches, and the complete gate passed 7 of 8 invocations.

**The one post-fix gate failure was this same defect at a site outside this ticket's scopes, so the gate is not yet reliable.** `spikes/macro-environment/probe.py:162` raised `PermissionError: [Errno 1] Operation not permitted` from `os.killpg` and failed `test_command_capture_rejects_output_while_streaming`. That test is the exact analogue of the embedding one — a `print('x' * 1000)` child that has usually exited by the time the output-cap branch fires — and `spikes/macro-environment` is inside the gate's pytest testpaths. It is unrelated to the pyenv `setitimer` confound described above, which the gate does avoid. Fixing `research/embedding` and `research/extensions` therefore removes two flaky surfaces but not the last one; closing the ticket's stated goal needs a follow-up covering `research/macro-environment`.

**Fact, out of scope here:** nine further unguarded `killpg` sites share this defect and were not touched, because they fall outside this ticket's scopes — `spikes/macro-environment/probe.py` (four, one of which is measured above), `spikes/numerics/sound_accuracy/daisy_runner.py`, `spikes/runtime/measure_semantic_validation.py`, `spikes/shapes/shape-evidence/measure.py`, and `spikes/shapes/nightly-dependent-static-shapes/measure.py`. Each also follows its `killpg` with an unbounded `process.wait()`.

**Measurement, refining the host-local confound above:** under an unsanitized `PATH`, where a bare `python3` is the pyenv shim, `test_overall_alarm_reaps_child_after_capture_pipes_close` does not only fail. One run of that test stopped making progress for more than nine minutes until the surrounding loop was terminated, against a child that sleeps ten seconds. The gate does avoid the shim through `sanitized_environment()`, so that remains an invocation artifact, but the failure mode is a hang rather than an assertion failure, and `probe.py`'s unbounded reap is the reason.
